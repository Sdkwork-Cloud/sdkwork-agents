import assert from 'node:assert/strict';
import { pathToFileURL } from 'node:url';
import type { SdkworkAgentsAppClient, AgentManagementProfile } from '@sdkwork/agents-pc-core/sdk/agentsAppSdkClient';
import type * as AgentServiceModule from '../packages/sdkwork-agents-pc-agents/src/services/AgentService.ts';

type AgentServiceExports = typeof AgentServiceModule;
type AgentConfig = AgentServiceModule.AgentConfig;
type RecordLike = Record<string, unknown>;
type AgentRequestBody = RecordLike & {
  managementProfile?: AgentManagementProfile | null;
};

async function loadAgentServiceModule(): Promise<AgentServiceExports> {
  const moduleUrl = pathToFileURL(
    './packages/sdkwork-agents-pc-agents/src/services/AgentService.ts',
  ).href;
  const loaded = (await import(moduleUrl)) as Partial<AgentServiceExports> & {
    default?: Partial<AgentServiceExports>;
  };
  const createSdkworkAgentService =
    loaded.createSdkworkAgentService ?? loaded.default?.createSdkworkAgentService;
  assert.equal(typeof createSdkworkAgentService, 'function');
  return {
    ...loaded.default,
    ...loaded,
    createSdkworkAgentService,
  } as AgentServiceExports;
}

function makeAgentRecord(overrides: RecordLike = {}): RecordLike {
  const agentId = String(overrides.agentId ?? 'agent.pc.management.profile');
  return {
    id: '1001',
    agentId,
    tenantId: '100001',
    organizationId: '0',
    ownerUserId: '100',
    code: agentId,
    displayName: String(overrides.displayName ?? 'Management Profile Agent'),
    description: String(overrides.description ?? 'Agent profile contract'),
    manifest: {
      description: 'Manifest system prompt',
    },
    defaultCodeTaskIntent: {
      contextPaths: ['knowledge.base.legacy.manual'],
      constraints: [],
    },
    implementationKind: 'manifest-only',
    implementationProviderId: null,
    managementProfile: overrides.managementProfile ?? {
      author: 'SDKWork Agent Studio',
      avatar: 'robot',
      categoryId: 'tech',
      color: 'bg-purple-500',
      debugMode: true,
      iconName: 'Terminal',
      jsonMode: true,
      knowledgeBaseIds: ['knowledge.base.product.manual'],
      memoryEnabled: true,
      model: 'model.openai.gpt-4o',
      skillIds: ['skill.planning'],
      suggestedPrompts: ['Summarize the knowledge base', 'Draft an execution plan'],
      systemPrompt: 'Act as a precise product agent.',
      temperature: 0.4,
      toolIds: ['tool.web-search'],
      type: 'independent',
      users: '42 users',
      voiceIds: ['voice.voice-1'],
      welcomeMessage: 'Welcome to the managed agent.',
    },
    status: 'active',
    tags: ['tech'],
    version: '3',
    visibility: overrides.visibility ?? 'private',
    createdAt: '2026-06-01T00:00:00Z',
    updatedAt: '2026-06-01T00:10:00Z',
    deletedAt: null,
    ...overrides,
  };
}

const requests: {
  create?: { body: AgentRequestBody; params: RecordLike };
  retrieve?: { id: string; params: RecordLike };
  update?: { id: string; body: AgentRequestBody; params: RecordLike };
} = {};

const compositionSlotsByAgent = new Map<string, Array<RecordLike>>();

const fakeClient = {
  ai: {
    agents: {
      compositionSlots: {
        async list(agentId: string) {
          return {
            items: compositionSlotsByAgent.get(agentId) ?? [],
            pageInfo: { page: 1, pageSize: 20, totalItems: '0', totalPages: 1 },
          };
        },
        async create(agentId: string, body: RecordLike) {
          const slot = {
            slotId: body.data?.slotId ?? 'slot.test',
            version: '1',
            ...(body.data as RecordLike),
          };
          const current = compositionSlotsByAgent.get(agentId) ?? [];
          compositionSlotsByAgent.set(agentId, [...current, slot]);
          return slot;
        },
        async delete(_agentId: string, _slotId: string, _params: RecordLike) {
          return { accepted: true };
        },
      },
      async list(params: RecordLike) {
        assert.equal(params.page, 1);
        assert.equal(params.pageSize, 20);
        return {
          items: [makeAgentRecord()],
          pageInfo: {
            page: 1,
            pageSize: 20,
            totalItems: '1',
            totalPages: 1,
          },
        };
      },
      async create(body: AgentRequestBody, params: RecordLike) {
        requests.create = { body, params };
        return makeAgentRecord({
          agentId: body.agentId,
          displayName: body.displayName,
          description: body.description,
          managementProfile: body.managementProfile,
        });
      },
      async retrieve(id: string, params: RecordLike) {
        requests.retrieve = { id, params };
        return makeAgentRecord({
          agentId: id,
        });
      },
      async update(id: string, body: AgentRequestBody, params: RecordLike) {
        requests.update = { id, body, params };
        return makeAgentRecord({
          agentId: id,
          displayName: body.displayName ?? 'Updated Management Profile Agent',
          description: body.description ?? 'Updated profile contract',
          managementProfile: body.managementProfile,
        });
      },
    },
  },
} as unknown as SdkworkAgentsAppClient;

