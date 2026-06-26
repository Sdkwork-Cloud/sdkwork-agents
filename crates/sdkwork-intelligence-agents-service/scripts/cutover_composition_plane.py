#!/usr/bin/env python3
"""Cut over agents service from inline kb/mem to composition_slots + ai_* tables."""
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SRC = ROOT / "src"
TEMPLATES = Path(__file__).resolve().parent / "composition_templates"


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def write(path: Path, content: str) -> None:
    path.write_text(content, encoding="utf-8")
    print(f"updated {path.relative_to(ROOT)}")


def delete_line_range(content: str, start: int, end: int) -> str:
    lines = content.splitlines(keepends=True)
    return "".join(lines[:start - 1] + lines[end:])


def replace_table_names(sql_block: str) -> str:
    replacements = [
        ("a_agent_business", "ai_agent"),
        ("a_agent_provider_binding", "ai_agent_runtime_binding"),
        ("a_agent_business_audit_event", "ai_agent_audit_event"),
        ("agent_business_id", "agent_internal_id"),
        ("SQL_INSERT_AGENT_BUSINESS", "SQL_INSERT_AGENT"),
        ("SQL_UPDATE_AGENT_BUSINESS", "SQL_UPDATE_AGENT"),
        ("SQL_LIST_AGENT_BUSINESS", "SQL_LIST_AGENT"),
    ]
    for old, new in replacements:
        sql_block = sql_block.replace(old, new)
    return sql_block


def strip_kb_mem_imports(content: str) -> str:
    patterns = [
        r"AgentKnowledge[A-Za-z]+Record,?\s*",
        r"AgentKnowledge[A-Za-z]+Kind,?\s*",
        r"AgentKnowledgeSearchResult,?\s*",
        r"AgentMemory[A-Za-z]+Record,?\s*",
        r"AgentMemory[A-Za-z]+Kind,?\s*",
        r"AgentMemoryStoreKind,?\s*",
        r"AgentKnowledgeSyncJobStatus,?\s*",
        r"AgentMemoryBindingScopeKind,?\s*",
        r"AgentMemoryIndexKind,?\s*",
        r"AgentMemoryNamespaceKind,?\s*",
        r"AgentMemoryRecordKind,?\s*",
        r"AgentMemoryRelationKind,?\s*",
        r"AgentMemorySourceKind,?\s*",
    ]
    for pat in patterns:
        content = re.sub(pat, "", content)
    content = re.sub(r",\s*,", ",", content)
    content = re.sub(r"\(\s*,", "(", content)
    content = re.sub(r",\s*\)", ")", content)
    return content


def process_persistence() -> None:
    path = SRC / "persistence.rs"
    content = read(path)
    content = delete_line_range(content, 35, 39)  # MAX_KNOWLEDGE constants
    content = delete_line_range(content, 74, 181)  # kb/mem SQL constants
    content = replace_table_names(content)
    content = delete_line_range(content, 516, 1647)  # kb/mem row structs
    # trait adapter kb/mem methods
    content = delete_line_range(content, 2641, 2807)
    # AgentRepository impl kb/mem on PostgresAgentRepository
    content = delete_line_range(content, 2928, 3334)
    # PostgresAgentRepositoryAdapter impl kb/mem
    content = delete_line_range(content, 3898, 5226)
    insert_after_mcp_sql = content.find("pub const SQL_LIST_AGENT_MCP_SERVERS")
    mcp_end = content.find("\n", content.find("FROM a_agent_mcp_server", insert_after_mcp_sql))
    composition_sql = read(TEMPLATES / "persistence_composition_sql.rs.snippet")
    content = content[:mcp_end + 1] + "\n" + composition_sql + content[mcp_end + 1:]
    # inject row struct after AgentMcpServerRow impl ends (~line 522 before delete)
    row_anchor = content.find("fn parse_implementation_kind")
    composition_row = read(TEMPLATES / "persistence_composition_row.rs.snippet")
    content = content[:row_anchor] + composition_row + "\n" + content[row_anchor:]
    trait_anchor = content.find("    fn list_mcp_server_rows")
    trait_end = content.find("}", trait_anchor)
    trait_methods = read(TEMPLATES / "persistence_trait_composition.rs.snippet")
    content = content[:trait_end] + trait_methods + content[trait_end:]
    repo_anchor = content.find("    fn list_mcp_servers(")
    repo_end = content.find("\n}\n\npub trait PostgresAuditAdapter", repo_anchor)
    repo_methods = read(TEMPLATES / "persistence_repo_composition.rs.snippet")
    content = content[:repo_end] + repo_methods + content[repo_end:]
    pg_anchor = content.find("    fn list_mcp_server_rows(")
    pg_end = content.find("\n}\n\n#[cfg(feature = \"postgres-sync\")]", pg_anchor)
    pg_methods = read(TEMPLATES / "persistence_postgres_composition.rs.snippet")
    content = content[:pg_end] + pg_methods + content[pg_end:]
    content = strip_kb_mem_imports(content)
    write(path, content)


