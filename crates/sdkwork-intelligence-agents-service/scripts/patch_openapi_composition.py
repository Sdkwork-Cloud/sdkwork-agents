#!/usr/bin/env python3
"""Patch agents OpenAPI: drop inline kb/mem surfaces, add composition_slots."""
from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OPENAPI_DIR = ROOT / "specs" / "openapi"

COMPOSITION_PATHS = '''
  /app/v3/api/ai/agents/{agentId}/composition_slots:
    get:
      tags: [ai]
      summary: List composition slots for one managed agent
      operationId: agents.compositionSlots.list
      security:
        - AuthToken: []
          AccessToken: []
      x-sdkwork-domain: ai
      x-sdkwork-resource: agents.compositionSlots
      x-sdkwork-permission: agent.business.composition_slot.list
      x-sdkwork-tenant-scope: tenant
      parameters:
        - $ref: '#/components/parameters/AgentIdPath'
      responses:
        '200':
          description: Managed agent composition slot list response
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/AgentCompositionSlotListResponse'
        '400':
          $ref: '#/components/responses/Problem'
        '403':
          $ref: '#/components/responses/Problem'
        '404':
          $ref: '#/components/responses/Problem'
        default:
          $ref: '#/components/responses/Problem'
    post:
      tags: [ai]
      summary: Create a composition slot for one managed agent
      operationId: agents.compositionSlots.create
      security:
        - AuthToken: []
          AccessToken: []
      x-sdkwork-domain: ai
      x-sdkwork-resource: agents.compositionSlots
      x-sdkwork-permission: agent.business.composition_slot.create
      x-sdkwork-tenant-scope: tenant
      x-sdkwork-audit-event: agent.business.composition_slot_created
      parameters:
        - $ref: '#/components/parameters/AgentIdPath'
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/CreateAgentCompositionSlotRequest'
      responses:
        '201':
          description: Created managed agent composition slot
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/AgentCompositionSlotResponse'
        '400':
          $ref: '#/components/responses/Problem'
        '403':
          $ref: '#/components/responses/Problem'
        '404':
          $ref: '#/components/responses/Problem'
        '409':
          $ref: '#/components/responses/Problem'
        default:
          $ref: '#/components/responses/Problem'
  /app/v3/api/ai/agents/{agentId}/composition_slots/{slotId}:
    get:
      tags: [ai]
      summary: Retrieve one managed agent composition slot
      operationId: agents.compositionSlots.retrieve
      security:
        - AuthToken: []
          AccessToken: []
      x-sdkwork-domain: ai
      x-sdkwork-resource: agents.compositionSlots
      x-sdkwork-permission: agent.business.composition_slot.retrieve
      x-sdkwork-tenant-scope: tenant
      parameters:
        - $ref: '#/components/parameters/AgentIdPath'
        - $ref: '#/components/parameters/SlotIdPath'
      responses:
        '200':
          description: Managed agent composition slot response
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/AgentCompositionSlotResponse'
        '400':
          $ref: '#/components/responses/Problem'
        '403':
          $ref: '#/components/responses/Problem'
        '404':
          $ref: '#/components/responses/Problem'
        default:
          $ref: '#/components/responses/Problem'
    patch:
      tags: [ai]
      summary: Update one managed agent composition slot
      operationId: agents.compositionSlots.update
      security:
        - AuthToken: []
          AccessToken: []
      x-sdkwork-domain: ai
      x-sdkwork-resource: agents.compositionSlots
      x-sdkwork-permission: agent.business.composition_slot.update
      x-sdkwork-tenant-scope: tenant
      x-sdkwork-audit-event: agent.business.composition_slot_updated
      parameters:
        - $ref: '#/components/parameters/AgentIdPath'
        - $ref: '#/components/parameters/SlotIdPath'
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/UpdateAgentCompositionSlotRequest'
      responses:
        '200':
          description: Updated managed agent composition slot response
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/AgentCompositionSlotResponse'
        '400':
          $ref: '#/components/responses/Problem'
        '403':
          $ref: '#/components/responses/Problem'
        '404':
          $ref: '#/components/responses/Problem'
        default:
          $ref: '#/components/responses/Problem'
    delete:
      tags: [ai]
      summary: Delete one managed agent composition slot
      operationId: agents.compositionSlots.delete
      security:
        - AuthToken: []
          AccessToken: []
      x-sdkwork-domain: ai
      x-sdkwork-resource: agents.compositionSlots
      x-sdkwork-permission: agent.business.composition_slot.delete
      x-sdkwork-tenant-scope: tenant
      x-sdkwork-audit-event: agent.business.composition_slot_deleted
      parameters:
        - $ref: '#/components/parameters/AgentIdPath'
        - $ref: '#/components/parameters/SlotIdPath'
        - $ref: '#/components/parameters/ExpectedVersion'
        - $ref: '#/components/parameters/RequestedAt'
      responses:
        '200':
          description: Deleted managed agent composition slot response
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/AgentCompositionSlotResponse'
        '400':
          $ref: '#/components/responses/Problem'
        '403':
          $ref: '#/components/responses/Problem'
        '404':
          $ref: '#/components/responses/Problem'
        default:
          $ref: '#/components/responses/Problem'
'''

