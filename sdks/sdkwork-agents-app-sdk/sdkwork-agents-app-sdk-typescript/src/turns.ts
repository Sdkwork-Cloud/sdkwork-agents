import type { SdkworkAppClient } from '../generated/server-openapi/src/index.ts';
import { appApiPath } from '../generated/server-openapi/src/api/paths.ts';
import type {
  AgentTurnExecutionResponse,
  CreateAgentTurnRequest,
} from '../generated/server-openapi/src/types/index.ts';

export type CompleteAgentTurnResult = AgentTurnExecutionResponse['data']['item'];

function pathSegment(value: string, name: string): string {
  const normalized = value.trim();
  if (!normalized) {
    throw new Error(`${name} is required.`);
  }
  return encodeURIComponent(normalized);
}

/** Execute one non-streaming turn through the canonical POST /turns operation. */
export async function completeAgentTurn(
  client: SdkworkAppClient,
  agentId: string,
  sessionId: string,
  body: CreateAgentTurnRequest,
): Promise<CompleteAgentTurnResult> {
  const path = appApiPath(
    `/ai/agents/${pathSegment(agentId, 'agentId')}/sessions/${pathSegment(sessionId, 'sessionId')}/turns`,
  );
  return client.http.request<CompleteAgentTurnResult>(`${path}?stream=false`, {
    method: 'POST',
    body,
    contentType: 'application/json',
    sdkworkUnwrapKind: 'item',
  });
}
