import assert from 'node:assert/strict';
import test from 'node:test';

import { createTokenManager } from '@sdkwork/sdk-common';

import {
  createClient,
  completeAgentTurn,
  type CompleteAgentTurnResult,
  SdkworkAppClient,
} from '../src/index.ts';

const APP_API_BASE_URL = 'http://127.0.0.1:8095/app/v3/api';
const SDK_BASE_URL = 'http://127.0.0.1:8095';
const EXPECTED_AGENTS_URL =
  'http://127.0.0.1:8095/app/v3/api/ai/agents?scope=market&page=1&page_size=20';

function createAuthenticatedClient(baseUrl: string): SdkworkAppClient {
  return createClient({
    baseUrl,
    tokenManager: createTokenManager({ accessToken: 'test-access-token' }),
  });
}

for (const baseUrl of [SDK_BASE_URL, APP_API_BASE_URL]) {
  test(`Agents App SDK composes the app-api prefix exactly once from ${baseUrl}`, async () => {
  const originalFetch = globalThis.fetch;
  const requestedUrls: string[] = [];
  globalThis.fetch = async (input) => {
    requestedUrls.push(String(input));
    return new Response(
      JSON.stringify({
        code: 0,
        data: {
          items: [],
          pageInfo: { page: 1, pageSize: 20, total: 0, totalPages: 0 },
        },
      }),
      { headers: { 'content-type': 'application/json' }, status: 200 },
    );
  };

  try {
    const client = createAuthenticatedClient(baseUrl);
    await client.ai.agents.list({ scope: 'market', page: 1, pageSize: 20 });
  } finally {
    globalThis.fetch = originalFetch;
  }

  assert.deepEqual(requestedUrls, [EXPECTED_AGENTS_URL]);
  });
}

test('Agents App SDK rejects empty and duplicated app-api base URLs', () => {
  assert.throws(
    () => new SdkworkAppClient({ baseUrl: '  ' }),
    /must not be empty/,
  );
  assert.throws(
    () => new SdkworkAppClient({
      baseUrl: 'http://127.0.0.1:8095/app/v3/api/app/v3/api',
    }),
    /must include \/app\/v3\/api exactly once/,
  );
  assert.throws(
    () => new SdkworkAppClient({ baseUrl: 'http://127.0.0.1:8095/app/v3/api/extra' }),
    /must identify a gateway root or end with \/app\/v3\/api/,
  );
});

test('Agents App SDK supports the standard same-origin app-api surface path', async () => {
  const originalFetch = globalThis.fetch;
  const requestedUrls: string[] = [];
  globalThis.fetch = async (input) => {
    requestedUrls.push(String(input));
    return new Response(
      JSON.stringify({
        code: 0,
        data: {
          items: [],
          pageInfo: { page: 1, pageSize: 20, total: 0, totalPages: 0 },
        },
      }),
      { headers: { 'content-type': 'application/json' }, status: 200 },
    );
  };

  try {
    const client = createAuthenticatedClient('/app/v3/api');
    await client.ai.agents.list({ scope: 'market', page: 1, pageSize: 20 });
  } finally {
    globalThis.fetch = originalFetch;
  }

  assert.deepEqual(requestedUrls, [
    '/app/v3/api/ai/agents?scope=market&page=1&page_size=20',
  ]);
});

