#!/usr/bin/env python3
"""Repair persistence.rs after failed line-number cutover."""
from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SRC = ROOT / "src"
TEMPLATES = Path(__file__).resolve().parent / "composition_templates"

AGENT_BUSINESS_ROW = """#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentBusinessRow {
    pub id: u64,
    pub uuid: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub owner_user_id: u64,
    pub agent_id: String,
    pub code: String,
    pub display_name: String,
    pub description: Option<String>,
    pub manifest_json: String,
    pub default_code_task_intent_json: Option<String>,
    pub implementation_provider_id: Option<String>,
    pub implementation_kind: Option<String>,
    pub implementation_type: String,
    pub status: i16,
    pub visibility: i16,
    pub tags_json: String,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub version: u64,
}
"""

MARKETPLACE_HELPERS = """
struct MarketplaceRecordView<'a> {
    tenant_id: u64,
    organization_id: u64,
    owner_user_id: u64,
    status: AgentBusinessStatus,
    visibility: AgentVisibility,
    deleted: bool,
    id: &'a str,
    code: &'a str,
    display_name: &'a str,
    description: Option<&'a str>,
    categories: &'a [String],
    tags: &'a [String],
}

fn marketplace_record_matches(query: &AgentMarketplaceListQuery, record: MarketplaceRecordView<'_>) -> bool {
    if record.tenant_id != query.tenant_id {
        return false;
    }
    if let Some(organization_id) = query.organization_id {
        if record.organization_id != organization_id {
            return false;
        }
    }
    if let Some(owner_user_id) = query.owner_user_id {
        if record.owner_user_id != owner_user_id {
            return false;
        }
    }
    if let Some(status) = query.status {
        if record.status != status {
            return false;
        }
    }
    if let Some(visibility) = query.visibility {
        if record.visibility != visibility {
            return false;
        }
    }
    if !query.include_deleted && record.deleted {
        return false;
    }
    if let Some(category) = query.category.as_deref() {
        if !record.categories.iter().any(|value| value == category) {
            return false;
        }
    }
    if let Some(tag) = query.tag.as_deref() {
        if !record.tags.iter().any(|value| value == tag) {
            return false;
        }
    }
    if let Some(search) = query.search_query.as_deref() {
        let needle = search.to_ascii_lowercase();
        let haystacks = [
            record.id,
            record.code,
            record.display_name,
            record.description.unwrap_or_default(),
        ];
        if !haystacks
            .iter()
            .any(|value| value.to_ascii_lowercase().contains(needle.as_str()))
        {
            return false;
        }
    }
    true
}

fn compare_marketplace_standard_order(
    left_updated_at: &str,
    left_code: &str,
    right_updated_at: &str,
    right_code: &str,
) -> std::cmp::Ordering {
    right_updated_at
        .cmp(left_updated_at)
        .then_with(|| left_code.cmp(right_code))
}

fn marketplace_row_matches(
    query: &AgentMarketplaceListQuery,
    organization_id: u64,
    owner_user_id: u64,
    status: i16,
    visibility: i16,
    deleted_at: Option<&str>,
    id: &str,
    code: &str,
    display_name: &str,
    description: Option<&str>,
    categories_json: &str,
    tags_json: &str,
) -> bool {
    let categories = string_list_from_json(categories_json, "categories").unwrap_or_default();
    let tags = string_list_from_json(tags_json, "tags").unwrap_or_default();
    marketplace_record_matches(
        query,
        MarketplaceRecordView {
            tenant_id: query.tenant_id,
            organization_id,
            owner_user_id,
            status: AgentBusinessStatus::from_db_code(status)
                .unwrap_or(AgentBusinessStatus::Draft),
            visibility: AgentVisibility::from_db_code(visibility)
                .unwrap_or(AgentVisibility::Private),
            deleted: deleted_at.is_some(),
            id,
            code,
            display_name,
            description,
            categories: categories.as_slice(),
            tags: tags.as_slice(),
        },
    )
}
"""

