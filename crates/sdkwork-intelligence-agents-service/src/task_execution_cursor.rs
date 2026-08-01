use sdkwork_agent_kernel::{KernelError, KernelResult};
use sdkwork_utils_rust::{
    encoding::{base64url_decode, base64url_encode},
    parse_datetime, sha256_hash,
};
use serde::{Deserialize, Serialize};

const TASK_EXECUTION_CURSOR_VERSION: u8 = 1;
const MAX_TASK_EXECUTION_CURSOR_BYTES: usize = 2048;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskCursor {
    pub updated_at: String,
    pub task_internal_id: u64,
    pub scope_fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskRunCursor {
    pub run_internal_id: u64,
    pub scope_fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskRunAttemptCursor {
    pub attempt_no: u16,
    pub attempt_internal_id: u64,
    pub scope_fingerprint: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct TaskExecutionCursorPayload {
    version: u8,
    kind: String,
    position: String,
    internal_id: String,
    scope_fingerprint: String,
}

pub(crate) fn encode_task_cursor(cursor: &TaskCursor) -> KernelResult<String> {
    encode_cursor(TaskExecutionCursorPayload {
        version: TASK_EXECUTION_CURSOR_VERSION,
        kind: "task".to_string(),
        position: cursor.updated_at.clone(),
        internal_id: cursor.task_internal_id.to_string(),
        scope_fingerprint: cursor.scope_fingerprint.clone(),
    })
}

pub(crate) fn decode_task_cursor(value: &str) -> KernelResult<TaskCursor> {
    let payload = decode_cursor(value, "task")?;
    if parse_datetime(&payload.position, None).is_none() {
        return Err(invalid_cursor());
    }
    Ok(TaskCursor {
        updated_at: payload.position,
        task_internal_id: parse_positive_u64(&payload.internal_id)?,
        scope_fingerprint: payload.scope_fingerprint,
    })
}

pub(crate) fn encode_task_run_cursor(cursor: &TaskRunCursor) -> KernelResult<String> {
    encode_cursor(TaskExecutionCursorPayload {
        version: TASK_EXECUTION_CURSOR_VERSION,
        kind: "run".to_string(),
        position: cursor.run_internal_id.to_string(),
        internal_id: cursor.run_internal_id.to_string(),
        scope_fingerprint: cursor.scope_fingerprint.clone(),
    })
}

pub(crate) fn decode_task_run_cursor(value: &str) -> KernelResult<TaskRunCursor> {
    let payload = decode_cursor(value, "run")?;
    let run_internal_id = parse_positive_u64(&payload.internal_id)?;
    if parse_positive_u64(&payload.position)? != run_internal_id {
        return Err(invalid_cursor());
    }
    Ok(TaskRunCursor {
        run_internal_id,
        scope_fingerprint: payload.scope_fingerprint,
    })
}

pub(crate) fn encode_task_run_attempt_cursor(
    cursor: &TaskRunAttemptCursor,
) -> KernelResult<String> {
    encode_cursor(TaskExecutionCursorPayload {
        version: TASK_EXECUTION_CURSOR_VERSION,
        kind: "attempt".to_string(),
        position: cursor.attempt_no.to_string(),
        internal_id: cursor.attempt_internal_id.to_string(),
        scope_fingerprint: cursor.scope_fingerprint.clone(),
    })
}

pub(crate) fn decode_task_run_attempt_cursor(value: &str) -> KernelResult<TaskRunAttemptCursor> {
    let payload = decode_cursor(value, "attempt")?;
    let attempt_no = payload
        .position
        .parse::<u16>()
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(invalid_cursor)?;
    Ok(TaskRunAttemptCursor {
        attempt_no,
        attempt_internal_id: parse_positive_u64(&payload.internal_id)?,
        scope_fingerprint: payload.scope_fingerprint,
    })
}

pub(crate) fn task_run_scope_fingerprint(
    tenant_id: u64,
    organization_id: u64,
    task_id: &str,
    owner_user_id: Option<u64>,
    status: Option<&str>,
    trigger_kind: Option<&str>,
) -> String {
    sha256_hash(
        serde_json::json!({
            "version": TASK_EXECUTION_CURSOR_VERSION,
            "kind": "run",
            "tenantId": tenant_id.to_string(),
            "organizationId": organization_id.to_string(),
            "taskId": task_id,
            "ownerUserId": owner_user_id.map(|value| value.to_string()),
            "status": status,
            "triggerKind": trigger_kind,
            "sort": "-id",
        })
        .to_string()
        .as_bytes(),
    )
}

pub(crate) fn task_scope_fingerprint(
    tenant_id: u64,
    organization_id: u64,
    agent_id: Option<&str>,
    owner_user_id: Option<u64>,
    status: Option<&str>,
) -> String {
    sha256_hash(
        serde_json::json!({
            "version": TASK_EXECUTION_CURSOR_VERSION,
            "kind": "task",
            "tenantId": tenant_id.to_string(),
            "organizationId": organization_id.to_string(),
            "agentId": agent_id,
            "ownerUserId": owner_user_id.map(|value| value.to_string()),
            "status": status,
            "sort": ["-updatedAt", "-id"],
        })
        .to_string()
        .as_bytes(),
    )
}

pub(crate) fn task_run_attempt_scope_fingerprint(
    tenant_id: u64,
    organization_id: u64,
    run_id: &str,
) -> String {
    sha256_hash(
        serde_json::json!({
            "version": TASK_EXECUTION_CURSOR_VERSION,
            "kind": "attempt",
            "tenantId": tenant_id.to_string(),
            "organizationId": organization_id.to_string(),
            "runId": run_id,
            "sort": ["-attemptNo", "-id"],
        })
        .to_string()
        .as_bytes(),
    )
}

fn encode_cursor(payload: TaskExecutionCursorPayload) -> KernelResult<String> {
    let json = serde_json::to_vec(&payload).map_err(|error| KernelError::Internal {
        message: format!("failed to encode task execution cursor: {error}"),
    })?;
    Ok(base64url_encode(&json))
}

fn decode_cursor(value: &str, expected_kind: &str) -> KernelResult<TaskExecutionCursorPayload> {
    if value.is_empty() || value.len() > MAX_TASK_EXECUTION_CURSOR_BYTES || value.trim() != value {
        return Err(invalid_cursor());
    }
    let decoded = base64url_decode(value).ok_or_else(invalid_cursor)?;
    let payload: TaskExecutionCursorPayload =
        serde_json::from_slice(&decoded).map_err(|_| invalid_cursor())?;
    if payload.version != TASK_EXECUTION_CURSOR_VERSION
        || payload.kind != expected_kind
        || payload.scope_fingerprint.len() != 64
        || !payload
            .scope_fingerprint
            .bytes()
            .all(|value| value.is_ascii_hexdigit())
    {
        return Err(invalid_cursor());
    }
    Ok(payload)
}

fn parse_positive_u64(value: &str) -> KernelResult<u64> {
    value
        .parse::<u64>()
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(invalid_cursor)
}

fn invalid_cursor() -> KernelError {
    KernelError::validation("cursor is not a valid opaque token")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_cursor_preserves_keyset_and_rejects_other_kinds() {
        let cursor = TaskCursor {
            updated_at: "2026-07-31T09:30:00.000Z".to_string(),
            task_internal_id: 21,
            scope_fingerprint: task_scope_fingerprint(
                10,
                20,
                Some("agent.cursor"),
                Some(30),
                Some("active"),
            ),
        };
        let encoded = encode_task_cursor(&cursor).expect("encode cursor");
        assert!(!encoded.contains("agent.cursor"));
        assert_eq!(decode_task_cursor(&encoded).unwrap(), cursor);
        assert!(decode_task_run_cursor(&encoded).is_err());
        assert!(decode_task_cursor("21").is_err());
    }

    #[test]
    fn run_cursor_is_versioned_opaque_and_scope_bound() {
        let cursor = TaskRunCursor {
            run_internal_id: 42,
            scope_fingerprint: task_run_scope_fingerprint(
                10,
                20,
                "task.cursor",
                Some(30),
                Some("failed"),
                None,
            ),
        };
        let encoded = encode_task_run_cursor(&cursor).expect("encode cursor");
        assert!(!encoded.contains("task.cursor"));
        assert_eq!(decode_task_run_cursor(&encoded).unwrap(), cursor);
        assert!(decode_task_run_attempt_cursor(&encoded).is_err());
        assert!(decode_task_run_cursor("42").is_err());
    }

    #[test]
    fn attempt_cursor_preserves_compound_order_position() {
        let cursor = TaskRunAttemptCursor {
            attempt_no: 3,
            attempt_internal_id: 84,
            scope_fingerprint: task_run_attempt_scope_fingerprint(10, 20, "run.cursor"),
        };
        let encoded = encode_task_run_attempt_cursor(&cursor).expect("encode cursor");
        assert_eq!(decode_task_run_attempt_cursor(&encoded).unwrap(), cursor);
    }
}
