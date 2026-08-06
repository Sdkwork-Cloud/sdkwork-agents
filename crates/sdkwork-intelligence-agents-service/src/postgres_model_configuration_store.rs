//! PostgreSQL-backed agent configuration profile store.
//!
//! The model configuration runtime persists applied profiles through the
//! kernel [`AgentConfigurationStore`] SPI. The server-authoritative store
//! keeps profiles in the canonical Agents PostgreSQL database
//! (`ai_agent_model_configuration_profile`, applied through the
//! `database/ddl/baseline/postgres/` baseline), so applied model
//! configurations survive process restarts. SQLite is client-local only
//! (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only).

use sdkwork_agent_kernel::{
    AgentConfigValue, AgentConfigurationChange, AgentConfigurationProfile, AgentConfigurationStore,
    AgentConfigurationStoreRecord, AgentConfigurationSubscriber, AgentConfigurationUpgradePlan,
    AgentProfileArchiveRequest, AgentSecretBindingKind, ConfigurationSubscription, KernelError,
    KernelResult,
};
use sqlx::Row;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use crate::postgres_sync_pool::BlockingPostgresPool;
use crate::persistence::AGENTS_DATABASE_SERVICE;

/// PostgreSQL-backed [`AgentConfigurationStore`] for the model configuration
/// runtime profiles. SQL executes on the shared process pool through
/// [`BlockingPostgresPool`]; the store stays `Send + Sync` like the kernel
/// SPI requires.
pub struct PostgresAgentConfigurationStore {
    pool: BlockingPostgresPool,
    subscribers: Arc<RwLock<Vec<(String, AgentConfigurationSubscriber)>>>,
    next_subscription_id: AtomicU64,
}

impl std::fmt::Debug for PostgresAgentConfigurationStore {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PostgresAgentConfigurationStore")
            .field("subscriber_count", &self.subscriber_count())
            .finish()
    }
}

impl PostgresAgentConfigurationStore {
    /// Wraps a shared process PostgreSQL pool (agents service profile).
    pub fn from_pool(pool: BlockingPostgresPool) -> Self {
        Self {
            pool,
            subscribers: Arc::new(RwLock::new(Vec::new())),
            next_subscription_id: AtomicU64::new(0),
        }
    }

    /// Connects through the canonical `sdkwork-database-config` Agents
    /// service profile (server-authoritative PostgreSQL).
    pub fn connect_from_agents_database_env() -> KernelResult<Self> {
        Ok(Self::from_pool(
            BlockingPostgresPool::connect_from_sdkwork_env(AGENTS_DATABASE_SERVICE)?,
        ))
    }

    fn subscriber_count(&self) -> usize {
        self.subscribers
            .read()
            .map(|subscribers| subscribers.len())
            .unwrap_or(0)
    }

    fn upsert(&self, profile: AgentConfigurationProfile) -> KernelResult<()> {
        let configuration_json = configuration_to_json(&profile).to_string();
        let secret_bindings_json = secret_bindings_to_json(&profile).to_string();
        let now = now_rfc3339();
        let pool = self.pool.clone();
        let profile_id = profile.profile_id.clone();
        let agent_id = profile.agent_id.clone();
        let configuration_version = profile.configuration_version.clone();
        let status = profile.status.as_str().to_owned();
        self.pool.block_on(async move {
            let updated = sqlx::query(
                "UPDATE ai_agent_model_configuration_profile
                 SET configuration_version = $1, status = $2,
                     configuration_json = $3, secret_bindings_json = $4,
                     updated_at = $5, version = version + 1
                 WHERE profile_id = $6 AND agent_id = $7",
            )
            .bind(&configuration_version)
            .bind(&status)
            .bind(&configuration_json)
            .bind(&secret_bindings_json)
            .bind(&now)
            .bind(&profile_id)
            .bind(&agent_id)
            .execute(pool.pool())
            .await
            .map_err(|error| store_error("update", error.to_string()))?
            .rows_affected();
            if updated == 0 {
                sqlx::query(
                    "INSERT INTO ai_agent_model_configuration_profile
                     (profile_id, agent_id, configuration_version, status,
                      configuration_json, secret_bindings_json, created_at, updated_at)
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $7)",
                )
                .bind(&profile_id)
                .bind(&agent_id)
                .bind(&configuration_version)
                .bind(&status)
                .bind(&configuration_json)
                .bind(&secret_bindings_json)
                .bind(&now)
                .execute(pool.pool())
                .await
                .map_err(|error| store_error("insert", error.to_string()))?;
            }
            Ok(())
        })
    }

    fn notify(&self, record: &AgentConfigurationStoreRecord, change: AgentConfigurationChange) {
        if let Ok(subscribers) = self.subscribers.read() {
            for (_, subscriber) in subscribers.iter() {
                subscriber(record, change);
            }
        }
    }
}

