//! Server-side persistence of generated media into the SDKWork Drive.
//!
//! Implements the DRIVE_SPEC server-side path: generated content is written
//! through the approved Drive uploader service (`sdkwork-drive-uploader-service`)
//! into the `AiGeneratedSpace` layout, with the storage provider resolved
//! through the Drive object-store runtime. Business code never talks to a
//! storage provider SDK directly.

use std::sync::Arc;

use sdkwork_agents_tool_contract::MediaResource;
use sdkwork_drive_object_runtime::DriveObjectStoreRuntime;
use sdkwork_drive_storage_contract::DriveObjectStore;
use sdkwork_drive_uploader_service::service::{
    DriveUploaderService, PrepareUploaderUploadCommand, SqlUploaderStore, UploadBytesCommand,
    UploaderActor, UploaderRetention, UploaderTarget,
};
use sdkwork_drive_workspace_service::ports::storage_provider_store::DriveStorageProviderStore;
use sqlx::PgPool;

/// Application identity recorded on every generated-media upload item.
const APP_ID: &str = "sdkwork-agents";
/// App resource type recorded on generated-media uploads.
const APP_RESOURCE_TYPE: &str = "ai_generated";
/// Scene label recorded on generated-media uploads.
const SCENE: &str = "ai_generated";
/// Source label recorded on generated-media uploads.
const SOURCE: &str = "sdkwork-agents";
/// Default chunk size aligned with the drive uploader convention.
const DEFAULT_CHUNK_SIZE_BYTES: i64 = 8 * 1024 * 1024;

/// Stable reference to a Drive-persisted generated asset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriveAssetRef {
    pub space_id: String,
    pub node_id: String,
    pub drive_uri: String,
}

/// Context describing who generated the content and under which app resource
/// it is stored.
#[derive(Debug, Clone)]
pub struct DriveSaveContext {
    pub tenant_id: String,
    pub organization_id: Option<String>,
    pub actor: UploaderActor,
    pub app_resource_id: String,
    pub tool_call_id: String,
    pub tool_id: String,
}

/// Server-side saver of generated media into the SDKWork Drive.
#[derive(Debug, Clone)]
pub struct DriveAssetSaver {
    pool: PgPool,
}

impl DriveAssetSaver {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Persists one generated media resource (already fetched into `body`)
    /// into the tenant's AI-generated Drive space, returning the stable
    /// `drive://spaces/.../nodes/...` reference.
    pub async fn save_generated_media(
        &self,
        context: &DriveSaveContext,
        resource: &MediaResource,
        body: Vec<u8>,
    ) -> Result<DriveAssetRef, DriveSaveError> {
        let profile = upload_profile_for_kind(&resource.kind);
        let fingerprint = format!("{}:ai-generated:{}", APP_ID, context.tool_call_id);
        let file_name = generated_file_name(context, resource);
        let prepare = PrepareUploaderUploadCommand {
            id: format!("upload-{}", context.tool_call_id),
            task_id: format!("task-{}", context.tool_call_id),
            tenant_id: context.tenant_id.clone(),
            organization_id: context.organization_id.clone(),
            actor: context.actor.clone(),
            app_id: APP_ID.to_string(),
            app_resource_type: APP_RESOURCE_TYPE.to_string(),
            app_resource_id: context.app_resource_id.clone(),
            scene: Some(SCENE.to_string()),
            source: Some(SOURCE.to_string()),
            upload_profile_code: profile.to_string(),
            file_fingerprint: fingerprint,
            original_file_name: file_name,
            content_type: resource
                .mime_type
                .clone()
                .unwrap_or_else(|| "application/octet-stream".to_string()),
            content_length: body.len() as i64,
            chunk_size_bytes: DEFAULT_CHUNK_SIZE_BYTES,
            target: UploaderTarget::AiGeneratedSpace {
                parent_node_id: None,
            },
            retention: UploaderRetention::LongTerm,
            operator_id: operator_id(&context.actor),
            now_epoch_ms: now_epoch_ms(),
        };

        let store = SqlUploaderStore::new(self.pool.clone());
        let service = DriveUploaderService::new(store);
        let object_store = self.resolve_active_object_store().await?;

        let item = service
            .upload_bytes(
                object_store.as_ref(),
                UploadBytesCommand {
                    prepare,
                    body,
                    uploaded_at_epoch_ms: now_epoch_ms(),
                },
            )
            .await
            .map_err(|error| DriveSaveError::Upload(service_error_message(error)))?;

        let space_id = item.space_id;
        let node_id = item.node_id;
        let drive_uri = format!("drive://spaces/{space_id}/nodes/{node_id}");
        Ok(DriveAssetRef {
            space_id,
            node_id,
            drive_uri,
        })
    }

