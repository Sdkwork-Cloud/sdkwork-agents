//! Typed error for the agents runtime facade.
//!
//! Replaces the previous `String` error type with structured variants
//! that callers can match on for differentiated error handling (e.g.
//! retry on kernel errors, reject on validation errors).

use thiserror::Error;

/// Typed error returned by all runtime-facade operations.
///
/// # Variants
///
/// - [`UnsupportedEngine`](Self::UnsupportedEngine) — the engine key is
///   not registered in the host or not recognised as canonical.
/// - [`UnsupportedCapability`](Self::UnsupportedCapability) — the engine is
///   registered but does not expose the requested capability.
/// - [`EngineMismatch`](Self::EngineMismatch) — the engine key in the
///   turn input does not match the slot's engine key.
/// - [`BlankPrompt`](Self::BlankPrompt) — the prompt is empty or
///   whitespace-only.
/// - [`UnsupportedLiveInteraction`](Self::UnsupportedLiveInteraction) —
///   the engine does not have a registered live-interaction handler for
///   the requested interaction type (approval or user-question).
/// - [`Kernel`](Self::Kernel) — a kernel provider error occurred during
///   model invocation.
/// - [`Handler`](Self::Handler) — a handler-specific error from a
///   live-interaction handler implementation.
#[derive(Debug, Error)]
pub enum RuntimeFacadeError {
    #[error("invalid facade input: {0}")]
    InvalidInput(String),
    /// The requested engine key is not supported or not registered.
    #[error("unsupported engineId \"{engine_key}\"")]
    UnsupportedEngine { engine_key: String },

    /// The engine is registered but does not expose the requested capability.
    #[error("engineId \"{engine_key}\" does not support capability \"{capability_id}\"")]
    UnsupportedCapability {
        engine_key: String,
        capability_id: String,
    },

    /// The requested engine was selected for this host but failed to bootstrap.
    #[error("engineId \"{engine_key}\" is unavailable: {reason}")]
    EngineUnavailable { engine_key: String, reason: String },

    /// The engine key in the turn input does not match the slot's engine key.
    #[error("engine mismatch: slot={slot_engine} input={input_engine}")]
    EngineMismatch {
        slot_engine: String,
        input_engine: String,
    },

    /// The prompt is blank or empty.
    #[error("prompt must not be blank")]
    BlankPrompt,

    /// The engine does not support live interaction replies.
    #[error("engineId \"{engine_key}\" does not support live {interaction_type} replies through agents runtime facade yet")]
    UnsupportedLiveInteraction {
        engine_key: String,
        interaction_type: &'static str,
    },

    /// A kernel provider error occurred during model invocation.
    #[error("kernel error: {0}")]
    Kernel(String),

    /// A handler-specific error from a live interaction handler.
    #[error("{0}")]
    Handler(String),
}

/// Convenience result alias used throughout the runtime facade.
pub type RuntimeFacadeResult<T> = Result<T, RuntimeFacadeError>;
