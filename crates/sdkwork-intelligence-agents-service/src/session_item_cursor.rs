use sdkwork_agent_kernel::{KernelError, KernelResult};
use sdkwork_utils_rust::{
    encoding::{base64url_decode, base64url_encode},
    sha256_hash,
};
use serde::{Deserialize, Serialize};

const SESSION_ITEM_CURSOR_VERSION: u8 = 1;
const MAX_SESSION_ITEM_CURSOR_BYTES: usize = 2048;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionItemCursor {
    pub sequence: u64,
    pub item_internal_id: u64,
    pub scope_fingerprint: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SessionItemCursorPayload {
    version: u8,
    sequence: String,
    item_internal_id: String,
    scope_fingerprint: String,
}

pub(crate) fn encode_session_item_cursor(cursor: &SessionItemCursor) -> KernelResult<String> {
    let payload = SessionItemCursorPayload {
        version: SESSION_ITEM_CURSOR_VERSION,
        sequence: cursor.sequence.to_string(),
        item_internal_id: cursor.item_internal_id.to_string(),
        scope_fingerprint: cursor.scope_fingerprint.clone(),
    };
    let json = serde_json::to_vec(&payload).map_err(|error| KernelError::Internal {
        message: format!("failed to encode session item cursor: {error}"),
    })?;
    Ok(base64url_encode(&json))
}

pub(crate) fn decode_session_item_cursor(value: &str) -> KernelResult<SessionItemCursor> {
    if value.is_empty() || value.len() > MAX_SESSION_ITEM_CURSOR_BYTES || value.trim() != value {
        return Err(KernelError::validation(
            "cursor is not a valid opaque token",
        ));
    }
    let decoded = base64url_decode(value)
        .ok_or_else(|| KernelError::validation("cursor is not a valid opaque token"))?;
    let payload: SessionItemCursorPayload = serde_json::from_slice(&decoded)
        .map_err(|_| KernelError::validation("cursor is not a valid opaque token"))?;
    if payload.version != SESSION_ITEM_CURSOR_VERSION {
        return Err(KernelError::validation("cursor version is not supported"));
    }
    let sequence = payload
        .sequence
        .parse::<u64>()
        .map_err(|_| KernelError::validation("cursor is not a valid opaque token"))?;
    let item_internal_id = payload
        .item_internal_id
        .parse::<u64>()
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| KernelError::validation("cursor is not a valid opaque token"))?;
    if payload.scope_fingerprint.len() != 64
        || !payload
            .scope_fingerprint
            .bytes()
            .all(|value| value.is_ascii_hexdigit())
    {
        return Err(KernelError::validation(
            "cursor is not a valid opaque token",
        ));
    }
    Ok(SessionItemCursor {
        sequence,
        item_internal_id,
        scope_fingerprint: payload.scope_fingerprint,
    })
}

pub(crate) fn session_item_scope_fingerprint(
    tenant_id: u64,
    organization_id: u64,
    session_id: &str,
    kind: Option<&str>,
    status: Option<&str>,
    sort: &str,
) -> String {
    let scope = serde_json::json!({
        "version": SESSION_ITEM_CURSOR_VERSION,
        "tenantId": tenant_id.to_string(),
        "organizationId": organization_id.to_string(),
        "sessionId": session_id,
        "kind": kind,
        "status": status,
        "sort": sort,
    });
    sha256_hash(scope.to_string().as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_item_cursor_is_opaque_versioned_and_scope_bound() {
        let scope_fingerprint = session_item_scope_fingerprint(
            10,
            20,
            "session.cursor",
            None,
            Some("completed"),
            "-sequence",
        );
        let cursor = SessionItemCursor {
            sequence: 42,
            item_internal_id: 84,
            scope_fingerprint,
        };

        let encoded = encode_session_item_cursor(&cursor).expect("encode cursor");

        assert!(!encoded.contains("session.cursor"));
        assert_eq!(decode_session_item_cursor(&encoded).unwrap(), cursor);
        assert!(decode_session_item_cursor("42").is_err());
        assert!(decode_session_item_cursor(" cursor ").is_err());
        assert!(decode_session_item_cursor(&format!("{encoded}x")).is_err());
    }
}