    /// Resolves the first active storage provider adapter for the shared
    /// database, used as the `DriveObjectStore` for uploads.
    async fn resolve_active_object_store(
        &self,
    ) -> Result<Arc<dyn DriveObjectStore>, DriveSaveError> {
        let provider_store =
            sdkwork_drive_workspace_service::infrastructure::sql::storage_provider_store::SqlStorageProviderStore::new(
                self.pool.clone(),
            );
        let providers = provider_store
            .list_storage_providers(Some("active"), 0, 1)
            .await
            .map_err(|error| DriveSaveError::Provider(service_error_message(error)))?;
        let provider = providers.first().ok_or_else(|| {
            DriveSaveError::Provider("no active drive storage provider configured".to_string())
        })?;

        let runtime = DriveObjectStoreRuntime::new(self.pool.clone());
        let store = runtime
            .resolve(&provider.id, provider.version)
            .await
            .map_err(|error| DriveSaveError::Provider(error.to_string()))?;
        Ok(store)
    }
}

/// Extracts the message from a drive service error without a Display impl.
fn service_error_message(error: sdkwork_drive_workspace_service::DriveServiceError) -> String {
    use sdkwork_drive_workspace_service::DriveServiceError;
    match error {
        DriveServiceError::Validation(message)
        | DriveServiceError::Conflict(message)
        | DriveServiceError::NotFound(message)
        | DriveServiceError::PermissionDenied(message)
        | DriveServiceError::Internal(message) => message,
    }
}

/// Maps a media resource kind to the drive uploader profile code.
fn upload_profile_for_kind(kind: &str) -> &'static str {
    match kind {
        "image" => "image",
        "video" => "video",
        "audio" | "voice" | "music" => "audio",
        "document" => "document",
        _ => "generic",
    }
}

/// Builds a stable file name for generated content.
fn generated_file_name(context: &DriveSaveContext, resource: &MediaResource) -> String {
    let extension = resource
        .file_name
        .as_deref()
        .and_then(|name| name.rsplit_once('.').map(|(_, ext)| ext))
        .unwrap_or("bin");
    format!("{}-{}.{extension}", context.tool_id, context.tool_call_id)
}

/// Extracts the operator identity from the uploader actor.
fn operator_id(actor: &UploaderActor) -> String {
    match actor {
        UploaderActor::Anonymous { anonymous_id } => anonymous_id.clone(),
        UploaderActor::User { user_id } => user_id.clone(),
        UploaderActor::System { operator_id } => operator_id.clone(),
    }
}

fn now_epoch_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0)
}

/// Fetches the bytes of a generated media resource from its delivery URL.
pub async fn fetch_resource_bytes(url: &str) -> Result<Vec<u8>, DriveSaveError> {
    let response = reqwest::get(url)
        .await
        .map_err(|error| DriveSaveError::Fetch(error.to_string()))?;
    if !response.status().is_success() {
        return Err(DriveSaveError::Fetch(format!(
            "fetch failed with status {}",
            response.status()
        )));
    }
    response
        .bytes()
        .await
        .map(|bytes| bytes.to_vec())
        .map_err(|error| DriveSaveError::Fetch(error.to_string()))
}

/// Errors surfaced by the server-side drive persistence path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DriveSaveError {
    /// Fetching the generated content failed.
    Fetch(String),
    /// Resolving the storage provider or object store failed.
    Provider(String),
    /// The drive uploader service rejected the upload.
    Upload(String),
}

impl std::fmt::Display for DriveSaveError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DriveSaveError::Fetch(message)
            | DriveSaveError::Provider(message)
            | DriveSaveError::Upload(message) => write!(formatter, "{message}"),
        }
    }
}

impl std::error::Error for DriveSaveError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_maps_media_kinds() {
        assert_eq!(upload_profile_for_kind("image"), "image");
        assert_eq!(upload_profile_for_kind("video"), "video");
        assert_eq!(upload_profile_for_kind("audio"), "audio");
        assert_eq!(upload_profile_for_kind("music"), "audio");
        assert_eq!(upload_profile_for_kind("document"), "document");
        assert_eq!(upload_profile_for_kind("unknown"), "generic");
    }

    #[test]
    fn file_name_uses_tool_and_call_ids() {
        let context = DriveSaveContext {
            tenant_id: "tenant.1".to_string(),
            organization_id: None,
            actor: UploaderActor::System {
                operator_id: "operator.1".to_string(),
            },
            app_resource_id: "call.1".to_string(),
            tool_call_id: "call.1".to_string(),
            tool_id: "image.generations.create".to_string(),
        };
        let resource = MediaResource {
            kind: "image".to_string(),
            source: "provider_asset".to_string(),
            url: "https://cdn.example/a.png".to_string(),
            file_name: Some("a.png".to_string()),
            ..Default::default()
        };
        assert_eq!(
            generated_file_name(&context, &resource),
            "image.generations.create-call.1.png"
        );
    }

    #[test]
    fn error_display_is_actionable() {
        assert_eq!(
            DriveSaveError::Fetch("timeout".to_string()).to_string(),
            "timeout"
        );
    }
}
