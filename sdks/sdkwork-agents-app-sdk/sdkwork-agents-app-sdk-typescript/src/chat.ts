import type { SdkworkAppClient } from '../generated/server-openapi/src/index';
import type { AppSendAgentChatMessageRequest } from '../generated/server-openapi/src/types';

/** Non-streaming managed-agent chat turn (OpenAPI `stream=false`). */
export async function sendAgentChatMessageSync(
  client: SdkworkAppClient,
  agentId: string,
  sessionId: string,
  body: AppSendAgentChatMessageRequest,
): Promise<Record<string, unknown>> {
  return client.ai.agents.messages.complete(agentId, sessionId, body);
}