def process_ports() -> None:
    path = SRC / "ports.rs"
    content = read(path)
    start = content.find("    fn insert_knowledge_base(")
    end = content.find("\n}\n\npub trait AgentAuditSink", start)
    composition = read(TEMPLATES / "ports_composition.rs.snippet")
    content = content[:start] + composition + content[end:]
    content = strip_kb_mem_imports(content)
    write(path, content)


def process_application() -> None:
    path = SRC / "application.rs"
    content = read(path)
    content = delete_line_range(content, 27, 34)  # MAX_KNOWLEDGE constants
    cmd_start = content.find("#[derive(Debug, Clone, PartialEq, Eq)]\npub struct AgentKnowledgeBaseCreateCommand")
    if cmd_start == -1:
        cmd_start = content.find("pub struct AgentKnowledgeBaseCreateCommand")
    agentservice = content.find("pub struct AgentsService<R, A, P>")
    content = content[:cmd_start] + read(TEMPLATES / "application_commands.rs.snippet") + content[agentservice:]
    method_start = content.find("    pub fn create_knowledge_base(")
    method_end = content.find("    pub fn update_agent(", method_start)
    methods = read(TEMPLATES / "application_methods.rs.snippet")
    content = content[:method_start] + methods + content[method_end:]
    content = strip_kb_mem_imports(content)
    # add composition imports
    content = content.replace(
        "AgentProviderBindingRecord, AgentRuntimeExecutionOperation",
        "AgentCompositionSlotRecord, AgentCompositionSlotKind, AgentCompositionTargetModule, "
        "AgentProviderBindingRecord, AgentRuntimeExecutionOperation",
    )
    write(path, content)


def process_infrastructure() -> None:
    path = SRC / "infrastructure.rs"
    content = read(path)
    start = content.find("    fn insert_knowledge_base(")
    end = content.find("\n}\n\nimpl Default for InMemoryAgentAuditSink", start)
    if end == -1:
        end = content.find("\n#[cfg(test)]", start)
    composition = read(TEMPLATES / "infrastructure_composition.rs.snippet")
    content = content[:start] + composition + content[end:]
    # remove kb/mem fields from struct
    for field in [
        "knowledge_bases", "knowledge_sources", "knowledge_documents", "knowledge_chunks",
        "knowledge_indexes", "knowledge_bindings", "knowledge_sync_jobs",
        "memory_stores", "memory_profiles", "memory_bindings", "memory_namespaces",
        "memory_records", "memory_sources", "memory_relations", "memory_retrieval_indexes",
    ]:
        content = re.sub(
            rf"    {field}: Vec<[^>]+>,\n",
            "",
            content,
        )
        content = re.sub(
            rf"            {field}: Vec::new\(\),\n",
            "",
            content,
        )
    if "composition_slots: Vec<AgentCompositionSlotRecord>" not in content:
        content = content.replace(
            "mcp_servers: Vec<AgentMcpServerRecord>,",
            "mcp_servers: Vec<AgentMcpServerRecord>,\n    composition_slots: Vec<AgentCompositionSlotRecord>,",
        )
        content = content.replace(
            "mcp_servers: Vec::new(),",
            "mcp_servers: Vec::new(),\n            composition_slots: Vec::new(),",
        )
    content = strip_kb_mem_imports(content)
    content = content.replace(
        "AgentMcpServerRecord, AgentProviderBindingRecord,",
        "AgentCompositionSlotRecord, AgentMcpServerRecord, AgentProviderBindingRecord,",
    )
    # strip kb/mem tests at bottom - keep only through provider binding test
    test_kb = content.find("    fn sample_knowledge_binding(")
    if test_kb != -1:
        content = content[:test_kb] + "}\n"
    write(path, content)