test('Agents App SDK turn helper uses the canonical app-api prefix', async () => {
  const originalFetch = globalThis.fetch;
  const requestedUrls: string[] = [];
  const turnResult = {
    session: { sessionId: 'session-1' },
    turn: { turnId: 'turn-1' },
    items: [{ itemId: 'item-1', kind: 'assistant_output', content: 'ok' }],
  };
  globalThis.fetch = async (input) => {
    requestedUrls.push(String(input));
    return new Response(
      JSON.stringify({ code: 0, data: { item: turnResult } }),
      { headers: { 'content-type': 'application/json' }, status: 200 },
    );
  };

  let completion: CompleteAgentTurnResult | undefined;
  try {
    const client = createAuthenticatedClient(APP_API_BASE_URL);
    completion = await completeAgentTurn(client, 'agent-1', 'session-1', {
      content: 'hello',
      turnMode: 'interactive',
      idempotencyKey: 'turn-test-1',
      payloadHash: 'sha256:turn-test-1',
      requestedAt: '2026-07-18T00:00:00.000Z',
    });
  } finally {
    globalThis.fetch = originalFetch;
  }

  assert.deepEqual(completion, turnResult);
  assert.equal('item' in completion!, false);
  assert.deepEqual(requestedUrls, [
    'http://127.0.0.1:8095/app/v3/api/ai/agents/agent-1/sessions/session-1/turns?stream=false',
  ]);
});

test('Agents App SDK exposes paginated interaction filters and single-item retrieval', async () => {
  const originalFetch = globalThis.fetch;
  const requestedUrls: string[] = [];
  globalThis.fetch = async (input) => {
    const url = String(input);
    requestedUrls.push(url);
    const isRetrieve = url.endsWith('/interactions/interaction-1');
    return new Response(
      JSON.stringify({
        code: 0,
        data: isRetrieve
          ? { item: { interactionId: 'interaction-1', kind: 'user_question', status: 'pending' } }
          : {
              items: [],
              pageInfo: {
                mode: 'offset',
                page: 1,
                pageSize: 20,
                totalItems: '0',
                totalPages: 0,
                hasMore: false,
              },
            },
      }),
      { headers: { 'content-type': 'application/json' }, status: 200 },
    );
  };

  try {
    const client = createAuthenticatedClient(APP_API_BASE_URL);
    const page = await client.ai.agents.interactions.list('agent-1', 'session-1', {
      page: 1,
      pageSize: 20,
      kind: 'user_question',
      status: 'pending',
    });
    assert.equal(page.pageInfo.hasMore, false);

    const retrieved = await client.ai.agents.interactions.retrieve(
      'agent-1',
      'session-1',
      'interaction-1',
    );
    assert.equal(retrieved.interactionId, 'interaction-1');
  } finally {
    globalThis.fetch = originalFetch;
  }

  assert.deepEqual(requestedUrls, [
    'http://127.0.0.1:8095/app/v3/api/ai/agents/agent-1/sessions/session-1/interactions?page=1&page_size=20&kind=user_question&status=pending',
    'http://127.0.0.1:8095/app/v3/api/ai/agents/agent-1/sessions/session-1/interactions/interaction-1',
  ]);
});

test('Agents App SDK exposes server-side Session Item filters and descending pages', async () => {
  const originalFetch = globalThis.fetch;
  const requestedUrls: string[] = [];
  globalThis.fetch = async (input) => {
    requestedUrls.push(String(input));
    return new Response(
      JSON.stringify({
        code: 0,
        data: {
          items: [],
          pageInfo: {
            mode: 'cursor',
            pageSize: 20,
            nextCursor: null,
            hasMore: false,
          },
        },
      }),
      { headers: { 'content-type': 'application/json' }, status: 200 },
    );
  };

  try {
    const client = createAuthenticatedClient(APP_API_BASE_URL);
    const page = await client.ai.agents.sessionItems.list('agent-1', 'session-1', {
      pageSize: 20,
      kind: 'assistant_output',
      status: 'completed',
      sort: '-sequence',
    });
    assert.equal(page.pageInfo.mode, 'cursor');
  } finally {
    globalThis.fetch = originalFetch;
  }

  assert.deepEqual(requestedUrls, [
    'http://127.0.0.1:8095/app/v3/api/ai/agents/agent-1/sessions/session-1/items?page_size=20&kind=assistant_output&status=completed&sort=-sequence',
  ]);
});

