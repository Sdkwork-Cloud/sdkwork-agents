use std::collections::{HashMap, HashSet};

use sdkwork_agent_kernel::{AgentMessage, AgentSession, SessionKind};
use sdkwork_agent_provider_claude_code::{
    discover_claude_code_provider_session_messages, discover_claude_code_provider_sessions,
};
use sdkwork_agent_provider_codex::{
    discover_codex_provider_session_messages, discover_codex_provider_sessions,
};
use sdkwork_agent_provider_core::{
    normalize_provider_session_path, provider_session_directory_fingerprint,
    provider_session_path_basename,
};
use sdkwork_agent_provider_opencode::{
    discover_opencode_provider_session_messages, discover_opencode_provider_sessions,
};

use crate::code_engines::CodeEngineSlot;
use crate::error::{RuntimeFacadeError, RuntimeFacadeResult};

/// Prevent a corrupt provider history directory from retaining an unbounded
/// cross-provider inventory in the runtime process. Callers receive an
/// explicit failure and can narrow the project selector before retrying.
const MAX_PROVIDER_SESSION_INVENTORY_ITEMS: usize = 10_000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderSessionProjectCwdSelector {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub owner_user_id: u64,
    pub project_id: String,
    pub project_name: String,
}

pub trait ProviderSessionProjectCwdResolver: Send + Sync {
    fn resolve_project_cwd(
        &self,
        selector: &ProviderSessionProjectCwdSelector,
    ) -> RuntimeFacadeResult<Option<String>>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderSessionInventorySelector {
    pub directory_fingerprint: Option<String>,
    pub exact_cwd: Option<String>,
    pub unique_basename: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderSessionInventoryItem {
    pub engine_key: String,
    pub agent_id: String,
    pub binding_id: String,
    pub provider_id: String,
    pub default_model_id: String,
    pub session: AgentSession,
}

pub(crate) fn discover_provider_sessions(
    slots: &HashMap<String, CodeEngineSlot>,
    selector: &ProviderSessionInventorySelector,
) -> RuntimeFacadeResult<Vec<ProviderSessionInventoryItem>> {
    let mut candidates = Vec::new();
    for engine_key in ["codex", "claude-code", "opencode"] {
        let Some(slot) = slots.get(engine_key) else {
            continue;
        };
        let sessions = match engine_key {
            "codex" => discover_codex_provider_sessions(),
            "claude-code" => discover_claude_code_provider_sessions(),
            "opencode" => discover_opencode_provider_sessions(),
            _ => unreachable!(),
        }
        .map_err(|error| RuntimeFacadeError::EngineUnavailable {
            engine_key: engine_key.to_string(),
            reason: error.to_string(),
        })?;
        let descriptors = slot.list_model_descriptors();
        let Some(default_model) = descriptors.first() else {
            continue;
        };
        let Some(agent_id) = crate::code_engines::code_engine_agent_id(engine_key) else {
            continue;
        };
        for session in sessions {
            if candidates.len() >= MAX_PROVIDER_SESSION_INVENTORY_ITEMS {
                return Err(RuntimeFacadeError::InvalidInput(format!(
                    "provider session inventory exceeds {MAX_PROVIDER_SESSION_INVENTORY_ITEMS} items"
                )));
            }
            candidates.push(ProviderSessionInventoryItem {
                engine_key: engine_key.to_string(),
                agent_id: agent_id.to_string(),
                binding_id: slot.binding_id().to_string(),
                provider_id: default_model.provider_id.clone(),
                default_model_id: default_model.model_id.clone(),
                session,
            });
        }
    }

    let selected_cwd = resolve_selected_cwd(&candidates, selector)?;
    Ok(select_top_level_provider_sessions(
        candidates,
        selected_cwd.as_deref(),
    ))
}

fn select_top_level_provider_sessions(
    candidates: Vec<ProviderSessionInventoryItem>,
    selected_cwd: Option<&str>,
) -> Vec<ProviderSessionInventoryItem> {
    let mut dedupe = HashSet::new();
    let mut selected = candidates
        .into_iter()
        .filter(|item| {
            item.session
                .cwd
                .as_deref()
                .map(normalize_provider_session_path)
                .as_deref()
                == selected_cwd
        })
        .filter(|item| {
            item.session.kind != SessionKind::Subagent && item.session.parent_session_id.is_none()
        })
        .filter(|item| {
            dedupe.insert((
                item.binding_id.trim().to_string(),
                item.provider_id.trim().to_string(),
                item.session.session_id.trim().to_string(),
            ))
        })
        .collect::<Vec<_>>();
    selected.sort_by(|left, right| {
        right
            .session
            .updated_at
            .as_deref()
            .unwrap_or_default()
            .cmp(left.session.updated_at.as_deref().unwrap_or_default())
            .then_with(|| left.engine_key.cmp(&right.engine_key))
            .then_with(|| left.session.session_id.cmp(&right.session.session_id))
    });
    selected
}

pub(crate) fn load_provider_session_messages(
    slots: &HashMap<String, CodeEngineSlot>,
    engine_key: &str,
    provider_session_id: &str,
) -> RuntimeFacadeResult<Vec<AgentMessage>> {
    if !slots.contains_key(engine_key) {
        return Err(RuntimeFacadeError::UnsupportedEngine {
            engine_key: engine_key.to_string(),
        });
    }
    if provider_session_id.trim().is_empty() {
        return Err(RuntimeFacadeError::InvalidInput(
            "provider session id is required to load transcript messages".to_string(),
        ));
    }
    match engine_key {
        "codex" => discover_codex_provider_session_messages(provider_session_id),
        "claude-code" => discover_claude_code_provider_session_messages(provider_session_id),
        "opencode" => discover_opencode_provider_session_messages(provider_session_id),
        _ => {
            return Err(RuntimeFacadeError::UnsupportedCapability {
                engine_key: engine_key.to_string(),
                capability_id: "sdk.session.history".to_string(),
            })
        }
    }
    .map_err(|error| RuntimeFacadeError::EngineUnavailable {
        engine_key: engine_key.to_string(),
        reason: error.to_string(),
    })
}

fn resolve_selected_cwd(
    candidates: &[ProviderSessionInventoryItem],
    selector: &ProviderSessionInventorySelector,
) -> RuntimeFacadeResult<Option<String>> {
    if let Some(cwd) = selector
        .exact_cwd
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        return Ok(Some(normalize_provider_session_path(cwd)));
    }
    let Some(basename) = selector
        .unique_basename
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Err(RuntimeFacadeError::InvalidInput(
            "provider session inventory requires an exact cwd or unique basename".to_string(),
        ));
    };
    let basename = basename.to_ascii_lowercase();
    if let Some(directory_fingerprint) = selector
        .directory_fingerprint
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let matching_paths = candidates
            .iter()
            .filter_map(|item| item.session.cwd.as_deref())
            .filter(|cwd| provider_session_path_basename(cwd).as_deref() == Some(basename.as_str()))
            .filter(|cwd| {
                provider_session_directory_fingerprint(cwd).ok().as_deref()
                    == Some(directory_fingerprint)
            })
            .map(normalize_provider_session_path)
            .collect::<HashSet<_>>();
        return match matching_paths.len() {
            0 => Ok(None),
            1 => Ok(matching_paths.into_iter().next()),
            _ => Err(RuntimeFacadeError::InvalidInput(format!(
                "provider session directory fingerprint is ambiguous: {basename}"
            ))),
        };
    }
    let matching_paths = candidates
        .iter()
        .filter_map(|item| item.session.cwd.as_deref())
        .filter(|cwd| provider_session_path_basename(cwd).as_deref() == Some(basename.as_str()))
        .map(normalize_provider_session_path)
        .collect::<HashSet<_>>();
    match matching_paths.len() {
        0 => Ok(None),
        1 => Ok(matching_paths.into_iter().next()),
        _ => Err(RuntimeFacadeError::InvalidInput(format!(
            "provider session directory name is ambiguous: {basename}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::code_engines::bootstrap_code_engine;

    fn item(engine: &str, session_id: &str, cwd: &str) -> ProviderSessionInventoryItem {
        ProviderSessionInventoryItem {
            engine_key: engine.to_string(),
            agent_id: format!("agent.intelligence.{engine}"),
            binding_id: format!("binding.agent-provider.{engine}"),
            provider_id: format!("provider.model.{engine}"),
            default_model_id: "model.default".to_string(),
            session: AgentSession::new(session_id).with_cwd(cwd),
        }
    }

    #[test]
    fn exact_windows_path_matches_extended_length_provider_cwd() {
        let candidates = vec![item("codex", "one", r"\\?\E:\Work\BirdCoder")];
        let selected = resolve_selected_cwd(
            &candidates,
            &ProviderSessionInventorySelector {
                directory_fingerprint: None,
                exact_cwd: Some("e:/work/birdcoder/".to_string()),
                unique_basename: None,
            },
        )
        .expect("selected cwd");
        assert_eq!(selected.as_deref(), Some("e:/work/birdcoder"));
    }

    #[test]
    fn basename_selection_fails_closed_when_multiple_paths_match() {
        let candidates = vec![
            item("codex", "one", "C:/one/BirdCoder"),
            item("opencode", "two", "D:/two/BirdCoder"),
        ];
        let result = resolve_selected_cwd(
            &candidates,
            &ProviderSessionInventorySelector {
                directory_fingerprint: None,
                exact_cwd: None,
                unique_basename: Some("BirdCoder".to_string()),
            },
        );
        assert!(matches!(result, Err(RuntimeFacadeError::InvalidInput(_))));
    }

    #[test]
    fn top_level_inventory_deduplicates_runtime_scoped_provider_identity_and_excludes_subagents() {
        let root = item("codex", "root-session", r"E:\Work\BirdCoder");
        let mut duplicate = item("codex", " root-session ", r"E:\Work\BirdCoder");
        duplicate.provider_id = root.provider_id.clone();
        let mut subagent = item("codex", "child-session", r"E:\Work\BirdCoder");
        subagent.session.kind = SessionKind::Subagent;
        subagent.session.parent_session_id = Some("root-session".to_string());

        let selected = select_top_level_provider_sessions(
            vec![root, duplicate, subagent],
            Some("e:/work/birdcoder"),
        );

        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].session.session_id, "root-session");
    }

    #[test]
    fn top_level_inventory_keeps_the_same_provider_session_from_distinct_runtime_bindings() {
        let root = item("codex", "shared-session", r"E:\Work\BirdCoder");
        let mut distinct_runtime = item("opencode", "shared-session", r"E:\Work\BirdCoder");
        distinct_runtime.provider_id = root.provider_id.clone();

        let selected = select_top_level_provider_sessions(
            vec![root, distinct_runtime],
            Some("e:/work/birdcoder"),
        );

        assert_eq!(selected.len(), 2);
        assert_ne!(selected[0].binding_id, selected[1].binding_id);
    }

    #[test]
    fn registered_engine_without_history_reports_missing_capability() {
        let mut slots = HashMap::new();
        slots.insert(
            "gemini".to_string(),
            bootstrap_code_engine("gemini").expect("Gemini bootstrap"),
        );

        let error = load_provider_session_messages(&slots, "gemini", "session-1")
            .expect_err("Gemini history is not implemented");
        assert!(matches!(
            error,
            RuntimeFacadeError::UnsupportedCapability {
                ref engine_key,
                ref capability_id,
            } if engine_key == "gemini" && capability_id == "sdk.session.history"
        ));
    }

    #[test]
    fn unknown_engine_still_reports_unsupported_engine() {
        let error = load_provider_session_messages(&HashMap::new(), "unknown", "session-1")
            .expect_err("unknown engine must fail");
        assert!(matches!(
            error,
            RuntimeFacadeError::UnsupportedEngine { ref engine_key }
                if engine_key == "unknown"
        ));
    }

    #[test]
    fn directory_fingerprint_selects_one_of_two_same_basename_paths() {
        let fixture_root = create_fingerprint_fixture_root("unique");
        let first = fixture_root.join("first").join("BirdCoder");
        let second = fixture_root.join("second").join("BirdCoder");
        std::fs::create_dir_all(first.join("apps")).expect("create first fixture");
        std::fs::create_dir_all(second.join("packages")).expect("create second fixture");
        let fingerprint =
            provider_session_directory_fingerprint(second.to_str().expect("second fixture path"))
                .expect("fingerprint second fixture");
        let candidates = vec![
            item("codex", "one", first.to_str().expect("first fixture path")),
            item(
                "opencode",
                "two",
                second.to_str().expect("second fixture path"),
            ),
        ];

        let selected = resolve_selected_cwd(
            &candidates,
            &ProviderSessionInventorySelector {
                directory_fingerprint: Some(fingerprint),
                exact_cwd: None,
                unique_basename: Some("BirdCoder".to_string()),
            },
        )
        .expect("select fingerprinted directory");
        let normalized_second =
            normalize_provider_session_path(second.to_str().expect("second fixture path"));
        assert_eq!(selected.as_deref(), Some(normalized_second.as_str()));
        std::fs::remove_dir_all(fixture_root).expect("remove fixtures");
    }

    #[test]
    fn directory_fingerprint_fails_closed_for_identical_roots() {
        let fixture_root = create_fingerprint_fixture_root("ambiguous");
        let first = fixture_root.join("first").join("BirdCoder");
        let second = fixture_root.join("second").join("BirdCoder");
        std::fs::create_dir_all(first.join("apps")).expect("create first fixture");
        std::fs::create_dir_all(second.join("apps")).expect("create second fixture");
        let fingerprint =
            provider_session_directory_fingerprint(first.to_str().expect("first fixture path"))
                .expect("fingerprint first fixture");
        let candidates = vec![
            item("codex", "one", first.to_str().expect("first fixture path")),
            item(
                "opencode",
                "two",
                second.to_str().expect("second fixture path"),
            ),
        ];

        let result = resolve_selected_cwd(
            &candidates,
            &ProviderSessionInventorySelector {
                directory_fingerprint: Some(fingerprint),
                exact_cwd: None,
                unique_basename: Some("BirdCoder".to_string()),
            },
        );
        assert!(matches!(result, Err(RuntimeFacadeError::InvalidInput(_))));
        std::fs::remove_dir_all(fixture_root).expect("remove fixtures");
    }

    fn create_fingerprint_fixture_root(label: &str) -> std::path::PathBuf {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("test clock")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "sdkwork-provider-session-selector-{label}-{}-{nonce}",
            std::process::id()
        ))
    }
}
