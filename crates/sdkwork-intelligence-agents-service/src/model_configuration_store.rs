//! SQLite-backed agent configuration profile store.
//!
//! The model configuration runtime persists applied profiles through the
//! kernel [`AgentConfigurationStore`] SPI. The production store keeps
//! profiles in a local SQLite database (the single-source baseline DDL under
//! `sql/0001_agent_model_configuration_baseline.sql`), so applied model
//! configurations survive process restarts. The store is synchronous
//! (matching the kernel SPI) and holds one connection guarded by the trait's
//! `&mut self` access.
//!
//! Profiles are serialized in the kernel's `agent_configuration_profile`
//! manifest shape and parsed back through [`AgentConfigurationProfile::from_json`],
//! so the kernel schema validation stays the single source of truth.

use rusqlite::{params, Connection, OptionalExtension};
use sdkwork_agent_kernel::{
    AgentConfigurationChange, AgentConfigurationProfile, AgentConfigurationStore,
    AgentConfigurationStoreRecord, AgentConfigurationSubscriber, AgentConfigValue,
    AgentProfileArchiveRequest, AgentSecretBindingKind, AgentConfigurationUpgradePlan,
    ConfigurationSubscription, KernelError, KernelResult,
};
use std::path::Path;
use std::sync::{Arc, Mutex, RwLock};

/// Authoritative local DDL (single source of truth with
/// `sql/0001_agent_model_configuration_baseline.sql`).
const AGENT_MODEL_CONFIGURATION_SCHEMA_SQL: &str = include_str!(
    "../sql/0001_agent_model_configuration_baseline.sql"
);

/// SQLite-backed [`AgentConfigurationStore`] for the model configuration
/// runtime profiles. The connection is guarded by a mutex so the store is
/// `Send + Sync` like the kernel SPI requires.
pub struct SqliteAgentConfigurationStore {
    connection: Mutex<Connection>,
    subscribers: Arc<RwLock<Vec<(String, AgentConfigurationSubscriber)>>>,
    next_subscription_id: std::sync::atomic::AtomicU64,
}

impl std::fmt::Debug for SqliteAgentConfigurationStore {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SqliteAgentConfigurationStore")
            .field("subscriber_count", &self.subscriber_count())
            .finish()
    }
}

impl SqliteAgentConfigurationStore {
    /// Opens (creating when absent) a file-backed store at `path`.
    pub fn new(path: impl AsRef<Path>) -> KernelResult<Self> {
        let connection = Connection::open(path).map_err(|error| {
            store_error("open", format!("SQLite profile store could not be opened: {error}"))
        })?;
        Self::with_connection(connection)
    }

    /// Opens an in-memory store (tests and explicitly ephemeral runtimes).
    pub fn in_memory() -> KernelResult<Self> {
        Self::with_connection(Connection::open_in_memory().map_err(|error| {
            store_error("open_memory", format!("in-memory profile store could not be opened: {error}"))
        })?)
    }

    fn with_connection(connection: Connection) -> KernelResult<Self> {
        let store = Self {
            connection: Mutex::new(connection),
            subscribers: Arc::new(RwLock::new(Vec::new())),
            next_subscription_id: std::sync::atomic::AtomicU64::new(0),
        };
        store.initialize_schema()?;
        Ok(store)
    }

    fn subscriber_count(&self) -> usize {
        self.subscribers
            .read()
            .map(|subscribers| subscribers.len())
            .unwrap_or(0)
    }

    fn initialize_schema(&self) -> KernelResult<()> {
        let connection = self.connection.lock().map_err(lock_error)?;
        connection
            .execute_batch(AGENT_MODEL_CONFIGURATION_SCHEMA_SQL)
            .map_err(|error| {
                store_error("schema", format!("profile store schema could not be applied: {error}"))
            })
    }