test('Agents App SDK exposes distinct agent, project, and workspace session lists', async () => {
  const originalFetch = globalThis.fetch;
  const requestedUrls: string[] = [];
  globalThis.fetch = async (input) => {
    requestedUrls.push(String(input));
    return new Response(
      JSON.stringify({
        code: 0,
        data: {
          items: [],
          pageInfo: {
            mode: 'offset',
            page: 2,
            pageSize: 50,
            totalItems: '0',
            totalPages: 0,
            hasMore: false,
          },
        },
      }),
      { headers: { 'content-type': 'application/json' }, status: 200 },
    );
  };

  try {
    const client = createAuthenticatedClient(APP_API_BASE_URL);
    const params = {
      page: 2,
      pageSize: 50,
      status: 'idle' as const,
      includeArchived: true,
    };
    await client.ai.agents.sessions.list('agent-1', {
      ...params,
      projectId: 'project-filter',
    });
    await client.ai.agents.projectSessions.list('project-1', params);
    await client.ai.agents.workspaceSessions.list('workspace-1', params);
  } finally {
    globalThis.fetch = originalFetch;
  }

  assert.deepEqual(requestedUrls, [
    'http://127.0.0.1:8095/app/v3/api/ai/agents/agent-1/sessions?page=2&page_size=50&project_id=project-filter&status=idle&include_archived=true',
    'http://127.0.0.1:8095/app/v3/api/ai/projects/project-1/sessions?page=2&page_size=50&status=idle&include_archived=true',
    'http://127.0.0.1:8095/app/v3/api/ai/workspaces/workspace-1/sessions?page=2&page_size=50&status=idle&include_archived=true',
  ]);
});

test('Agents App SDK delivers named SSE delta frames before completion', async () => {
  const originalFetch = globalThis.fetch;
  const encoder = new TextEncoder();
  const requestedBodies: string[] = [];
  let releaseCompletion = () => {};
  const completionGate = new Promise<void>((resolve) => {
    releaseCompletion = resolve;
  });

  globalThis.fetch = async (_input, init) => {
    requestedBodies.push(String(init?.body ?? ''));
    let frame = 0;
    return new Response(
      new ReadableStream({
        async pull(controller) {
          if (frame === 0) {
            frame += 1;
            controller.enqueue(encoder.encode([
              'event: delta',
              'data: {"eventType":"delta","index":0,"delta":"hello"}',
              '',
              '',
            ].join('\r\n')));
            return;
          }
          if (frame === 1) {
            frame += 1;
            await completionGate;
            controller.enqueue(encoder.encode([
              'event: completion',
              'data: {"eventType":"completion","response":{"code":0}}',
              '',
              '',
            ].join('\r\n')));
            controller.close();
          }
        },
      }),
      { headers: { 'content-type': 'text/event-stream; charset=utf-8' }, status: 200 },
    );
  };

  const body = {
    content: 'hello',
    idempotencyKey: 'turn-stream-contract',
    payloadHash: 'sha256:turn-stream-contract',
    requestedAt: '2026-08-01T00:00:00.000Z',
    turnMode: 'interactive' as const,
  };

  try {
    const client = createAuthenticatedClient(APP_API_BASE_URL);
    const events = await client.ai.agents.turns.stream(
      'agent.code-engine.codex',
      'session.stream-contract',
      body,
      { eventProtocol: 'kernel-v1', stream: true },
    );
    const iterator = events[Symbol.asyncIterator]();

    const first = await iterator.next();
    assert.deepEqual(first.value, {
      eventType: 'delta',
      index: 0,
      delta: 'hello',
    });

    let completionDelivered = false;
    const completion = iterator.next().then((result) => {
      completionDelivered = true;
      return result;
    });
    await new Promise<void>((resolve) => setImmediate(resolve));
    assert.equal(
      completionDelivered,
      false,
      'the first delta must be observable while the completion frame is still gated',
    );

    releaseCompletion();
    assert.deepEqual((await completion).value, {
      eventType: 'completion',
      response: { code: 0 },
    });
  } finally {
    releaseCompletion();
    globalThis.fetch = originalFetch;
  }

  assert.deepEqual(JSON.parse(requestedBodies[0] ?? ''), body);
});
