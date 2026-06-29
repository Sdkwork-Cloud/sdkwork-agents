import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const base = path.resolve(__dirname, '../specs/openapi');

const openPath = path.join(base, 'agents-open-api.openapi.yaml');
const open = fs.readFileSync(openPath, 'utf8');
const lines = open.split(/\r?\n/);
const start = lines.findIndex((line) =>
  line.startsWith('  /agent/v3/api/ai/agents/{agentId}/sessions:'),
);
const end = lines.findIndex(
  (line, index) =>
    index > start &&
    line.startsWith('  /agent/v3/api/ai/agents/{agentId}/composition_slots:'),
);
if (start < 0 || end < 0) {
  throw new Error('open-api session path block not found');
}
const sessionPathsBlock = lines.slice(start, end).join('\n');

function projectPaths(block, prefix) {
  return block.replaceAll('/agent/v3/api', prefix);
}

const sharedSchemaNames = [
  'AgentSessionRecord',
  'AgentSessionResponse',
  'AgentSessionListResponse',
  'CreateAgentSessionRequest',
  'CloseAgentSessionRequest',
  'AgentMessageRecord',
  'AgentMessageResponse',
  'AgentMessageListResponse',
  'SendAgentChatMessageRequest',
  'AgentChatCompletionResponse',
];

function extractSchemaBlock(text, name) {
  const marker = `    ${name}:`;
  const index = text.indexOf(marker);
  if (index < 0) {
    return null;
  }
  const slice = text.slice(index).split(/\r?\n/);
  const chunk = [slice[0]];
  for (let i = 1; i < slice.length; i += 1) {
    const line = slice[i];
    if (/^    [A-Za-z0-9_]+:/.test(line) && !line.startsWith('      ')) {
      break;
    }
    chunk.push(line);
  }
  return chunk.join('\n');
}

function extractSchemas(text, names) {
  return names
    .map((name) => extractSchemaBlock(text, name))
    .filter(Boolean)
    .join('\n');
}

const sharedSchemas = extractSchemas(open, sharedSchemaNames);

const surfaceSchemas = `    AppCreateAgentSessionRequest:
      type: object
      additionalProperties: false
      required: [requestedAt]
      properties:
        sessionId:
          type: string
          pattern: '^session\\.[a-z0-9_-]+(\\.[a-z0-9_-]+)*$'
        title:
          type: string
        providerBindingId:
          type: string
        modelId:
          type: string
        metadataJson:
          type: string
        requestedAt:
          type: string
          format: date-time
    AppSendAgentChatMessageRequest:
      type: object
      additionalProperties: false
      required: [content, requestedAt]
      properties:
        content:
          type: string
          minLength: 1
        contentType:
          type: string
        metadataJson:
          type: string
        modelId:
          type: string
        requestedAt:
          type: string
          format: date-time
    AppCloseAgentSessionRequest:
      type: object
      additionalProperties: false
      required: [requestedAt]
      properties:
        expectedVersion:
          $ref: '#/components/schemas/Int64String'
        requestedAt:
          type: string
          format: date-time
    ArchiveAgentSessionRequest:
      type: object
      additionalProperties: false
      required: [tenantId, requestedAt]
      properties:
        tenantId:
          $ref: '#/components/schemas/Int64String'
        expectedVersion:
          $ref: '#/components/schemas/Int64String'
        requestedAt:
          type: string
          format: date-time`;

const chatParameterBlock = `    SessionIdPath:
      name: sessionId
      in: path
      required: true
      schema:
        type: string
        minLength: 1
        maxLength: 128
        pattern: '^session\\.[a-z0-9_-]+(\\.[a-z0-9_-]+)*$'
    MessageIdPath:
      name: messageId
      in: path
      required: true
      schema:
        type: string
        minLength: 1
        maxLength: 128
        pattern: '^msg\\.[a-z0-9_-]+(\\.[a-z0-9_-]+)*$'`;

function stripTenantQuery(block) {
  return block
    .split("\n        - $ref: '#/components/parameters/TenantIdQuery'")
    .join('');
}

function insertBeforeFirstAnchor(text, anchor, insertion) {
  const index = text.indexOf(anchor);
  if (index < 0) {
    throw new Error(`anchor missing: ${anchor.trim()}`);
  }
  return `${text.slice(0, index)}${insertion}${anchor}${text.slice(index + anchor.length)}`;
}

function ensureParameters(text) {
  if (text.includes('    SessionIdPath:')) {
    return text;
  }
  return insertBeforeFirstAnchor(
    text,
    '    BindingIdPath:',
    `${chatParameterBlock}\n`,
  );
}

function ensureSchemas(text) {
  if (text.includes('    AgentChatCompletionResponse:')) {
    return text;
  }
  return insertBeforeFirstAnchor(
    text,
    '    FieldError:',
    `${sharedSchemas}\n${surfaceSchemas}\n`,
  );
}

function patchApp() {
  const file = path.join(base, 'agents-app-api.openapi.yaml');
  let text = fs.readFileSync(file, 'utf8');
  if (!text.includes('/app/v3/api/ai/agents/{agentId}/sessions:')) {
    let paths = projectPaths(sessionPathsBlock, '/app/v3/api');
    paths = stripTenantQuery(paths);
    paths = paths.replaceAll(
      'CreateAgentSessionRequest',
      'AppCreateAgentSessionRequest',
    );
    paths = paths.replaceAll(
      'CloseAgentSessionRequest',
      'AppCloseAgentSessionRequest',
    );
    paths = paths.replaceAll(
      'SendAgentChatMessageRequest',
      'AppSendAgentChatMessageRequest',
    );
    text = text.replace(
      '  /app/v3/api/ai/agents/{agentId}/composition_slots:',
      `${paths}\n\n  /app/v3/api/ai/agents/{agentId}/composition_slots:`,
    );
  }
  text = ensureParameters(text);
  text = ensureSchemas(text);
  fs.writeFileSync(file, text);
}

function patchBackend() {
  const file = path.join(base, 'agents-backend-api.openapi.yaml');
  let text = fs.readFileSync(file, 'utf8');
  if (!text.includes('/backend/v3/api/ai/agents/{agentId}/sessions:')) {
    let paths = projectPaths(sessionPathsBlock, '/backend/v3/api');
    paths = paths.replaceAll('TenantIdQuery', 'TenantId');
    const archivePath = `  /backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/archive:
    post:
      tags: [ai]
      summary: Archive one chat session
      operationId: agents.sessions.archive
      security:
        - AuthToken: []
          AccessToken: []
      x-sdkwork-domain: ai
      x-sdkwork-resource: agents.sessions
      x-sdkwork-permission: agent.business.session.archive
      x-sdkwork-tenant-scope: tenant
      x-sdkwork-audit-event: agent.business.session_archived
      parameters:
        - $ref: '#/components/parameters/AgentIdPath'
        - $ref: '#/components/parameters/SessionIdPath'
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/ArchiveAgentSessionRequest'
      responses:
        '200':
          description: Archived managed agent session
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/AgentSessionResponse'
        default:
          $ref: '#/components/responses/Problem'

`;
    text = text.replace(
      '  /backend/v3/api/ai/agents/{agentId}/composition_slots:',
      `${paths}\n${archivePath}  /backend/v3/api/ai/agents/{agentId}/composition_slots:`,
    );
  }
  text = ensureParameters(text);
  text = ensureSchemas(text);
  fs.writeFileSync(file, text);
}

patchApp();
patchBackend();
console.log('synced chat openapi surfaces');