    fn upsert(&self, profile: AgentConfigurationProfile) -> KernelResult<()> {
        let configuration_json = configuration_to_json(&profile).to_string();
        let secret_bindings_json = secret_bindings_to_json(&profile).to_string();
        let now = now_rfc3339();
        let connection = self.connection.lock().map_err(lock_error)?;
        let updated = connection
            .execute(
                "UPDATE agent_model_configuration_profile
                 SET configuration_version = ?1, status = ?2,
                     configuration_json = ?3, secret_bindings_json = ?4,
                     updated_at = ?5, version = version + 1
                 WHERE profile_id = ?6 AND agent_id = ?7",
                params![
                    profile.configuration_version,
                    profile.status.as_str(),
                    configuration_json,
                    secret_bindings_json,
                    now,
                    profile.profile_id,
                    profile.agent_id,
                ],
            )
            .map_err(|error| store_error("update", error.to_string()))?;
        if updated == 0 {
            connection
                .execute(
                    "INSERT INTO agent_model_configuration_profile
                     (profile_id, agent_id, configuration_version, status,
                      configuration_json, secret_bindings_json, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
                    params![
                        profile.profile_id,
                        profile.agent_id,
                        profile.configuration_version,
                        profile.status.as_str(),
                        configuration_json,
                        secret_bindings_json,
                        now,
                    ],
                )
                .map_err(|error| store_error("insert", error.to_string()))?;
        }
        Ok(())
    }

    fn notify(&self, record: &AgentConfigurationStoreRecord, change: AgentConfigurationChange) {
        if let Ok(subscribers) = self.subscribers.read() {
            for (_, subscriber) in subscribers.iter() {
                subscriber(record, change);
            }
        }
    }
}

impl AgentConfigurationStore for SqliteAgentConfigurationStore {
    fn save_profile(
        &mut self,
        profile: AgentConfigurationProfile,
    ) -> KernelResult<AgentConfigurationStoreRecord> {
        self.upsert(profile.clone())?;
        let record = AgentConfigurationStoreRecord::created(profile);
        self.notify(&record, AgentConfigurationChange::Saved);
        Ok(record)
    }

    fn load_profile(
        &self,
        agent_id: &str,
        profile_id: &str,
    ) -> KernelResult<AgentConfigurationProfile> {
        self.find_profile(agent_id, profile_id)?
            .ok_or_else(|| {
                KernelError::validation(format!(
                    "agent configuration profile not found: {agent_id}/{profile_id}"
                ))
            })
    }