def process_api() -> None:
    path = SRC / "api.rs"
    content = read(path)
    composition_ops = read(TEMPLATES / "api_composition_operations.rs.snippet")
    # replace kb/mem block in each API surface (open, app, backend)
    for marker in [
        "operation_id: \"agents.promptOptimizations.create\"",
    ]:
        idx = content.find(marker)
        if idx == -1:
            continue
        close = content.find("},", idx)
        close = content.find("},", close + 1)  # end of ApiOperation
        next_start = content.find("ApiOperation {", close)
        # find end of kb/mem section - next agents. or mcp or closing ];
        end_markers = [
            "operation_id: \"mcpServers.list\"",
            "operation_id: \"agents.list\"",
        ]
        end = len(content)
        for em in end_markers:
            pos = content.find(em, close)
            if pos != -1 and pos < end:
                end = content.rfind("ApiOperation {", close, pos)
        content = content[:close + 2] + "\n" + composition_ops + content[end:]
    # remove kb/mem from test helper sections
    content = re.sub(
        r'"operationId: knowledgeBases\.[^"]+",?\s*',
        "",
        content,
    )
    content = re.sub(
        r'"operationId: memoryStores\.[^"]+",?\s*',
        "",
        content,
    )
    content = re.sub(
        r'\("GET", "/ai/knowledge_bases[^"]*", "knowledgeBases[^"]*"\),?\s*',
        "",
        content,
    )
    content = re.sub(
        r'\("POST", "/ai/memory_stores[^"]*", "memoryStores[^"]*"\),?\s*',
        "",
        content,
    )
    write(path, content)


def process_http_build_routes(content: str) -> str:
    content = re.sub(
        r"add_app_memory_routes\(\s*add_app_knowledge_routes\(",
        "",
        content,
    )
    content = re.sub(
        r"add_memory_routes\(\s*add_knowledge_routes\(",
        "",
        content,
        count=3,
    )
    content = re.sub(
        r",\s*\"/app/v3/api\",\s*\),\s*\"/app/v3/api\",\s*\)",
        "",
        content,
    )
    content = re.sub(
        r",\s*\"/agent/v3/api\",\s*\),\s*\"/agent/v3/api\",\s*\)",
        "",
        content,
        count=2,
    )
    content = re.sub(
        r",\s*\"/backend/v3/api\",\s*\),\s*\"/backend/v3/api\",\s*\)",
        "",
        content,
    )
    route_snippet = read(TEMPLATES / "http_composition_routes.rs.snippet")
    anchor = ".route(\n                    \"/app/v3/api/ai/agents/{agentId}/prompt_optimizations\","
    pos = content.find(anchor)
    if pos != -1:
        insert_at = content.find("),", pos) + 2
        content = content[:insert_at] + route_snippet + content[insert_at:]
    anchor2 = ".route(\n                    \"/agent/v3/api/ai/agents/{agentId}/prompt_optimizations\","
    pos2 = content.find(anchor2)
    if pos2 != -1:
        insert_at2 = content.find("),", pos2) + 2
        if route_snippet not in content[insert_at2:insert_at2 + 200]:
            content = content[:insert_at2] + route_snippet.replace("/app/", "/agent/").replace(
                "app_", "backend_"
            ) + content[insert_at2:]
    anchor3 = ".route(\n                    \"/backend/v3/api/ai/agents/{agentId}/prompt_optimizations\","
    pos3 = content.find(anchor3)
    if pos3 != -1:
        insert_at3 = content.find("),", pos3) + 2
        content = content[:insert_at3] + route_snippet.replace("/app/", "/backend/").replace(
            "app_", "backend_"
        ) + content[insert_at3:]
    # delete route helper functions
    for fn in [
        "fn add_knowledge_routes",
        "fn add_memory_routes",
        "fn add_app_knowledge_routes",
        "fn add_app_memory_routes",
    ]:
        start = content.find(fn)
        if start == -1:
            continue
        brace = content.find("{", start)
        depth = 0
        i = brace
        while i < len(content):
            if content[i] == "{":
                depth += 1
            elif content[i] == "}":
                depth -= 1
                if depth == 0:
                    content = content[:start] + content[i + 1:]
                    break
            i += 1
    return content