COMPOSITION_SCHEMAS = '''
    AgentCompositionSlotKind:
      type: string
      enum: [memory, knowledge, skill, prompt, drive, tool]
    AgentCompositionTargetModule:
      type: string
      enum: [memory, knowledgebase, skills, prompts, drive]
    AgentCompositionSlotRecord:
      type: object
      additionalProperties: false
      required:
        - id
        - tenantId
        - organizationId
        - agentId
        - slotId
        - slotKind
        - targetModule
        - targetRef
        - priority
        - enabled
        - policyJson
        - status
        - version
        - createdAt
        - updatedAt
      properties:
        id:
          $ref: '#/components/schemas/Int64String'
        tenantId:
          $ref: '#/components/schemas/Int64String'
        organizationId:
          $ref: '#/components/schemas/Int64String'
        agentId:
          type: string
          minLength: 1
          maxLength: 128
          pattern: '^agent\\.[a-z0-9_-]+(\\.[a-z0-9_-]+)*$'
        slotId:
          type: string
          minLength: 1
          maxLength: 128
          pattern: '^slot\\.[a-z0-9_-]+(\\.[a-z0-9_-]+)*$'
        slotKind:
          $ref: '#/components/schemas/AgentCompositionSlotKind'
        targetModule:
          $ref: '#/components/schemas/AgentCompositionTargetModule'
        targetRef:
          type: string
          minLength: 1
          maxLength: 256
        targetVersionRef:
          type: [string, 'null']
          maxLength: 128
        priority:
          type: integer
          format: int32
        enabled:
          type: boolean
        policyJson:
          type: string
        status:
          $ref: '#/components/schemas/AgentStatus'
        version:
          $ref: '#/components/schemas/Int64String'
        createdAt:
          type: string
          format: date-time
        updatedAt:
          type: string
          format: date-time
        deletedAt:
          type: [string, 'null']
          format: date-time
    AgentCompositionSlotResponse:
      type: object
      additionalProperties: false
      required: [data]
      properties:
        data:
          $ref: '#/components/schemas/AgentCompositionSlotRecord'
        requestId:
          type: string
    AgentCompositionSlotListResponse:
      type: object
      additionalProperties: false
      required: [data]
      properties:
        data:
          type: object
          additionalProperties: false
          required: [items]
          properties:
            items:
              type: array
              items:
                $ref: '#/components/schemas/AgentCompositionSlotRecord'
        requestId:
          type: string
    AgentCompositionSlotCreateData:
      type: object
      additionalProperties: false
      required: [tenantId, organizationId, slotId, slotKind, targetModule, targetRef]
      properties:
        tenantId:
          $ref: '#/components/schemas/Int64String'
        organizationId:
          $ref: '#/components/schemas/Int64String'
        slotId:
          type: string
          minLength: 1
          maxLength: 128
          pattern: '^slot\\.[a-z0-9_-]+(\\.[a-z0-9_-]+)*$'
        slotKind:
          $ref: '#/components/schemas/AgentCompositionSlotKind'
        targetModule:
          $ref: '#/components/schemas/AgentCompositionTargetModule'
        targetRef:
          type: string
          minLength: 1
          maxLength: 256
        targetVersionRef:
          type: [string, 'null']
          maxLength: 128
        priority:
          type: integer
          format: int32
          default: 0
        enabled:
          type: boolean
          default: true
        policyJson:
          type: string
          default: '{}'
    CreateAgentCompositionSlotRequest:
      type: object
      additionalProperties: false
      required: [data, requestedAt]
      properties:
        data:
          $ref: '#/components/schemas/AgentCompositionSlotCreateData'
        requestedAt:
          type: string
          format: date-time
    UpdateAgentCompositionSlotData:
      type: object
      additionalProperties: false
      required: [tenantId]
      properties:
        tenantId:
          $ref: '#/components/schemas/Int64String'
        expectedVersion:
          $ref: '#/components/schemas/Int64String'
        slotKind:
          $ref: '#/components/schemas/AgentCompositionSlotKind'
        targetModule:
          $ref: '#/components/schemas/AgentCompositionTargetModule'
        targetRef:
          type: string
          minLength: 1
          maxLength: 256
        targetVersionRef:
          type: [string, 'null']
          maxLength: 128
        priority:
          type: integer
          format: int32
        enabled:
          type: boolean
        policyJson:
          type: string
    UpdateAgentCompositionSlotRequest:
      type: object
      additionalProperties: false
      required: [data, requestedAt]
      properties:
        data:
          $ref: '#/components/schemas/UpdateAgentCompositionSlotData'
        requestedAt:
          type: string
          format: date-time
'''