    fn list_profiles(&self, agent_id: &str) -> KernelResult<Vec<AgentConfigurationProfile>> {
        let connection = self.connection.lock().map_err(lock_error)?;
        let mut statement = connection
            .prepare(
                "SELECT profile_id, agent_id, configuration_version, status,
                        configuration_json, secret_bindings_json
                 FROM agent_model_configuration_profile
                 WHERE agent_id = ?1
                 ORDER BY profile_id",
            )
            .map_err(|error| store_error("list_prepare", error.to_string()))?;
        let rows = statement
            .query_map(params![agent_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                ))
            })
            .map_err(|error| store_error("list_query", error.to_string()))?;
        let mut profiles = Vec::new();
        for row in rows {
            let (profile_id, agent_id, configuration_version, status, configuration_json, secret_bindings_json) =
                row.map_err(|error| store_error("list_row", error.to_string()))?;
            profiles.push(profile_from_columns(
                profile_id,
                agent_id,
                configuration_version,
                status,
                &configuration_json,
                &secret_bindings_json,
            )?);
        }
        Ok(profiles)
    }

    fn find_profile(
        &self,
        agent_id: &str,
        profile_id: &str,
    ) -> KernelResult<Option<AgentConfigurationProfile>> {
        let connection = self.connection.lock().map_err(lock_error)?;
        let row = connection
            .query_row(
                "SELECT profile_id, agent_id, configuration_version, status,
                        configuration_json, secret_bindings_json
                 FROM agent_model_configuration_profile
                 WHERE profile_id = ?1 AND agent_id = ?2",
                params![profile_id, agent_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, String>(5)?,
                    ))
                },
            )
            .optional()
            .map_err(|error| store_error("find_query", error.to_string()))?;
        row.map(
            |(profile_id, agent_id, configuration_version, status, configuration_json, secret_bindings_json)| {
                profile_from_columns(
                    profile_id,
                    agent_id,
                    configuration_version,
                    status,
                    &configuration_json,
                    &secret_bindings_json,
                )
            },
        )
        .transpose()
    }

    fn migrate_profile(
        &mut self,
        plan: &AgentConfigurationUpgradePlan,
        current_profile: AgentConfigurationProfile,
    ) -> KernelResult<AgentConfigurationStoreRecord> {
        self.upsert(current_profile.clone())?;
        let record = AgentConfigurationStoreRecord::migrated(current_profile, &plan.plan_id);
        self.notify(&record, AgentConfigurationChange::Migrated);
        Ok(record)
    }

    fn archive_profile(
        &mut self,
        request: &AgentProfileArchiveRequest,
    ) -> KernelResult<AgentConfigurationStoreRecord> {
        let profile = self
            .load_profile(&request.agent_id, &request.profile_id)?
            .archive();
        self.upsert(profile.clone())?;
        let record = AgentConfigurationStoreRecord::archived(profile, &request.request_id);
        self.notify(&record, AgentConfigurationChange::Archived);
        Ok(record)
    }

    fn subscribe(&mut self, subscriber: AgentConfigurationSubscriber) -> ConfigurationSubscription {
        let subscription_id = format!(
            "subscription.{}",
            self.next_subscription_id
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        );
        if let Ok(mut subscribers) = self.subscribers.write() {
            subscribers.push((subscription_id.clone(), subscriber));
        }
        ConfigurationSubscription::registered(subscription_id, self.subscribers.clone())
    }
}

fn store_error(context: impl AsRef<str>, message: impl AsRef<str>) -> KernelError {
    KernelError::provider_error(
        format!("agent_configuration_store.{}", context.as_ref()),
        message.as_ref(),
    )
}

fn lock_error(
    _error: std::sync::PoisonError<std::sync::MutexGuard<'_, Connection>>,
) -> KernelError {
    store_error("lock", "profile store connection lock is poisoned")
}

fn now_rfc3339() -> String {
    time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

/// Serializes the profile `configuration` entries in the kernel manifest
/// shape (`{"entries": [...]}`) so [`AgentConfigurationProfile::from_json`]
/// can parse it back.
fn configuration_to_json(profile: &AgentConfigurationProfile) -> serde_json::Value {
    let entries = profile
        .configuration
        .entries
        .iter()
        .map(|entry| {
            serde_json::json!({
                "key": entry.key,
                "value_kind": value_kind_str(&entry.value),
                "value": value_to_json(&entry.value),
            })
        })
        .collect::<Vec<_>>();
    serde_json::json!({ "entries": entries })
}

/// Serializes the profile secret bindings in the kernel manifest shape.
fn secret_bindings_to_json(profile: &AgentConfigurationProfile) -> serde_json::Value {
    let bindings = profile
        .secret_bindings
        .iter()
        .map(|binding| {
            let mut value = serde_json::json!({
                "field_key": binding.field_key,
                "kind": binding_kind_str(binding.binding_kind),
                "secret_ref": binding.secret_ref,
            });
            if let Some(provider_hint) = &binding.provider_hint {
                value["provider_hint"] = serde_json::Value::String(provider_hint.clone());
            }
            value
        })
        .collect::<Vec<_>>();
    serde_json::Value::Array(bindings)
}

/// Reconstructs a profile from its column values through the kernel's
/// validated manifest parser.
fn profile_from_columns(
    profile_id: String,
    agent_id: String,
    configuration_version: String,
    status: String,
    configuration_json: &str,
    secret_bindings_json: &str,
) -> KernelResult<AgentConfigurationProfile> {
    let configuration = serde_json::from_str::<serde_json::Value>(configuration_json).map_err(
        |error| {
            store_error(
                "parse_configuration",
                format!("stored configuration is not valid JSON: {error}"),
            )
        },
    )?;
    let secret_bindings =
        serde_json::from_str::<serde_json::Value>(secret_bindings_json).map_err(|error| {
            store_error(
                "parse_bindings",
                format!("stored secret bindings are not valid JSON: {error}"),
            )
        })?;
    let document = serde_json::json!({
        "manifest_type": "agent_configuration_profile",
        "profile_id": profile_id,
        "agent_id": agent_id,
        "configuration_version": configuration_version,
        "status": status,
        "configuration": configuration,
        "secret_bindings": secret_bindings,
    });
    AgentConfigurationProfile::from_json(&document.to_string())
}

fn value_kind_str(value: &AgentConfigValue) -> &'static str {
    match value {
        AgentConfigValue::String(_) => "string",
        AgentConfigValue::Boolean(_) => "boolean",
        AgentConfigValue::Integer(_) => "integer",
        AgentConfigValue::SecretRef(_) => "secret_ref",
        AgentConfigValue::StringList(_) => "string_list",
        AgentConfigValue::Json(_) => "json",
    }
}