impl AgentConfigurationStore for PostgresAgentConfigurationStore {
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
        self.find_profile(agent_id, profile_id)?.ok_or_else(|| {
            KernelError::validation(format!(
                "agent configuration profile not found: {agent_id}/{profile_id}"
            ))
        })
    }

    fn list_profiles(&self, agent_id: &str) -> KernelResult<Vec<AgentConfigurationProfile>> {
        let pool = self.pool.clone();
        let agent_id = agent_id.to_owned();
        let rows: Vec<(String, String, String, String, String, String)> = self
            .pool
            .block_on(async move {
                sqlx::query_as::<_, (String, String, String, String, String, String)>(
                    "SELECT profile_id, agent_id, configuration_version, status,
                            configuration_json, secret_bindings_json
                     FROM ai_agent_model_configuration_profile
                     WHERE agent_id = $1
                     ORDER BY profile_id",
                )
                .bind(&agent_id)
                .fetch_all(pool.pool())
                .await
                .map_err(|error| store_error("list_query", error.to_string()))
            })?;
        rows.into_iter()
            .map(
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
            .collect()
    }

    fn find_profile(
        &self,
        agent_id: &str,
        profile_id: &str,
    ) -> KernelResult<Option<AgentConfigurationProfile>> {
        let pool = self.pool.clone();
        let agent_id = agent_id.to_owned();
        let profile_id = profile_id.to_owned();
        let row: Option<(String, String, String, String, String, String)> = self
            .pool
            .block_on(async move {
                sqlx::query_as::<_, (String, String, String, String, String, String)>(
                    "SELECT profile_id, agent_id, configuration_version, status,
                            configuration_json, secret_bindings_json
                     FROM ai_agent_model_configuration_profile
                     WHERE profile_id = $1 AND agent_id = $2",
                )
                .bind(&profile_id)
                .bind(&agent_id)
                .fetch_optional(pool.pool())
                .await
                .map_err(|error| store_error("find_query", error.to_string()))
            })?;
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
                .fetch_add(1, Ordering::Relaxed)
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
    let configuration =
        serde_json::from_str::<serde_json::Value>(configuration_json).map_err(|error| {
            store_error(
                "parse_configuration",
                format!("stored configuration is not valid JSON: {error}"),
            )
        })?;
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
        AgentConfigValue::StringList(values) => serde_json::Value::Array(
            values
                .iter()
                .cloned()
                .map(serde_json::Value::String)
                .collect(),
        ),
        AgentConfigValue::Json(value) => {
            serde_json::from_str(value).unwrap_or_else(|_| serde_json::Value::String(value.clone()))
        }
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
                .set(
                    "codex.model.base_url",
                    AgentConfigValue::string("https://api.birdcoder.com/v1"),
                )
                .set("codex.model.default", AgentConfigValue::string("gpt-5.4"))
                .set(
                    "codex.model.api_key",
                    AgentConfigValue::secret_ref("secret.model.key-1"),
                )
                .set(
                    "codex.model.supported",
                    AgentConfigValue::string_list(vec!["gpt-5.4".to_string()]),
                )
                .set(
                    "codex.model.supports_multimodal",
                    AgentConfigValue::boolean(false),
                ),
        )
        .add_secret_binding(AgentSecretBinding::llm_api_key(
            "codex.model.api_key",
            "openai",
            "secret.model.key-1",
        ))
        .activate()
    }

    #[test]
    fn profile_serialization_round_trips_kernel_manifest_shape() {
        let profile = sample_profile("agent-1", "profile-1");
        let document = serde_json::json!({
            "manifest_type": "agent_configuration_profile",
            "profile_id": profile.profile_id,
            "agent_id": profile.agent_id,
            "configuration_version": profile.configuration_version,
            "status": profile.status.as_str(),
            "configuration": configuration_to_json(&profile),
            "secret_bindings": secret_bindings_to_json(&profile),
        });
        let parsed = AgentConfigurationProfile::from_json(&document.to_string())
            .expect("kernel manifest parser");
        assert_eq!(parsed.profile_id, profile.profile_id);
        assert_eq!(parsed.agent_id, profile.agent_id);
        assert_eq!(parsed.status, AgentConfigurationProfileStatus::Active);
        assert_eq!(parsed.secret_bindings.len(), 1);
    }
}
