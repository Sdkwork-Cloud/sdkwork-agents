use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;

use serde::Deserialize;

type BuildResult<T> = Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Deserialize)]
struct OpenApiDocument {
    paths: BTreeMap<String, BTreeMap<String, Operation>>,
}

#[derive(Debug, Deserialize)]
struct Operation {
    tags: Option<Vec<String>>,
    #[serde(rename = "operationId")]
    operation_id: String,
    security: Option<Vec<BTreeMap<String, Vec<serde_yaml::Value>>>>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").map_err(|error| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("required CARGO_MANIFEST_DIR environment variable is missing: {error}"),
        )
    })?);
    let app_root = manifest_dir.join("../..");
    let out_dir = PathBuf::from(env::var("OUT_DIR").map_err(|error| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("required OUT_DIR environment variable is missing: {error}"),
        )
    })?);

    let surfaces = [
        (
            "agent_app_routes.rs",
            "APP_ROUTES",
            app_root.join(
                "crates/sdkwork-intelligence-agents-service/specs/openapi/agents-app-api.openapi.yaml",
            ),
        ),
        (
            "agent_backend_routes.rs",
            "BACKEND_ROUTES",
            app_root.join(
                "crates/sdkwork-intelligence-agents-service/specs/openapi/agents-backend-api.openapi.yaml",
            ),
        ),
        (
            "agent_open_routes.rs",
            "OPEN_ROUTES",
            app_root.join(
                "crates/sdkwork-intelligence-agents-service/specs/openapi/agents-open-api.openapi.yaml",
            ),
        ),
    ];

    let mut combined_entries = Vec::new();

    for (file_name, const_name, path) in &surfaces {
        let yaml = fs::read_to_string(path).map_err(|error| {
            io::Error::new(
                error.kind(),
                format!(
                    "failed to read OpenAPI authority {}: {error}",
                    path.display()
                ),
            )
        })?;
        let document: OpenApiDocument = serde_yaml::from_str(&yaml).map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "failed to parse OpenAPI authority {}: {error}",
                    path.display()
                ),
            )
        })?;
        let entries = collect_routes(&document);
        combined_entries.extend(entries.iter().cloned());
        let source = render_routes(const_name, &entries);
        write_generated_route_manifest(out_dir.join(file_name), source)?;
    }

    combined_entries.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then(left.method.cmp(&right.method))
    });
    combined_entries.dedup_by(|left, right| left.path == right.path && left.method == right.method);
    let combined_source = render_routes("COMBINED_ROUTES", &combined_entries);
    write_generated_route_manifest(out_dir.join("agent_combined_routes.rs"), combined_source)?;

    println!("cargo:rerun-if-changed=build.rs");
    for (_, _, path) in &surfaces {
        println!("cargo:rerun-if-changed={}", path.display());
    }

    Ok(())
}

fn write_generated_route_manifest(path: PathBuf, source: String) -> BuildResult<()> {
    fs::write(&path, source).map_err(|error| {
        io::Error::new(
            error.kind(),
            format!(
                "failed to write generated route manifest {}: {error}",
                path.display()
            ),
        )
    })?;
    Ok(())
}

#[derive(Clone)]
struct RouteEntry {
    method: String,
    path: String,
    tag: String,
    operation_id: String,
    auth: String,
}

fn collect_routes(document: &OpenApiDocument) -> Vec<RouteEntry> {
    let mut routes = Vec::new();
    for (path, operations) in &document.paths {
        for (method, operation) in operations {
            let Some(http_method) = normalize_method(method) else {
                continue;
            };
            routes.push(RouteEntry {
                method: http_method.to_owned(),
                tag: operation
                    .tags
                    .as_ref()
                    .and_then(|tags| tags.first())
                    .cloned()
                    .unwrap_or_else(|| "ai".to_owned()),
                operation_id: operation.operation_id.clone(),
                auth: classify_auth(operation.security.as_ref()),
                path: path.clone(),
            });
        }
    }
    routes
}

fn normalize_method(method: &str) -> Option<&'static str> {
    match method.to_ascii_lowercase().as_str() {
        "get" => Some("Get"),
        "post" => Some("Post"),
        "patch" => Some("Patch"),
        "put" => Some("Put"),
        "delete" => Some("Delete"),
        _ => None,
    }
}

fn classify_auth(security: Option<&Vec<BTreeMap<String, Vec<serde_yaml::Value>>>>) -> String {
    let Some(entries) = security else {
        return "Public".to_owned();
    };
    let mut has_auth_token = false;
    let mut has_access_token = false;
    let mut has_api_key = false;
    for entry in entries {
        for scheme in entry.keys() {
            match scheme.as_str() {
                "AuthToken" => has_auth_token = true,
                "AccessToken" => has_access_token = true,
                "ApiKey" => has_api_key = true,
                _ => {}
            }
        }
    }
    if has_auth_token && has_access_token {
        "DualToken".to_owned()
    } else if has_auth_token || has_api_key {
        "ApiKey".to_owned()
    } else {
        "Public".to_owned()
    }
}

fn render_routes(const_name: &str, routes: &[RouteEntry]) -> String {
    let mut output = String::from(
        "// @generated by sdkwork-routes-agents-http-shared/build.rs - do not edit\n\n",
    );
    output.push_str(&format!(
        "pub const {const_name}: &[sdkwork_web_contract::HttpRoute] = &[\n"
    ));
    for route in routes {
        output.push_str(&format!(
            "    sdkwork_web_contract::HttpRoute::new(sdkwork_web_contract::HttpMethod::{}, {:?}, {:?}, {:?}, sdkwork_web_contract::RouteAuth::{}),\n",
            route.method, route.path, route.tag, route.operation_id, route.auth
        ));
    }
    output.push_str("];\n");
    output
}
