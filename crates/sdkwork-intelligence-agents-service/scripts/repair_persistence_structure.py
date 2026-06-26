#!/usr/bin/env python3
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
path = ROOT / "src" / "persistence.rs"
lines = path.read_text(encoding="utf-8").splitlines(keepends=True)
lines = lines[:530] + lines[624:]
text = "".join(lines)
text = text.replace(
    '    validate_slug_list(record.tags.as_slice(), "tags")\n}',
    '    validate_slug_list(record.tags.as_slice(), "tags")?;\n    Ok(())\n}',
    1,
)
text = text.replace(
    """        .unwrap_or_default()
    }

#[cfg(feature = "postgres-sync")]
fn pg_row_to_agent_composition_slot_row""",
    """        .unwrap_or_default()
    }
}

#[cfg(feature = "postgres-sync")]
fn pg_row_to_agent_composition_slot_row""",
    1,
)
text = text.replace(
    "\n}\n\n\n#[cfg(feature = \"postgres-sync\")]\nimpl PostgresAuditAdapter",
    "\n\n#[cfg(feature = \"postgres-sync\")]\nimpl PostgresAuditAdapter",
    1,
)
marker = '#[cfg(feature = "postgres-sync")]\n        assert!(SQL_INSERT_AGENT_KNOWLEDGE_SOURCE'
idx = text.find(marker)
if idx != -1:
    text = text[:idx].rstrip() + "\n"
dup = text.find("}#[derive(Debug, Clone, PartialEq, Eq)]\npub struct AgentCompositionSlotRow")
if dup != -1:
    text = text[: dup + 1] + "\n"
path.write_text(text, encoding="utf-8")
print(f"repaired {path.name}: {len(text.splitlines())} lines")
