#!/usr/bin/env python3
"""Finish composition-plane cutover after partial run of cutover_composition_plane.py."""
from __future__ import annotations

import re
import sys
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parent.parent
SRC = ROOT / "src"
TEMPLATES = Path(__file__).resolve().parent / "composition_templates"

# Reuse cutover helpers
sys.path.insert(0, str(Path(__file__).resolve().parent))
from cutover_composition_plane import (  # noqa: E402
    process_http,
    process_lib,
    process_tests,
    read,
    write,
)


def fix_application() -> None:
    path = SRC / "application.rs"
    content = read(path)
    content = re.sub(r"AgentMemoryRecord,?\s*", "", content)

    start = content.find("    fn get_active_knowledge_base(")
    end = content.find("    fn emit_audit_event(", start)
    if start != -1 and end != -1:
        content = content[:start] + content[end:]

    for fn in ["    fn emit_memory_audit_event(", "    fn emit_knowledge_audit_event("]:
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
                    content = content[:start] + content[i + 1 :]
                    break
            i += 1

    start = content.find("fn knowledge_sync_job_audit_sequence(")
    end = content.find("fn validate_agent_id(", start)
    if start != -1 and end != -1:
        content = content[:start] + content[end:]

    write(path, content)


def openapi_operations(prefix: str, yaml_path: Path) -> str:
    doc = yaml.safe_load(yaml_path.read_text(encoding="utf-8"))
    lines = []
    for path in sorted(doc.get("paths", {}).keys()):
        methods = doc["paths"][path]
        for method in sorted(methods.keys()):
            if method in {"parameters"} or method.startswith("x-"):
                continue
            spec = methods[method]
            op_id = spec.get("operationId", "")
            tag = (spec.get("tags") or ["ai"])[0]
            lines.append(
                "    ApiOperation {\n"
                f'        method: "{method.upper()}",\n'
                f'        path: "{path}",\n'
                f'        tag: "{tag}",\n'
                f'        operation_id: "{op_id}",\n'
                "    },"
            )
    const_name = {
        "/agent/v3/api": "AGENT_OPEN_API_OPERATIONS",
        "/app/v3/api": "AGENT_APP_API_OPERATIONS",
        "/backend/v3/api": "AGENT_BACKEND_API_OPERATIONS",
    }[prefix]
    return f"pub const {const_name}: &[ApiOperation] = &[\n" + "\n".join(lines) + "\n];"


def fix_api() -> None:
    openapi_dir = ROOT / "specs" / "openapi"
    header = """#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ApiOperation {
    pub method: &'static str,
    pub path: &'static str,
    pub tag: &'static str,
    pub operation_id: &'static str,
}

pub const AGENT_OPEN_API_PREFIX: &str = "/agent/v3/api";
pub const AGENT_APP_API_PREFIX: &str = "/app/v3/api";
pub const AGENT_BACKEND_API_PREFIX: &str = "/backend/v3/api";

"""
    operations = "\n\n".join(
        [
            openapi_operations(
                "/agent/v3/api", openapi_dir / "agents-open-api.openapi.yaml"
            ),
            openapi_operations(
                "/app/v3/api", openapi_dir / "agents-app-api.openapi.yaml"
            ),
            openapi_operations(
                "/backend/v3/api", openapi_dir / "agents-backend-api.openapi.yaml"
            ),
        ]
    )
    tests = read(TEMPLATES / "api_tests_composition.rs.snippet")
    write(SRC / "api.rs", header + operations + "\n\n" + tests)


def main() -> int:
    fix_application()
    fix_api()
    process_http()
    process_lib()
    process_tests()
    print("finish composition cutover completed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
