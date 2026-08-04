//! Canonical durable id scheme for provider-imported Sessions, items, and
//! runtime bindings.
//!
//! Provider-imported identities carry the engine key directly as the first
//! segment after the kind marker, so an id already names its engine:
//!
//! - `session.{engine}.{stable}` (e.g. `session.codex.abc123`)
//! - `item.{engine}.{stable}` (e.g. `item.codex.abc123`)
//! - `runtime_binding.{engine}.{stable}` (e.g. `runtime_binding.codex.abc123`)
//!
//! There is exactly one active scheme: the canonical form above. Older
//! schemes carried a redundant marker between the kind and the engine
//! (`session.provider.codex.*` / `session.native.codex.*`); those prefixes
//! are recognized only by `is_legacy_provider_session_id`, the single
//! migration detector used by the retirement path, so persisted identities
//! from the old schemes are retired and re-imported under the canonical
//! scheme instead of being stranded.

use crate::validation::{ID_PREFIX_ITEM, ID_PREFIX_RUNTIME_BINDING, ID_PREFIX_SESSION};

/// Canonical provider-import Session id: `session.{engine}.{stable}`.
pub(crate) fn canonical_provider_session_id(engine_key: &str, stable_key: &str) -> String {
    format!("{ID_PREFIX_SESSION}{engine_key}.{stable_key}")
}

/// Canonical provider-import Session item id: `item.{engine}.{stable}`.
pub(crate) fn canonical_provider_item_id(engine_key: &str, stable_key: &str) -> String {
    format!("{ID_PREFIX_ITEM}{engine_key}.{stable_key}")
}

/// Canonical provider-import runtime binding id:
/// `runtime_binding.{engine}.{stable}`.
pub(crate) fn canonical_provider_runtime_binding_id(engine_key: &str, stable_key: &str) -> String {
    format!("{ID_PREFIX_RUNTIME_BINDING}{engine_key}.{stable_key}")
}

/// Canonical provider-import item id prefix for a given engine:
/// `item.{engine}.`.
pub(crate) fn canonical_provider_item_id_prefix(engine_key: &str) -> String {
    format!("{ID_PREFIX_ITEM}{engine_key}.")
}

fn is_canonical_id(kind: &str, engine_key: &str, id: &str) -> bool {
    // `{kind}.{engine_key}.` without allocating a prefix String.
    id.strip_prefix(kind)
        .and_then(|rest| rest.strip_prefix('.'))
        .and_then(|rest| rest.strip_prefix(engine_key))
        .is_some_and(|rest| rest.starts_with('.'))
}

/// Whether the id belongs to a provider-imported Session under any known
/// scheme: the canonical `session.{engine}.` form or a legacy
/// `session.provider.` / `session.native.` prefix.
pub(crate) fn is_provider_session_id(id: &str) -> bool {
    sdkwork_agents_runtime_facade::bootstrappable_engine_keys()
        .iter()
        .any(|engine_key| is_canonical_id("session", engine_key, id))
        || is_legacy_provider_session_id(id)
}

/// Whether the id belongs to a provider-imported Session created under a
/// pre-canonical scheme: `session.provider.{engine}.` or
/// `session.native.{engine}.`. This is the single migration detector: only
/// the retirement path consults it, so legacy Sessions are retired and
/// re-imported under the canonical scheme instead of being reconciled in
/// place.
pub(crate) fn is_legacy_provider_session_id(id: &str) -> bool {
    id.starts_with("session.provider.") || id.starts_with("session.native.")
}

/// Whether the id is a provider-imported Session belonging to the given
/// engine under the canonical scheme: `session.{engine}.`. Legacy-scheme
/// Sessions are not reconciled through the provider-history gates; they are
/// retired by the migration path on the next provider inventory pass.
pub(crate) fn is_provider_session_id_for(id: &str, engine_key: &str) -> bool {
    is_canonical_id("session", engine_key, id)
}