SLOT_PARAM = '''
    SlotIdPath:
      name: slotId
      in: path
      required: true
      schema:
        type: string
        minLength: 1
        maxLength: 128
        pattern: '^slot\\.[a-z0-9_-]+(\\.[a-z0-9_-]+)*$'
'''


def drop_yaml_blocks(text: str, name_predicate) -> str:
    lines = text.splitlines(keepends=True)
    out: list[str] = []
    i = 0
    while i < len(lines):
        line = lines[i]
        m = re.match(r"^    ([A-Za-z0-9]+):\s*$", line)
        if m and name_predicate(m.group(1)):
            i += 1
            while i < len(lines):
                nxt = lines[i]
                if re.match(r"^    [A-Za-z0-9]+:\s*$", nxt) and not nxt.startswith("      "):
                    break
                i += 1
            continue
        out.append(line)
        i += 1
    return "".join(out)


def is_kb_mem_name(name: str) -> bool:
    return name.startswith(("Knowledge", "Memory")) or "Knowledge" in name or "Memory" in name


def patch_file(path: Path, api_prefix: str) -> None:
    text = path.read_text(encoding="utf-8")
    kb_marker = f"  {api_prefix}/ai/knowledge_bases:"
    comp_marker = f"  {api_prefix}/ai/agents/{{agentId}}/composition_slots:"
    if comp_marker not in text:
        kb_start = text.find(kb_marker)
        comp_start = text.find("components:")
        if kb_start == -1:
            raise SystemExit(f"no knowledge_bases path in {path.name}")
        composition = COMPOSITION_PATHS.replace("/app/v3/api", api_prefix)
        text = text[:kb_start] + composition + text[comp_start:]

    text = drop_yaml_blocks(text, is_kb_mem_name)

    if "SlotIdPath:" not in text:
        text = text.replace("    BindingIdPath:", SLOT_PARAM + "    BindingIdPath:", 1)

    if "AgentCompositionSlotRecord:" not in text:
        text = text.replace(
            "    CreateAgentRequest:",
            COMPOSITION_SCHEMAS + "    CreateAgentRequest:",
            1,
        )

    path.write_text(text, encoding="utf-8")
    print(f"patched {path.name}")


def main() -> None:
    patch_file(OPENAPI_DIR / "agents-app-api.openapi.yaml", "/app/v3/api")
    patch_file(OPENAPI_DIR / "agents-backend-api.openapi.yaml", "/backend/v3/api")
    patch_file(OPENAPI_DIR / "agents-open-api.openapi.yaml", "/agent/v3/api")


if __name__ == "__main__":
    main()
