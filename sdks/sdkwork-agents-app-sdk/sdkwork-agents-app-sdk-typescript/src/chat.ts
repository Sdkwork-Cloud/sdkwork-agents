import type { SdkworkAppClient } from '../generated/server-openapi/src/index';
import type { AppSendAgentChatMessageRequest } from '../generated/server-openapi/src/types';

/** Non-streaming managed-agent chat turn (OpenAPI `stream=false`). */
export async function sendAgentChatMessageSync(
  client: SdkworkAppClient,
  agentId: string,
  sessionId: string,
  body: AppSendAgentChatMessageRequest,
): Promise<Record<string, unknown>> {
  const path = `/ai/agents/${encodeURIComponent(agentId)}/sessions/${encodeURIComponent(sessionId)}/messages?stream=false`;
  const response = await client.http.post<unknown>(
    path,
    body,
    undefined,
    undefined,
    'application/json',
  );
  if (!response || typeof response !== 'object' || Array.isArray(response)) {
    throw new Error('Chat completion response was not a JSON object.');
  }
  return response as Record<string, unknown>;
}