/// Whether the id is a provider-imported Session item belonging to the given
/// engine under the canonical scheme: `item.{engine}.`.
pub(crate) fn is_provider_item_id_for(id: &str, engine_key: &str) -> bool {
    is_canonical_id("item", engine_key, id)
}

/// Whether the id is a provider-import runtime binding belonging to the given
/// engine under the canonical scheme: `runtime_binding.{engine}.`.
pub(crate) fn is_provider_runtime_binding_id_for(id: &str, engine_key: &str) -> bool {
    is_canonical_id("runtime_binding", engine_key, id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_provider_ids_name_the_engine() {
        assert_eq!(
            canonical_provider_session_id("codex", "abc"),
            "session.codex.abc"
        );
        assert_eq!(canonical_provider_item_id("codex", "abc"), "item.codex.abc");
        assert_eq!(
            canonical_provider_runtime_binding_id("codex", "abc"),
            "runtime_binding.codex.abc"
        );
        assert_eq!(canonical_provider_item_id_prefix("codex"), "item.codex.");
        assert_eq!(
            canonical_provider_session_id("claude-code", "abc"),
            "session.claude-code.abc"
        );
    }

    #[test]
    fn classifies_provider_session_ids_across_schemes() {
        assert!(is_provider_session_id("session.codex.abc"));
        assert!(is_provider_session_id("session.claude-code.abc"));
        assert!(is_provider_session_id("session.opencode.abc"));
        // Legacy schemes are still detected for migration retirement.
        assert!(is_provider_session_id("session.provider.codex.abc"));
        assert!(is_provider_session_id("session.native.codex.abc"));
        assert!(!is_provider_session_id("session.12345"));
        assert!(!is_provider_session_id("session.test.abc"));
        assert!(!is_provider_session_id("session.test.facade.parent"));
        assert!(!is_provider_session_id("session.test.subagent.abc"));
        assert!(!is_provider_session_id("item.codex.abc"));
        assert!(!is_provider_session_id("runtime_binding.codex.abc"));
    }

    #[test]
    fn detects_legacy_provider_session_ids_only() {
        assert!(is_legacy_provider_session_id("session.provider.codex.abc"));
        assert!(is_legacy_provider_session_id("session.native.codex.abc"));
        assert!(!is_legacy_provider_session_id("session.codex.abc"));
        assert!(!is_legacy_provider_session_id("session.12345"));
        assert!(!is_legacy_provider_session_id("session.test.abc"));
        assert!(!is_legacy_provider_session_id("item.provider.codex.abc"));
    }

    #[test]
    fn provider_history_gates_accept_only_canonical_ids() {
        assert!(is_provider_session_id_for("session.codex.abc", "codex"));
        assert!(!is_provider_session_id_for(
            "session.provider.codex.abc",
            "codex"
        ));
        assert!(!is_provider_session_id_for(
            "session.native.codex.abc",
            "codex"
        ));
        assert!(!is_provider_session_id_for("session.codex.abc", "gemini"));
        assert!(!is_provider_session_id_for("session.gemini.abc", "codex"));
        assert!(!is_provider_session_id_for("session.12345", "codex"));
        assert!(is_provider_item_id_for("item.codex.abc", "codex"));
        assert!(!is_provider_item_id_for("item.provider.codex.abc", "codex"));
        assert!(!is_provider_item_id_for("item.12345", "codex"));
        assert!(!is_provider_item_id_for("item.gemini.abc", "codex"));
        assert!(!is_provider_item_id_for("session.codex.abc", "codex"));
        assert!(is_provider_runtime_binding_id_for(
            "runtime_binding.codex.abc",
            "codex"
        ));
        assert!(!is_provider_runtime_binding_id_for(
            "runtime_binding.provider.codex.abc",
            "codex"
        ));
        assert!(!is_provider_runtime_binding_id_for(
            "runtime_binding.12345",
            "codex"
        ));
        assert!(!is_provider_runtime_binding_id_for(
            "runtime_binding.gemini.abc",
            "codex"
        ));
    }
}
