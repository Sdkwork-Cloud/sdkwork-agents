//! Gateway assembly scaffold for sdkwork-agents.
//! Implement `bootstrap.rs` with application-specific service wiring until every route crate exports `gateway_mount`.

mod bootstrap;
mod generated;

pub use bootstrap::{assemble_application_router, ApplicationAssembly};

pub fn assembly_route_count() -> usize {
    generated::ROUTE_CRATE_COUNT
}