LIST_MCP_SERVER_ROWS = """
    fn list_mcp_server_rows(&self, query: &AgentMarketplaceListQuery) -> Vec<AgentMcpServerRow> {
        let tenant_id = match u64_to_i64(query.tenant_id, "tenant_id") {
            Ok(value) => value,
            Err(_) => return Vec::new(),
        };
        let mut rows = self
            .with_pool(|pool| {
                let rows = pg_query!(pool, SQL_LIST_AGENT_MCP_SERVERS, tenant_id)?;
                rows.into_iter()
                    .map(pg_row_to_agent_mcp_server_row)
                    .collect::<KernelResult<Vec<_>>>()
            })
            .unwrap_or_default()
            .into_iter()
            .filter_map(|row| row.ok())
            .filter(|row| {
                marketplace_row_matches(
                    query,
                    row.organization_id,
                    row.owner_user_id,
                    row.status,
                    row.visibility,
                    row.deleted_at.as_deref(),
                    row.mcp_server_id.as_str(),
                    row.code.as_str(),
                    row.display_name.as_str(),
                    row.description.as_deref(),
                    row.categories_json.as_str(),
                    row.tags_json.as_str(),
                )
            })
            .collect::<Vec<_>>();
        rows.sort_by(|left, right| {
            compare_marketplace_standard_order(
                left.updated_at.as_str(),
                left.code.as_str(),
                right.updated_at.as_str(),
                right.code.as_str(),
            )
        });
        rows
    }
"""

POSTGRES_AUDIT_TRAIT = """
pub trait PostgresAuditAdapter {
    fn next_id(&mut self) -> KernelResult<u64>;
    fn insert_audit_row(&mut self, row: AgentAuditEventRow) -> KernelResult<()>;
    fn list_audit_rows(
        &self,
        tenant_id: u64,
        agent_id: &str,
    ) -> KernelResult<Vec<AgentAuditEventRow>> {
        let _ = (tenant_id, agent_id);
        Ok(Vec::new())
    }
}
"""


def remove_fn_block(content: str, signature: str) -> str:
    start = content.find(signature)
    if start == -1:
        return content
    brace = content.find("{", start)
    depth = 0
    i = brace
    while i < len(content):
        if content[i] == "{":
            depth += 1
        elif content[i] == "}":
            depth -= 1
            if depth == 0:
                return content[:start] + content[i + 1 :]
        i += 1
    return content