fn value_to_json(value: &AgentConfigValue) -> serde_json::Value {
    match value {
        AgentConfigValue::String(value) => serde_json::Value::String(value.clone()),
        AgentConfigValue::Boolean(value) => serde_json::Value::Bool(*value),
        AgentConfigValue::Integer(value) => serde_json::Value::Number((*value).into()),
        AgentConfigValue::SecretRef(value) => serde_json::Value::String(value.clone()),
        AgentConfigValue::StringList(values) => {
            serde_json::Value::Array(values.iter().cloned().map(serde_json::Value::String).collect())
        }
        AgentConfigValue::Json(value) => serde_json::from_str(value).unwrap_or_else(|_| {
            serde_json::Value::String(value.clone())
        }),
    }
}

fn binding_kind_str(kind: AgentSecretBindingKind) -> &'static str {
    match kind {
        AgentSecretBindingKind::LoginPassword => "login_password",
        AgentSecretBindingKind::LoginToken => "login_token",
        AgentSecretBindingKind::OAuthCredential => "oauth_credential",
        AgentSecretBindingKind::LlmApiKey => "llm_api_key",
        AgentSecretBindingKind::CustomSecret => "custom_secret",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sdkwork_agent_kernel::{
        AgentConfiguration, AgentConfigurationProfileStatus, AgentSecretBinding,
    };

    fn sample_profile(agent_id: &str, profile_id: &str) -> AgentConfigurationProfile {
        AgentConfigurationProfile::new(
            profile_id,
            agent_id,
            "0.2.0",
            AgentConfiguration::new(agent_id, profile_id)
                .set("codex.model.base_url", AgentConfigValue::string("https://api.birdcoder.com/v1"))
                .set("codex.model.default", AgentConfigValue::string("gpt-5.4"))
                .set(
                    "codex.model.api_key",
                    AgentConfigValue::secret_ref("secret.model.key-1"),
                )
                .set(
                    "codex.model.supported",
                    AgentConfigValue::string_list(vec!["gpt-5.4".to_string()]),
                )
                .set("codex.model.supports_multimodal", AgentConfigValue::boolean(false)),
        )
        .add_secret_binding(AgentSecretBinding::llm_api_key(
            "codex.model.api_key",
            "openai",
            "secret.model.key-1",
        ))
        .activate()
    }

    #[test]
    fn save_and_load_round_trip_preserves_profile() {
        let mut store = SqliteAgentConfigurationStore::in_memory().expect("store");
        let profile = sample_profile("agent.codex", "profile.test");
        store.save_profile(profile.clone()).expect("save");

        let loaded = store
            .load_profile("agent.codex", "profile.test")
            .expect("load");
        assert_eq!(loaded, profile);
        assert_eq!(loaded.status, AgentConfigurationProfileStatus::Active);
        assert_eq!(
            loaded
                .configuration
                .value("codex.model.base_url"),
            Some(&AgentConfigValue::String(
                "https://api.birdcoder.com/v1".to_string()
            ))
        );
        assert_eq!(loaded.secret_bindings.len(), 1);
    }

    #[test]
    fn update_overwrites_and_bumps_version() {
        let mut store = SqliteAgentConfigurationStore::in_memory().expect("store");
        let profile = sample_profile("agent.codex", "profile.test");
        store.save_profile(profile.clone()).expect("first save");

        let updated = AgentConfigurationProfile::new(
            "profile.test",
            "agent.codex",
            "0.3.0",
            AgentConfiguration::new("agent.codex", "profile.test")
                .set("codex.model.base_url", AgentConfigValue::string("https://relay.new/v1")),
        );
        store.save_profile(updated.clone()).expect("second save");
        let loaded = store
            .load_profile("agent.codex", "profile.test")
            .expect("load");
        assert_eq!(loaded.configuration_version, "0.3.0");
        assert_eq!(
            loaded.configuration.value("codex.model.base_url"),
            Some(&AgentConfigValue::String("https://relay.new/v1".to_string()))
        );
    }

    #[test]
    fn list_and_find_scope_by_agent() {
        let mut store = SqliteAgentConfigurationStore::in_memory().expect("store");
        store
            .save_profile(sample_profile("agent.codex", "profile.a"))
            .expect("save a");
        store
            .save_profile(sample_profile("agent.codex", "profile.b"))
            .expect("save b");
        store
            .save_profile(sample_profile("agent.rig-general", "profile.c"))
            .expect("save c");

        let profiles = store.list_profiles("agent.codex").expect("list");
        assert_eq!(profiles.len(), 2);
        assert_eq!(profiles[0].profile_id, "profile.a");
        assert_eq!(profiles[1].profile_id, "profile.b");
        assert!(
            store
                .find_profile("agent.rig-general", "profile.c")
                .expect("find")
                .is_some()
        );
        assert!(
            store
                .find_profile("agent.rig-general", "profile.missing")
                .expect("find")
                .is_none()
        );
    }

    #[test]
    fn archive_changes_status_and_notifies() {
        let mut store = SqliteAgentConfigurationStore::in_memory().expect("store");
        store
            .save_profile(sample_profile("agent.codex", "profile.test"))
            .expect("save");
        let notified = Arc::new(std::sync::Mutex::new(Vec::new()));
        {
            let notified = notified.clone();
            store.subscribe(Box::new(move |record, change| {
                notified
                    .lock()
                    .unwrap()
                    .push((record.profile.profile_id.clone(), change));
            }));
        }

        let record = store
            .archive_profile(
                &AgentProfileArchiveRequest::new("request-1", "agent.codex", "profile.test"),
            )
            .expect("archive");
        assert_eq!(
            record.profile.status,
            AgentConfigurationProfileStatus::Archived
        );
        let loaded = store
            .load_profile("agent.codex", "profile.test")
            .expect("load");
        assert_eq!(loaded.status, AgentConfigurationProfileStatus::Archived);
        assert_eq!(
            *notified.lock().unwrap(),
            vec![(
                "profile.test".to_string(),
                AgentConfigurationChange::Archived
            )]
        );
    }

    #[test]
    fn optimistic_save_conflicts_on_stale_version() {
        let mut store = SqliteAgentConfigurationStore::in_memory().expect("store");
        let profile = sample_profile("agent.codex", "profile.test");
        store.save_profile(profile.clone()).expect("save");

        let stale = AgentConfigurationProfile::new(
            "profile.test",
            "agent.codex",
            "0.1.0",
            AgentConfiguration::new("agent.codex", "profile.test"),
        );
        let result = store.save_profile_if_version(stale, "0.1.0");
        assert!(result.is_err(), "stale version must conflict");
        assert!(matches!(
            result.unwrap_err().kind(),
            sdkwork_agent_kernel::KernelErrorKind::Conflict
        ));
    }
}