const { createSdkworkAgentService } = await loadAgentServiceModule();
const agentService = createSdkworkAgentService(() => fakeClient);

const listedPage = await agentService.listAgentsPage();
const listedAgent = listedPage.items[0];
assert.ok(listedAgent, 'expected one listed agent');
assert.equal(listedAgent?.id, 'agent.pc.management.profile');
assert.equal(listedAgent?.systemPrompt, 'Act as a precise product agent.');
assert.deepEqual(listedAgent?.knowledgeBaseIds, ['knowledge.base.product.manual']);
assert.equal(listedAgent.model, 'openai.gpt-4o');
assert.equal(listedAgent.temperature, 0.4);
assert.equal(listedAgent.debugMode, true);
assert.equal(listedAgent.jsonMode, true);
assert.equal(listedAgent.memoryEnabled, true);
assert.deepEqual(listedAgent.suggestedPrompts, [
  'Summarize the knowledge base',
  'Draft an execution plan',
]);
assert.deepEqual(listedAgent.voiceIds, ['voice-1']);
assert.deepEqual(listedAgent.toolIds, ['web-search']);
assert.deepEqual(listedAgent.skillIds, ['planning']);

await agentService.createAgent({
  author: 'SDKWork Agent Studio',
  avatar: 'robot',
  categoryId: 'tech',
  color: 'bg-purple-500',
  debugMode: true,
  description: 'Created profile contract',
  iconName: 'Terminal',
  jsonMode: true,
  knowledgeBaseIds: ['knowledge.base.product.manual'],
  memoryEnabled: true,
  model: 'openai.gpt-4o',
  name: 'Created Management Profile Agent',
  skillIds: ['planning'],
  suggestedPrompts: ['Summarize the knowledge base', 'Draft an execution plan'],
  systemPrompt: 'Act as a precise product agent.',
  temperature: 0.4,
  toolIds: ['web-search'],
  type: 'independent',
  users: '42 users',
  voiceIds: ['voice-1'],
  welcomeMessage: 'Welcome to the managed agent.',
} satisfies AgentConfig);

assert.deepEqual(requests.create?.body.managementProfile, {
  author: 'SDKWork Agent Studio',
  avatar: 'robot',
  categoryId: 'tech',
  color: 'bg-purple-500',
  debugMode: true,
  iconName: 'Terminal',
  jsonMode: true,
  knowledgeBaseIds: ['knowledge.base.product.manual'],
  memoryEnabled: true,
  model: 'model.openai.gpt-4o',
  skillIds: ['skill.planning'],
  suggestedPrompts: ['Summarize the knowledge base', 'Draft an execution plan'],
  systemPrompt: 'Act as a precise product agent.',
  temperature: 0.4,
  toolIds: ['tool.web-search'],
  type: 'independent',
  users: '42 users',
  voiceIds: ['voice.voice-1'],
  welcomeMessage: 'Welcome to the managed agent.',
});

await agentService.updateAgent('agent.pc.management.profile', {
  debugMode: false,
  jsonMode: true,
  memoryEnabled: false,
  model: 'anthropic.claude-3.5-sonnet',
  skillIds: ['multi-agent'],
  temperature: 0.2,
  toolIds: ['mcp-github'],
  voiceIds: ['voice-my-1'],
} satisfies Partial<AgentConfig>);

assert.equal(requests.retrieve?.id, 'agent.pc.management.profile');
assert.equal(requests.update?.id, 'agent.pc.management.profile');
assert.equal(requests.update?.body.managementProfile?.author, 'SDKWork Agent Studio');
assert.equal(requests.update?.body.managementProfile?.avatar, 'robot');
assert.equal(requests.update?.body.managementProfile?.categoryId, 'tech');
assert.equal(requests.update?.body.managementProfile?.color, 'bg-purple-500');
assert.equal(requests.update?.body.managementProfile?.iconName, 'Terminal');
assert.deepEqual(requests.update?.body.managementProfile?.knowledgeBaseIds, [
  'knowledge.base.product.manual',
]);
assert.deepEqual(requests.update?.body.managementProfile?.suggestedPrompts, [
  'Summarize the knowledge base',
  'Draft an execution plan',
]);
assert.equal(requests.update?.body.managementProfile?.systemPrompt, 'Act as a precise product agent.');
assert.equal(requests.update?.body.managementProfile?.type, 'independent');
assert.equal(requests.update?.body.managementProfile?.users, '42 users');
assert.equal(requests.update?.body.managementProfile?.welcomeMessage, 'Welcome to the managed agent.');
assert.equal(requests.update?.body.managementProfile?.model, 'model.anthropic.claude-3.5-sonnet');
assert.deepEqual(requests.update?.body.managementProfile?.toolIds, ['tool.mcp-github']);
assert.deepEqual(requests.update?.body.managementProfile?.skillIds, ['skill.multi-agent']);
assert.deepEqual(requests.update?.body.managementProfile?.voiceIds, ['voice.voice-my-1']);
assert.equal(requests.update?.body.managementProfile?.debugMode, false);
assert.equal(requests.update?.body.managementProfile?.jsonMode, true);
assert.equal(requests.update?.body.managementProfile?.memoryEnabled, false);
assert.equal(requests.update?.body.managementProfile?.temperature, 0.2);

console.log('sdkwork agents pc agent service management profile contract passed.');