def main() -> None:
    path = SRC / "persistence.rs"
    content = path.read_text(encoding="utf-8")

    content = content.replace(
        "    AgentMemoryRecord, AgentProviderBindingRecord,",
        "    AgentCompositionSlotKind, AgentCompositionSlotRecord, AgentCompositionTargetModule,\n"
        "    AgentProviderBindingRecord,",
    )

    content = re.sub(
        r"pub const SQL_INSERT_AGENT_KNOWLEDGE_BASE:.*?pub const SQL_SELECT_AGENT_KNOWLEDGE_BASE: &str =\s*",
        "",
        content,
        flags=re.S,
    )
    content = content.replace("a_agent_deployment", "ai_agent_deployment")

    content = re.sub(
        r"#\[derive\(Debug, Clone, PartialEq, Eq\)\]\s*pub struct AgentKnowledgeBaseRow:.*?^}\n\nimpl AgentKnowledgeBaseRow",
        "",
        content,
        count=1,
        flags=re.M | re.S,
    )
    content = re.sub(
        r"#\[derive\(Debug, Clone, PartialEq, Eq\)\]\s*pub struct AgentKnowledgeSourceRow:.*?validate_capabilities\(record\.capabilities\.as_slice\(\), \"capabilities\"\)\s*}\s*",
        "",
        content,
        count=1,
        flags=re.S,
    )

    composition_row = (TEMPLATES / "persistence_composition_row.rs.snippet").read_text(encoding="utf-8")
    anchor = "        validate_mcp_server_storage_contract(&record)?;\n        Ok(record)\n    }\n}\n"
    if anchor in content and "pub struct AgentCompositionSlotRow" not in content[: content.find(anchor) + len(anchor) + 200]:
        content = content.replace(anchor, anchor + "\n" + composition_row + "\n")

    for sig in [
        "fn validate_memory_store_storage_contract",
        "fn validate_memory_profile_storage_contract",
        "fn validate_memory_binding_storage_contract",
        "fn validate_memory_namespace_storage_contract",
        "fn validate_memory_record_storage_contract",
        "fn validate_memory_source_storage_contract",
        "fn validate_memory_relation_storage_contract",
        "fn validate_memory_retrieval_index_storage_contract",
        "fn validate_knowledge_base_storage_contract",
        "fn validate_knowledge_source_storage_contract",
        "fn validate_knowledge_document_storage_contract",
        "fn validate_knowledge_chunk_storage_contract",
        "fn validate_knowledge_index_storage_contract",
        "fn validate_knowledge_binding_storage_contract",
        "fn validate_knowledge_sync_job_storage_contract",
        "fn validate_memory_index_kinds",
        "fn validate_knowledge_index_kinds",
    ]:
        content = remove_fn_block(content, sig)

    trait_start = content.find("    fn insert_knowledge_base_row(")
    trait_comp = content.find("    fn insert_composition_slot_row(", trait_start)
    if trait_start != -1 and trait_comp != -1:
        content = content[:trait_start] + content[trait_comp:]

    repo_start = content.find("    fn insert_knowledge_base(&mut self, record: )")
    repo_comp = content.find("    fn insert_composition_slot(&mut self, record: AgentCompositionSlotRecord)", repo_start)
    repo_end = content.find("\n}\n\npub trait PostgresAuditAdapter", repo_comp)
    if repo_start != -1 and repo_comp != -1 and repo_end != -1:
        content = content[:repo_start] + content[repo_comp:repo_end] + content[repo_end:]

    audit_start = content.find("pub trait PostgresAuditAdapter {")
    audit_end = content.find("#[cfg(feature = \"postgres-sync\")]\npub const AGENTS_MANAGED_STORE_DATABASE_SERVICE", audit_start)
    if audit_start != -1 and audit_end != -1:
        content = content[:audit_start] + POSTGRES_AUDIT_TRAIT + "\n" + content[audit_end:]

    mcp_start = content.find("    fn list_mcp_server_rows(&self, query: &AgentMarketplaceListQuery)")
    mcp_end = content.find("\n#[cfg(feature = \"postgres-sync\")]\nimpl PostgresAuditAdapter for SyncPostgresAdapter", mcp_start)
    pg_comp = (TEMPLATES / "persistence_postgres_composition.rs.snippet").read_text(encoding="utf-8")
    if mcp_start != -1 and mcp_end != -1:
        content = content[:mcp_start] + LIST_MCP_SERVER_ROWS + "\n" + pg_comp + "\n}\n\n" + content[mcp_end:]

    if "struct MarketplaceRecordView" not in content:
        uuid_anchor = "fn build_agent_provider_binding_uuid("
        content = content.replace(uuid_anchor, MARKETPLACE_HELPERS + "\n" + uuid_anchor)

    for prefix in [
        "build_agent_memory_",
        "build_agent_knowledge_",
    ]:
        content = re.sub(
            rf"fn {prefix}[a-z_]+\([^\)]*\) -> String \{{\n(?:    [^\n]*\n)*?\}}\n\n",
            "",
            content,
        )

    content = re.sub(
        r"\}#\[derive\(Debug, Clone, PartialEq, Eq\)\]\s*pub struct AgentCompositionSlotRow:.*",
        "}\n",
        content,
        flags=re.S,
    )

    content = re.sub(
        r"    fn postgres_knowledge.*?^\}\n",
        "",
        content,
        flags=re.M | re.S,
    )
    content = re.sub(
        r"    fn postgres_memory.*?^\}\n",
        "",
        content,
        flags=re.M | re.S,
    )
    content = re.sub(
        r"    fn managed_store_schema_exposes_knowledge.*?^\}\n",
        "",
        content,
        flags=re.M | re.S,
    )

    broken_header = content.find("pub const SQL_INSERT_AGENT_KNOWLEDGE_BASE")
    if broken_header != -1:
        struct_end = content.find("impl AgentBusinessRow", broken_header)
        if struct_end != -1:
            content = content[:broken_header] + AGENT_BUSINESS_ROW + "\n\n" + content[struct_end:]

    path.write_text(content, encoding="utf-8")
    print(f"updated {path.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
