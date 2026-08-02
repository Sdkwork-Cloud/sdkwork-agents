//! Application-owned facade for bootstrapping and executing kernel code-engine providers.
//!
//! Product repositories (BirdCoder, IM PC) must depend on this crate instead of
//! importing `sdkwork-agent-provider-*` or `sdkwork-agent-kernel` types directly.

mod code_engines;
mod engine_catalog;
mod error;
mod live_interaction;
mod provider_sessions;
mod runtime_host;
mod sessions;
mod turn;

pub use code_engines::{
    apply_code_engine_model_configuration, apply_code_engine_model_selection,
    bootstrap_code_engine, bootstrappable_engine_keys, canonical_code_engine_keys,
    code_engine_agent_id, code_engine_binding_id, is_canonical_code_engine,
    resolve_code_engine_runtime_identity, CodeEngineBootstrapError,
    CodeEngineInteractionResolution, CodeEngineRuntimeIdentity, CodeEngineSlot,
    CANONICAL_CODE_ENGINE_KEYS,
};
pub use engine_catalog::{
    bootstrap_bootstrappable_code_engine_catalog, bootstrap_canonical_code_engine_catalog,
    build_code_engine_catalog, list_slot_catalog_entries, model_descriptor_to_catalog_entry,
    CodeEngineAccessModeCatalogEntry, CodeEngineCatalog, CodeEngineCatalogEngine,
    CodeEngineModelCatalogEntry,
};
pub use error::{RuntimeFacadeError, RuntimeFacadeResult};
pub use live_interaction::{
    ApprovalDecision, EngineLiveInteraction, LiveInteractionRegistry, UserQuestionAnswer,
};
pub use provider_sessions::{
    ProviderSessionDirectoryEntry, ProviderSessionInventoryIssue, ProviderSessionInventoryItem,
    ProviderSessionInventorySelector, ProviderSessionInventorySnapshot,
    ProviderSessionProjectCwdResolver, ProviderSessionProjectCwdSelector,
};
pub use runtime_host::AgentsCodeEngineHost;
pub use sdkwork_agent_kernel::{
    AgentConfigurationProfile, AgentConfigurationStore, AgentModelConfigurationApplication,
    AgentModelConfigurationRequest, AgentModelSelectionRequest, InMemoryAgentConfigurationStore,
    KernelError, KernelResult, ModelDescriptor, ModelResponseFormat, ModelStreamChunk,
    ModelStreamSink, ToolCall,
};
pub use sessions::*;
pub use turn::{
    cancel_code_engine_turn, code_engine_model_request_id, execute_code_engine_turn,
    execute_code_engine_turn_with_stream, execute_code_engine_turn_with_stream_sink,
    CodeEngineTurnCancellation, CodeEngineTurnInput, CodeEngineTurnOutput,
    CodeEngineTurnStreamCompletion, MAX_CODE_ENGINE_MODEL_REQUEST_ID_BYTES,
};