def process_http() -> None:
    path = SRC / "http.rs"
    content = read(path)
    content = process_http_build_routes(content)
    struct_anchor = content.find("struct TenantAgentBindingPathParams")
    if struct_anchor != -1:
        struct_end = content.find("\n\n", struct_anchor)
        structs = read(TEMPLATES / "http_composition_structs.rs.snippet")
        if "TenantAgentSlotPathParams" not in content:
            content = content[:struct_end] + "\n" + structs + content[struct_end:]
    dyn_start = content.find("    fn insert_knowledge_base(")
    dyn_end = content.find("\n}\n\nimpl AgentAuditSink for DynAgentAuditSink", dyn_start)
    if dyn_start != -1 and dyn_end != -1:
        dyn_methods = read(TEMPLATES / "http_dyn_repository_composition.rs.snippet")
        content = content[:dyn_start] + dyn_methods + content[dyn_end:]
    # delete app kb handlers through execute_list_knowledge_bases
    start = content.find("async fn app_list_knowledge_bases(")
    end = content.find("async fn execute_list_knowledge_bases(", start)
    if start != -1 and end != -1:
        content = content[:start] + read(TEMPLATES / "http_composition_handlers.rs.snippet") + content[end:]
    # delete execute kb/mem through execute_list_provider_bindings
    start2 = content.find("async fn execute_list_knowledge_bases(")
    end2 = content.find("async fn execute_list_provider_bindings(", start2)
    if start2 != -1 and end2 != -1:
        content = content[:start2] + read(TEMPLATES / "http_composition_execute.rs.snippet") + content[end2:]
    # open/backend kb handlers between cancel and memory create
    start3 = content.find("async fn list_knowledge_bases(")
    end3 = content.find("async fn app_create_memory_store(", start3)
    if start3 != -1 and end3 != -1:
        content = content[:start3] + content[end3:]
    start4 = content.find("async fn app_create_memory_store(")
    end4 = content.find("async fn execute_list_knowledge_bases(", start4)
    if start4 != -1 and end4 != -1 and start4 < end4:
        content = content[:start4] + content[end4:]
    content = content.replace(
        "    \"deployment_created\",\n];",
        "    \"deployment_created\",\n    \"composition_slot_created\",\n    \"composition_slot_updated\",\n    \"composition_slot_deleted\",\n];",
    )
    content = strip_kb_mem_imports(content)
    # fix application imports
    content = re.sub(
        r"use crate::application::\{[^}]+\};",
        "use crate::application::AgentsService;",
        content,
        count=1,
    )
    write(path, content)


def process_domain() -> None:
    path = SRC / "domain.rs"
    content = read(path)
    if "AgentCompositionSlotRecord" in content:
        return
    anchor = content.find("pub struct AgentMcpServerRecord")
    snippet = read(TEMPLATES / "domain_composition.rs.snippet")
    content = content[:anchor] + snippet + "\n" + content[anchor:]
    # add audit actions before closing of enum - after McpServerRestored
    content = content.replace(
        "    McpServerRestored,\n    MemoryStoreCreated,",
        "    McpServerRestored,\n    CompositionSlotCreated,\n    CompositionSlotUpdated,\n    CompositionSlotDeleted,\n    MemoryStoreCreated,",
    )
    content = content.replace(
        "            Self::McpServerRestored => \"agent.business.mcp.restored\",\n            Self::MemoryStoreCreated =>",
        "            Self::McpServerRestored => \"agent.business.mcp.restored\",\n            Self::CompositionSlotCreated => \"agent.business.composition_slot.created\",\n            Self::CompositionSlotUpdated => \"agent.business.composition_slot.updated\",\n            Self::CompositionSlotDeleted => \"agent.business.composition_slot.deleted\",\n            Self::MemoryStoreCreated =>",
    )
    content = content.replace(
        "            Self::McpServerRestored => \"mcp_restored\",\n            Self::MemoryStoreCreated =>",
        "            Self::McpServerRestored => \"mcp_restored\",\n            Self::CompositionSlotCreated => \"composition_slot_created\",\n            Self::CompositionSlotUpdated => \"composition_slot_updated\",\n            Self::CompositionSlotDeleted => \"composition_slot_deleted\",\n            Self::MemoryStoreCreated =>",
    )
    write(path, content)


