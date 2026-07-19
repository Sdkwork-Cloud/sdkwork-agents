import assert from 'node:assert/strict';
import test from 'node:test';

import {
  createClient,
  sendAgentChatMessageSync,
  SdkworkAppClient,
} from '../src/index.ts';

const APP_API_BASE_URL = 'http://127.0.0.1:8095/app/v3/api';
const EXPECTED_AGENTS_URL =
  'http://127.0.0.1:8095/app/v3/api/ai/agents?scope=market&page=1&page_size=20';

test('Agents App SDK composes the app-api prefix exactly once', async () => {
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
    const client = createClient({ baseUrl: APP_API_BASE_URL });
    await client.ai.agents.list({ scope: 'market', page: 1, pageSize: 20 });
  } finally {
    globalThis.fetch = originalFetch;
  }

  assert.deepEqual(requestedUrls, [EXPECTED_AGENTS_URL]);
});

test('Agents App SDK requires one canonical app-api surface URL', () => {
  assert.throws(
    () => new SdkworkAppClient({ baseUrl: 'http://127.0.0.1:8095' }),
    /must end with \/app\/v3\/api/,
  );
  assert.throws(
    () => new SdkworkAppClient({
      baseUrl: 'http://127.0.0.1:8095/app/v3/api/app/v3/api',
    }),
    /must include \/app\/v3\/api exactly once/,
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
    const client = createClient({ baseUrl: '/app/v3/api' });
    await client.ai.agents.list({ scope: 'market', page: 1, pageSize: 20 });
  } finally {
    globalThis.fetch = originalFetch;
  }

  assert.deepEqual(requestedUrls, [
    '/app/v3/api/ai/agents?scope=market&page=1&page_size=20',
  ]);
});

test('Agents App SDK chat helper uses the same canonical app-api prefix', async () => {
  const originalFetch = globalThis.fetch;
  const requestedUrls: string[] = [];
  globalThis.fetch = async (input) => {
    requestedUrls.push(String(input));
    return new Response(
      JSON.stringify({ code: 0, data: { item: { content: 'ok' } } }),
      { headers: { 'content-type': 'application/json' }, status: 200 },
    );
  };

  try {
    const client = createClient({ baseUrl: APP_API_BASE_URL });
    await sendAgentChatMessageSync(client, 'agent-1', 'session-1', {
      content: 'hello',
      idempotencyKey: 'chat-test-1',
      requestedAt: '2026-07-18T00:00:00.000Z',
    });
  } finally {
    globalThis.fetch = originalFetch;
  }

  assert.deepEqual(requestedUrls, [
    'http://127.0.0.1:8095/app/v3/api/ai/agents/agent-1/sessions/session-1/messages/complete',
  ]);
});
