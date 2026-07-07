//! Application-owned facade for bootstrapping and executing kernel code-engine providers.
//!
//! Product repositories (BirdCoder, IM PC) must depend on this crate instead of
//! importing `sdkwork-agent-provider-*` or `sdkwork-agent-kernel` types directly.

mod code_engines;
mod engine_catalog;
mod error;
mod live_interaction;
mod runtime_host;
mod turn;

pub use code_engines::{
    bootstrap_code_engine, canonical_code_engine_keys, code_engine_agent_id,
    code_engine_binding_id, is_canonical_code_engine, CodeEngineBootstrapError, CodeEngineSlot,
};
pub use engine_catalog::{
    bootstrap_canonical_code_engine_catalog, build_code_engine_catalog, list_slot_catalog_entries,
    model_descriptor_to_catalog_entry, CodeEngineCatalog, CodeEngineCatalogEngine,
    CodeEngineModelCatalogEntry,
};
pub use error::{RuntimeFacadeError, RuntimeFacadeResult};
pub use live_interaction::{
    ApprovalDecision, EngineLiveInteraction, LiveInteractionRegistry, UserQuestionAnswer,
};
pub use runtime_host::AgentsCodeEngineHost;
pub use sdkwork_agent_kernel::{ModelDescriptor, ModelResponseFormat};
pub use turn::{
    execute_code_engine_turn, execute_code_engine_turn_with_stream, CodeEngineTurnInput,
    CodeEngineTurnOutput,
};