def process_dto() -> None:
    path = SRC / "dto.rs"
    content = read(path)
    kb_start = content.find("pub struct AgentKnowledgeBaseCreateRequestDto")
    agent_mgmt_end = content.rfind("}", 0, kb_start)
    composition = read(TEMPLATES / "dto_composition.rs.snippet")
    # remove kb/mem dtos to end of file tests
    test_start = content.find("#[cfg(test)]", kb_start)
    if test_start == -1:
        test_start = len(content)
    content = content[:kb_start] + composition + "\n" + content[test_start:]
    write(path, content)


def process_lib() -> None:
    path = SRC / "lib.rs"
    content = read(path)
    composition_exports = read(TEMPLATES / "lib_exports.rs.snippet")
    # remove kb/mem from pub use blocks - replace application block
    app_start = content.find("pub use application::{")
    app_end = content.find("};", app_start) + 2
    content = content[:app_start] + composition_exports + content[app_end:]
    # domain exports - strip kb/mem
    dom_start = content.find("pub use domain::{")
    dom_end = content.find("};", dom_start) + 2
    dom_block = content[dom_start:dom_end]
    dom_block = strip_kb_mem_imports(dom_block)
    dom_block = dom_block.replace(
        "AgentMcpAuthKind, AgentMcpServerRecord",
        "AgentCompositionSlotKind, AgentCompositionSlotRecord, AgentCompositionTargetModule, "
        "AgentMcpAuthKind, AgentMcpServerRecord",
    )
    content = content[:dom_start] + dom_block + content[dom_end:]
    dto_start = content.find("pub use dto::{")
    dto_end = content.find("};", dto_start) + 2
    dto_block = content[dto_start:dto_end]
    dto_block = strip_kb_mem_imports(dto_block)
    dto_block = dto_block.replace(
        "ActivateAgentProviderBindingRequestDto, AgentDeploymentListResponseDto",
        "ActivateAgentProviderBindingRequestDto, AgentCompositionSlotCreateRequestDto, "
        "AgentCompositionSlotDeleteRequestDto, AgentCompositionSlotListResponseDto, "
        "AgentCompositionSlotRecordDto, AgentCompositionSlotResponseDto, "
        "AgentCompositionSlotUpdateRequestDto, AgentDeploymentListResponseDto",
    )
    content = content[:dto_start] + dto_block + content[dto_end:]
    write(path, content)


def process_tests() -> None:
    http_tests = ROOT / "tests" / "http_axum_contracts.rs"
    content = read(http_tests)
    kb_test = content.find("async fn app_knowledge_base_response_should_expose_document_count_projection")
    provider_test = content.find("async fn provider_bindings_and_deployments_should_work_over_http")
    if kb_test != -1 and provider_test != -1 and kb_test < provider_test:
        composition_test = read(TEMPLATES / "test_composition_http.rs.snippet")
        content = content[:kb_test] + composition_test + "\n\n" + content[provider_test:]
    pg_tests = ROOT / "tests" / "agent_postgres_sync_contracts.rs"
    write(pg_tests, read(TEMPLATES / "test_postgres_sync.rs.snippet"))
    write(http_tests, content)


def main() -> int:
    process_domain()
    process_dto()
    process_ports()
    process_application()
    process_persistence()
    process_infrastructure()
    process_api()
    process_http()
    process_lib()
    process_tests()
    print("cutover script completed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
