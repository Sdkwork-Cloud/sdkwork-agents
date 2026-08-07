//! Cloud Router open-api adapter for the SDKWork Agents media tool family.

mod client;
mod vendor;

pub use client::{
    map_cloudrouter_error, run_sync, CloudRouterMediaClient, DEFAULT_CLOUDROUTER_BASE_URL,
    ENV_CLOUDROUTER_BASE_URL,
};
pub use vendor::{
    model_arg, normalize_vendor_status, normalized_vendor_media, optional_i64_arg, string_array_arg,
};
