use std::sync::Arc;

use sdkwork_database_drift::{DriftEngine, DriftReport};
use sdkwork_database_spi::{DatabaseModule, DefaultDatabaseModule};
use sdkwork_database_sqlx::DatabasePool;
use sdkwork_utils_rust::string::truncate;

const MAX_DRIFT_ERROR_DETAILS: usize = 8;
const MAX_DRIFT_ERROR_DETAIL_CHARS: usize = 256;

pub(crate) async fn ensure_agents_schema_current(
    pool: &DatabasePool,
    module: Arc<DefaultDatabaseModule>,
) -> Result<(), String> {
    let drift_module: Arc<dyn DatabaseModule> = module;
    let report = DriftEngine::new(pool.clone(), drift_module)
        .analyze()
        .await
        .map_err(|error| format!("analyze agents database drift failed: {error}"))?;
    enforce_agents_schema_drift_gate(&report)?;

    if report.summary.warn > 0 {
        tracing::warn!(
            module = %report.module_id,
            drift_status = %report.status,
            expected_tables = report.expected_tables.len(),
            warning_diffs = report.summary.warn,
            info_diffs = report.summary.info,
            "agents database schema passed startup drift gate with warnings"
        );
    } else {
        tracing::info!(
            module = %report.module_id,
            drift_status = %report.status,
            expected_tables = report.expected_tables.len(),
            info_diffs = report.summary.info,
            "agents database schema passed startup drift gate"
        );
    }

    Ok(())
}

fn enforce_agents_schema_drift_gate(report: &DriftReport) -> Result<(), String> {
    let observed_error_count = report
        .diffs
        .iter()
        .filter(|diff| diff.severity == "error")
        .count();
    let error_count = observed_error_count.max(report.summary.error as usize);
    if error_count == 0 {
        return Ok(());
    }

    let error_details = report
        .diffs
        .iter()
        .filter(|diff| diff.severity == "error")
        .take(MAX_DRIFT_ERROR_DETAILS)
        .map(|diff| sanitize_drift_error_detail(&diff.message))
        .collect::<Vec<_>>();
    let included_detail_count = error_details.len();
    let mut detail_summary = if error_details.is_empty() {
        "drift report omitted error details".to_string()
    } else {
        error_details.join("; ")
    };
    let omitted_detail_count = error_count.saturating_sub(included_detail_count);
    if omitted_detail_count > 0 {
        detail_summary.push_str(&format!(
            "; {omitted_detail_count} additional error detail(s) omitted"
        ));
    }

    Err(format!(
        "agents database schema is incomplete: {} error-level drift(s): {}",
        error_count, detail_summary
    ))
}

fn sanitize_drift_error_detail(message: &str) -> String {
    let single_line = message
        .chars()
        .map(|character| {
            if character.is_control() {
                ' '
            } else {
                character
            }
        })
        .collect::<String>();
    truncate(&single_line, MAX_DRIFT_ERROR_DETAIL_CHARS, Some("..."))
}

#[cfg(test)]
mod tests {
    use sdkwork_database_drift::{DriftDiff, DriftReport, DriftSummary};

    use super::{enforce_agents_schema_drift_gate, MAX_DRIFT_ERROR_DETAIL_CHARS};

    fn drift_report(summary: DriftSummary, diffs: Vec<DriftDiff>) -> DriftReport {
        DriftReport {
            schema_version: 1,
            kind: "sdkwork.database.drift-report".to_string(),
            checked_at: "2026-07-24T00:00:00Z".to_string(),
            module_id: "agents".to_string(),
            service_code: "AGENTS".to_string(),
            engine: "postgres".to_string(),
            status: if summary.error > 0 {
                "drift_detected".to_string()
            } else {
                "clean".to_string()
            },
            summary,
            pending_migrations: Vec::new(),
            live_tables: Vec::new(),
            expected_tables: vec!["ai_agent".to_string()],
            diffs,
        }
    }

    fn drift_diff(severity: &str, message: impl Into<String>) -> DriftDiff {
        DriftDiff {
            code: "test_drift".to_string(),
            severity: severity.to_string(),
            message: message.into(),
        }
    }

    #[test]
    fn current_schema_passes_the_startup_gate() {
        let report = drift_report(DriftSummary::default(), Vec::new());

        assert_eq!(enforce_agents_schema_drift_gate(&report), Ok(()));
    }

    #[test]
    fn warning_only_schema_passes_the_startup_gate() {
        let report = drift_report(
            DriftSummary {
                warn: 1,
                ..DriftSummary::default()
            },
            vec![drift_diff("warn", "missing optional index")],
        );

        assert_eq!(enforce_agents_schema_drift_gate(&report), Ok(()));
    }

    #[test]
    fn error_diff_fails_closed_when_summary_is_inconsistent() {
        let report = drift_report(
            DriftSummary::default(),
            vec![drift_diff("error", "missing table: ai_agent_session")],
        );

        let error = enforce_agents_schema_drift_gate(&report).expect_err("error drift must fail");
        assert!(error.contains("1 error-level drift(s)"));
        assert!(error.contains("missing table: ai_agent_session"));
    }

    #[test]
    fn summary_error_fails_closed_when_details_are_missing() {
        let report = drift_report(
            DriftSummary {
                error: 2,
                ..DriftSummary::default()
            },
            Vec::new(),
        );

        let error = enforce_agents_schema_drift_gate(&report).expect_err("summary error must fail");
        assert!(error.contains("2 error-level drift(s)"));
        assert!(error.contains("drift report omitted error details"));
    }

    #[test]
    fn error_details_are_single_line_and_bounded() {
        let diffs = (0..10)
            .map(|index| {
                drift_diff(
                    "error",
                    format!(
                        "detail-{index}\n{}",
                        "x".repeat(MAX_DRIFT_ERROR_DETAIL_CHARS * 2)
                    ),
                )
            })
            .collect();
        let report = drift_report(
            DriftSummary {
                error: 10,
                ..DriftSummary::default()
            },
            diffs,
        );

        let error = enforce_agents_schema_drift_gate(&report).expect_err("error drift must fail");
        assert!(!error.contains('\n'));
        assert!(error.contains("detail-0"));
        assert!(error.contains("detail-7"));
        assert!(!error.contains("detail-8"));
        assert!(error.contains("2 additional error detail(s) omitted"));
        assert!(!error.contains(&"x".repeat(MAX_DRIFT_ERROR_DETAIL_CHARS)));
    }
}
