use std::collections::BTreeMap;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
struct InterfaceAddress {
    name: String,
    ip: IpAddr,
}

#[derive(Debug, Eq, PartialEq)]
struct AccessUrl {
    network: String,
    origin: String,
}

pub fn log_access_urls(local_address: SocketAddr) {
    let interfaces = match if_addrs::get_if_addrs() {
        Ok(interfaces) => interfaces
            .into_iter()
            .map(|interface| {
                let ip = interface.ip();
                InterfaceAddress {
                    name: interface.name,
                    ip,
                }
            })
            .collect(),
        Err(error) => {
            tracing::warn!(%error, "failed to enumerate network interfaces");
            Vec::new()
        }
    };
    let access_urls = build_access_urls(local_address, interfaces);

    tracing::info!(
        bind = %local_address,
        count = access_urls.len(),
        "sdkwork-api-agents-standalone-gateway started; accessible URLs"
    );
    for access_url in access_urls {
        tracing::info!(
            network = %access_url.network,
            url = %format!("{}/healthz", access_url.origin),
            "sdkwork-agents health URL"
        );
        tracing::info!(
            network = %access_url.network,
            url = %access_url.origin,
            "sdkwork-agents API origin (authentication required)"
        );
    }
}

fn build_access_urls(
    local_address: SocketAddr,
    interfaces: impl IntoIterator<Item = InterfaceAddress>,
) -> Vec<AccessUrl> {
    let bound_ip = local_address.ip();
    let port = local_address.port();
    let mut addresses = BTreeMap::new();

    if bound_ip.is_unspecified() {
        for interface in interfaces {
            if is_accessible_for_bind(interface.ip, bound_ip) {
                addresses.entry(interface.ip).or_insert(interface.name);
            }
        }
        let loopback = if bound_ip.is_ipv4() {
            IpAddr::V4(Ipv4Addr::LOCALHOST)
        } else {
            IpAddr::V6(Ipv6Addr::LOCALHOST)
        };
        addresses.insert(loopback, "loopback".to_owned());
    } else {
        let interface_name = interfaces
            .into_iter()
            .find(|interface| interface.ip == bound_ip)
            .map_or_else(
                || network_kind(bound_ip).to_owned(),
                |interface| interface.name,
            );
        addresses.insert(bound_ip, interface_name);
    }

    let mut access_urls = addresses
        .into_iter()
        .map(|(ip, name)| AccessUrl {
            network: network_label(&InterfaceAddress { name, ip }),
            origin: format_http_url(ip, port),
        })
        .collect::<Vec<_>>();
    access_urls.sort_by(|left, right| {
        access_url_sort_key(left)
            .cmp(&access_url_sort_key(right))
            .then_with(|| left.origin.cmp(&right.origin))
    });
    access_urls
}

fn is_accessible_for_bind(ip: IpAddr, bound_ip: IpAddr) -> bool {
    if ip.is_unspecified() || ip.is_multicast() || ip.is_ipv4() != bound_ip.is_ipv4() {
        return false;
    }
    match ip {
        IpAddr::V4(ipv4) => !ipv4.is_link_local() && !ipv4.is_broadcast(),
        IpAddr::V6(ipv6) => !ipv6.is_unicast_link_local(),
    }
}

fn network_label(interface: &InterfaceAddress) -> String {
    if interface.ip.is_loopback() {
        "Local".to_owned()
    } else {
        format!("Network ({})", interface.name)
    }
}

fn network_kind(ip: IpAddr) -> &'static str {
    if ip.is_loopback() {
        "Local"
    } else {
        "Network"
    }
}

fn format_http_url(ip: IpAddr, port: u16) -> String {
    match ip {
        IpAddr::V4(ipv4) => format!("http://{ipv4}:{port}"),
        IpAddr::V6(ipv6) => format!("http://[{ipv6}]:{port}"),
    }
}

fn access_url_sort_key(access_url: &AccessUrl) -> (bool, bool) {
    (
        !access_url.network.eq("Local"),
        access_url.origin.contains('['),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn interface(name: &str, ip: IpAddr) -> InterfaceAddress {
        InterfaceAddress {
            name: name.to_owned(),
            ip,
        }
    }

    #[test]
    fn wildcard_ipv4_bind_lists_loopback_and_every_routable_ipv4_interface() {
        let urls = build_access_urls(
            "0.0.0.0:8095".parse().expect("valid socket address"),
            [
                interface("Ethernet", IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10))),
                interface("WSL", IpAddr::V4(Ipv4Addr::new(172, 23, 0, 1))),
                interface("Link local", IpAddr::V4(Ipv4Addr::new(169, 254, 1, 1))),
                interface("IPv6", "2001:db8::1".parse().expect("valid IPv6")),
            ],
        );

        assert_eq!(
            urls,
            [
                AccessUrl {
                    network: "Local".to_owned(),
                    origin: "http://127.0.0.1:8095".to_owned(),
                },
                AccessUrl {
                    network: "Network (WSL)".to_owned(),
                    origin: "http://172.23.0.1:8095".to_owned(),
                },
                AccessUrl {
                    network: "Network (Ethernet)".to_owned(),
                    origin: "http://192.168.1.10:8095".to_owned(),
                },
            ]
        );
    }

    #[test]
    fn explicit_bind_only_lists_the_bound_address() {
        let urls = build_access_urls(
            "192.168.1.10:9000".parse().expect("valid socket address"),
            [
                interface("Ethernet", IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10))),
                interface("Wi-Fi", IpAddr::V4(Ipv4Addr::new(10, 0, 0, 10))),
            ],
        );

        assert_eq!(
            urls,
            [AccessUrl {
                network: "Network (Ethernet)".to_owned(),
                origin: "http://192.168.1.10:9000".to_owned(),
            }]
        );
    }

    #[test]
    fn wildcard_ipv6_bind_formats_global_addresses_as_valid_urls() {
        let urls = build_access_urls(
            "[::]:8095".parse().expect("valid socket address"),
            [
                interface("Ethernet", "2001:db8::10".parse().expect("valid IPv6")),
                interface("Link local", "fe80::1".parse().expect("valid IPv6")),
                interface("IPv4", IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10))),
            ],
        );

        assert_eq!(
            urls,
            [
                AccessUrl {
                    network: "Local".to_owned(),
                    origin: "http://[::1]:8095".to_owned(),
                },
                AccessUrl {
                    network: "Network (Ethernet)".to_owned(),
                    origin: "http://[2001:db8::10]:8095".to_owned(),
                },
            ]
        );
    }
}
