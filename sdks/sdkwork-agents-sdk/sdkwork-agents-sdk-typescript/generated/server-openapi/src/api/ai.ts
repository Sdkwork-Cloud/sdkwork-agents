import { customApiPath } from './paths';
import type { HttpClient } from '../http/client';

import type { ActivateAgentProviderBindingRequest, AgentCompositionSlotRecord, AgentInteractionKind, AgentInteractionRecord, AgentInteractionStatus, AgentProviderBindingRecord, AgentRecord, AgentRuntimeExecutionRecord, AgentSessionCheckpointRecord, AgentSessionItemKind, AgentSessionItemRecord, AgentSessionItemStatus, AgentSessionRecord, AgentSessionRuntimeBindingRecord, AgentTaskRecord, AgentTurnRecord, AgentTurnStreamEvent, AnswerAgentInteractionRequest, ApproveAgentInteractionRequest, CancelAgentTaskRequest, CancelAgentTurnRequest, ChangeAgentSessionRuntimeBindingStatusRequest, ClaimAgentInteractionRequest, CloseAgentSessionRequest, CreateAgentCompositionSlotRequest, CreateAgentInteractionRequest, CreateAgentPreviewResponseRequest, CreateAgentPromptOptimizationRequest, CreateAgentProviderBindingRequest, CreateAgentRequest, CreateAgentSessionCheckpointRequest, CreateAgentSessionRequest, CreateAgentSessionRuntimeBindingRequest, CreateAgentTaskRequest, CreateAgentTurnRequest, Int64String, InvalidateAgentSessionCheckpointRequest, RestoreAgentSessionCheckpointRequest, SdkWorkPageData, UpdateAgentCompositionSlotRequest, UpdateAgentRequest, UpdateAgentSessionRuntimeBindingRequest } from '../types';


export class AiAgentsCompositionSlotsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List composition slots for one managed agent */
  async list(agentId: string): Promise<SdkWorkPageData & { items: AgentCompositionSlotRecord[]; }> {
    return this.client.get<SdkWorkPageData & { items: AgentCompositionSlotRecord[]; }>(customApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/composition_slots`));
  }

/** Create a composition slot for one managed agent */
  async create(agentId: string, body: CreateAgentCompositionSlotRequest): Promise<AgentCompositionSlotRecord> {
    return this.client.post<AgentCompositionSlotRecord>(customApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/composition_slots`), body, undefined, undefined, 'application/json');
  }

/** Retrieve one managed agent composition slot */
  async retrieve(agentId: string, slotId: string): Promise<AgentCompositionSlotRecord> {
    return this.client.get<AgentCompositionSlotRecord>(customApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/composition_slots/${serializePathParameter(slotId, { name: 'slotId', style: 'simple', explode: false })}`));
  }

/** Update one managed agent composition slot */
  async update(agentId: string, slotId: string, body: UpdateAgentCompositionSlotRequest): Promise<AgentCompositionSlotRecord> {
    return this.client.patch<AgentCompositionSlotRecord>(customApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/composition_slots/${serializePathParameter(slotId, { name: 'slotId', style: 'simple', explode: false })}`), body, undefined, undefined, 'application/json');
  }

/** Delete one managed agent composition slot */
  async delete(agentId: string, slotId: string): Promise<void> {
    return this.client.delete<void>(customApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/composition_slots/${serializePathParameter(slotId, { name: 'slotId', style: 'simple', explode: false })}`));
  }
}

export interface AiAgentsTasksListParams {
  page?: number;
  pageSize?: number;
}

export class AiAgentsTasksApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List scheduled tasks for one managed agent */
  async list(agentId: string, params?: AiAgentsTasksListParams): Promise<SdkWorkPageData & { items: AgentTaskRecord[]; }> {
    const query = buildQueryString([
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<SdkWorkPageData & { items: AgentTaskRecord[]; }>(appendQueryString(customApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/tasks`), query));
  }

/** Create a scheduled task for one managed agent */
  async create(agentId: string, body: CreateAgentTaskRequest): Promise<AgentTaskRecord> {
    return this.client.post<AgentTaskRecord>(customApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/tasks`), body, undefined, undefined, 'application/json');
  }

/** Retrieve one scheduled task */
  async retrieve(agentId: string, taskId: string): Promise<AgentTaskRecord> {
    return this.client.get<AgentTaskRecord>(customApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/tasks/${serializePathParameter(taskId, { name: 'taskId', style: 'simple', explode: false })}`));
  }

/** Cancel one scheduled task */
  async cancel(agentId: string, taskId: string, body: CancelAgentTaskRequest): Promise<AgentTaskRecord> {
    return this.client.post<AgentTaskRecord>(customApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/tasks/${serializePathParameter(taskId, { name: 'taskId', style: 'simple', explode: false })}/cancel`), body, undefined, undefined, 'application/json');
  }

/** Execute one deferred scheduled task */
  async execute(agentId: string, taskId: string, body: CancelAgentTaskRequest): Promise<AgentTaskRecord> {
    return this.client.post<AgentTaskRecord>(customApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/tasks/${serializePathParameter(taskId, { name: 'taskId', style: 'simple', explode: false })}/execute`), body, undefined, undefined, 'application/json');
  }
}

export interface AiAgentsSessionRuntimeBindingsListParams {
  page?: number;
  pageSize?: number;
}

export class AiAgentsSessionRuntimeBindingsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List runtime bindings for one agent session */
  async list(agentId: string, sessionId: string, params?: AiAgentsSessionRuntimeBindingsListParams): Promise<SdkWorkPageData & { items: AgentSessionRuntimeBindingRecord[]; }> {
    const query = buildQueryString([
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<SdkWorkPageData & { items: AgentSessionRuntimeBindingRecord[]; }>(appendQueryString(customApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/runtime_bindings`), query));
  }

/** Create the current runtime binding for one agent session */
  async create(agentId: string, sessionId: string, body: CreateAgentSessionRuntimeBindingRequest): Promise<AgentSessionRuntimeBindingRecord> {
    return this.client.post<AgentSessionRuntimeBindingRecord>(customApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/runtime_bindings`), body, undefined, undefined, 'application/json');
  }

/** Retrieve one agent session runtime binding */
  async retrieve(agentId: string, sessionId: string, runtimeBindingId: string): Promise<AgentSessionRuntimeBindingRecord> {
    return this.client.get<AgentSessionRuntimeBindingRecord>(customApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/runtime_bindings/${serializePathParameter(runtimeBindingId, { name: 'runtimeBindingId', style: 'simple', explode: false })}`));
  }

/** Update one agent session runtime binding */
  async update(agentId: string, sessionId: string, runtimeBindingId: string, body: UpdateAgentSessionRuntimeBindingRequest): Promise<AgentSessionRuntimeBindingRecord> {
    return this.client.patch<AgentSessionRuntimeBindingRecord>(customApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/runtime_bindings/${serializePathParameter(runtimeBindingId, { name: 'runtimeBindingId', style: 'simple', explode: false })}`), body, undefined, undefined, 'application/json');
  }

/** Activate one agent session runtime binding as current */
  async activate(agentId: string, sessionId: string, runtimeBindingId: string, body: ChangeAgentSessionRuntimeBindingStatusRequest): Promise<AgentSessionRuntimeBindingRecord> {
    return this.client.post<AgentSessionRuntimeBindingRecord>(customApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/runtime_bindings/${serializePathParameter(runtimeBindingId, { name: 'runtimeBindingId', style: 'simple', explode: false })}/activate`), body, undefined, undefined, 'application/json');
  }

/** Deactivate one agent session runtime binding */
  async deactivate(agentId: string, sessionId: string, runtimeBindingId: string, body: ChangeAgentSessionRuntimeBindingStatusRequest): Promise<AgentSessionRuntimeBindingRecord> {
    return this.client.post<AgentSessionRuntimeBindingRecord>(customApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/runtime_bindings/${serializePathParameter(runtimeBindingId, { name: 'runtimeBindingId', style: 'simple', explode: false })}/deactivate`), body, undefined, undefined, 'application/json');
  }
}

export interface AiAgentsCheckpointsListParams {
  page?: number;
  pageSize?: number;
}

export class AiAgentsCheckpointsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List resumable checkpoints for one agent session */
  async list(agentId: string, sessionId: string, params?: AiAgentsCheckpointsListParams): Promise<SdkWorkPageData & { items: AgentSessionCheckpointRecord[]; }> {
    const query = buildQueryString([
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<SdkWorkPageData & { items: AgentSessionCheckpointRecord[]; }>(appendQueryString(customApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/checkpoints`), query));
  }

/** Create one bounded agent session checkpoint */
  async create(agentId: string, sessionId: string, body: CreateAgentSessionCheckpointRequest): Promise<AgentSessionCheckpointRecord> {
    return this.client.post<AgentSessionCheckpointRecord>(customApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/checkpoints`), body, undefined, undefined, 'application/json');
  }

/** Retrieve one agent session checkpoint */
  async retrieve(agentId: string, sessionId: string, checkpointId: string): Promise<AgentSessionCheckpointRecord> {
    return this.client.get<AgentSessionCheckpointRecord>(customApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/checkpoints/${serializePathParameter(checkpointId, { name: 'checkpointId', style: 'simple', explode: false })}`));
  }

/** Restore one resumable agent session checkpoint */
  async restore(agentId: string, sessionId: string, checkpointId: string, body: RestoreAgentSessionCheckpointRequest): Promise<AgentSessionCheckpointRecord> {
    return this.client.post<AgentSessionCheckpointRecord>(customApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/checkpoints/${serializePathParameter(checkpointId, { name: 'checkpointId', style: 'simple', explode: false })}/restore`), body, undefined, undefined, 'application/json');
  }

/** Invalidate one agent session checkpoint */
  async invalidate(agentId: string, sessionId: string, checkpointId: string, body: InvalidateAgentSessionCheckpointRequest): Promise<AgentSessionCheckpointRecord> {
    return this.client.post<AgentSessionCheckpointRecord>(customApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/checkpoints/${serializePathParameter(checkpointId, { name: 'checkpointId', style: 'simple', explode: false })}/invalidate`), body, undefined, undefined, 'application/json');
  }
}

export interface AiAgentsInteractionsListParams {
  page?: number;
  pageSize?: number;
  kind?: AgentInteractionKind;
  status?: AgentInteractionStatus;
}

export class AiAgentsInteractionsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List durable interactions for one agent session */
  async list(agentId: string, sessionId: string, params?: AiAgentsInteractionsListParams): Promise<SdkWorkPageData & { items: AgentInteractionRecord[]; }> {
    const query = buildQueryString([
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'kind', value: params?.kind, style: 'form', explode: true, allowReserved: false },
      { name: 'status', value: params?.status, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<SdkWorkPageData & { items: AgentInteractionRecord[]; }>(appendQueryString(customApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/interactions`), query));
  }

/** Create one durable approval or user-question interaction */
  async create(agentId: string, sessionId: string, body: CreateAgentInteractionRequest): Promise<AgentInteractionRecord> {
    return this.client.post<AgentInteractionRecord>(customApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/interactions`), body, undefined, undefined, 'application/json');
  }

/** Retrieve one durable agent interaction */
  async retrieve(agentId: string, sessionId: string, interactionId: string): Promise<AgentInteractionRecord> {
    return this.client.get<AgentInteractionRecord>(customApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/interactions/${serializePathParameter(interactionId, { name: 'interactionId', style: 'simple', explode: false })}`));
  }

/** Claim one pending agent interaction for exclusive resolution */
  async claim(agentId: string, sessionId: string, interactionId: string, body: ClaimAgentInteractionRequest): Promise<{ interaction: AgentInteractionRecord; claimToken: string; claimExpiresAt: string; fencingToken: Int64String; }> {
    return this.client.post<{ interaction: AgentInteractionRecord; claimToken: string; claimExpiresAt: string; fencingToken: Int64String; }>(customApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/interactions/${serializePathParameter(interactionId, { name: 'interactionId', style: 'simple', explode: false })}/claim`), body, undefined, undefined, 'application/json');
  }

/** Approve or reject one approval interaction */
  async approve(agentId: string, sessionId: string, interactionId: string, body: ApproveAgentInteractionRequest): Promise<AgentInteractionRecord> {
    return this.client.post<AgentInteractionRecord>(customApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/interactions/${serializePathParameter(interactionId, { name: 'interactionId', style: 'simple', explode: false })}/approve`), body, undefined, undefined, 'application/json');
  }

/** Answer or reject one user-question interaction */
  async answer(agentId: string, sessionId: string, interactionId: string, body: AnswerAgentInteractionRequest): Promise<AgentInteractionRecord> {
    return this.client.post<AgentInteractionRecord>(customApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/interactions/${serializePathParameter(interactionId, { name: 'interactionId', style: 'simple', explode: false })}/answer`), body, undefined, undefined, 'application/json');
  }
}

export interface AiAgentsTurnsListParams {
  page?: number;
  pageSize?: number;
}

export interface AiAgentsTurnsStreamParams {
  stream?: boolean;
}

export class AiAgentsTurnsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List durable turns for one agent session */
  async list(agentId: string, sessionId: string, params?: AiAgentsTurnsListParams): Promise<SdkWorkPageData & { items: AgentTurnRecord[]; }> {
    const query = buildQueryString([
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<SdkWorkPageData & { items: AgentTurnRecord[]; }>(appendQueryString(customApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/turns`), query));
  }

/** Create one idempotent agent turn */
  async stream(agentId: string, sessionId: string, body: CreateAgentTurnRequest, params?: AiAgentsTurnsStreamParams): Promise<AsyncIterable<AgentTurnStreamEvent>> {
    const query = buildQueryString([
      { name: 'stream', value: params?.stream, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.streamJson<AgentTurnStreamEvent>(appendQueryString(customApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/turns`), query), { method: 'POST' as any, body, contentType: 'application/json' });
  }

/** Retrieve one durable agent turn */
  async retrieve(agentId: string, sessionId: string, turnId: string): Promise<AgentTurnRecord> {
    return this.client.get<AgentTurnRecord>(customApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/turns/${serializePathParameter(turnId, { name: 'turnId', style: 'simple', explode: false })}`));
  }

/** Request cancellation of one agent turn */
  async cancel(agentId: string, sessionId: string, turnId: string, body: CancelAgentTurnRequest): Promise<AgentTurnRecord> {
    return this.client.post<AgentTurnRecord>(customApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/turns/${serializePathParameter(turnId, { name: 'turnId', style: 'simple', explode: false })}/cancel`), body, undefined, undefined, 'application/json');
  }
}

export interface AiAgentsSessionItemsListParams {
  page?: number;
  pageSize?: number;
  kind?: AgentSessionItemKind;
  status?: AgentSessionItemStatus;
  sort?: 'sequence' | '-sequence';
}

export class AiAgentsSessionItemsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List ordered items for one agent session */
  async list(agentId: string, sessionId: string, params?: AiAgentsSessionItemsListParams): Promise<SdkWorkPageData & { items: AgentSessionItemRecord[]; }> {
    const query = buildQueryString([
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'kind', value: params?.kind, style: 'form', explode: true, allowReserved: false },
      { name: 'status', value: params?.status, style: 'form', explode: true, allowReserved: false },
      { name: 'sort', value: params?.sort, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<SdkWorkPageData & { items: AgentSessionItemRecord[]; }>(appendQueryString(customApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/items`), query));
  }

/** Retrieve one agent session item */
  async retrieve(agentId: string, sessionId: string, itemId: string): Promise<AgentSessionItemRecord> {
    return this.client.get<AgentSessionItemRecord>(customApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/items/${serializePathParameter(itemId, { name: 'itemId', style: 'simple', explode: false })}`));
  }
}

export interface AiAgentsSessionsListParams {
  page?: number;
  pageSize?: number;
}

export class AiAgentsSessionsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List agent sessions for one managed agent */
  async list(agentId: string, params?: AiAgentsSessionsListParams): Promise<SdkWorkPageData & { items: AgentSessionRecord[]; }> {
    const query = buildQueryString([
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<SdkWorkPageData & { items: AgentSessionRecord[]; }>(appendQueryString(customApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions`), query));
  }

/** Create a agent session for one managed agent */
  async create(agentId: string, body: CreateAgentSessionRequest): Promise<AgentSessionRecord> {
    return this.client.post<AgentSessionRecord>(customApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions`), body, undefined, undefined, 'application/json');
  }

/** Retrieve one agent session */
  async retrieve(agentId: string, sessionId: string): Promise<AgentSessionRecord> {
    return this.client.get<AgentSessionRecord>(customApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}`));
  }

/** Close one agent session */
  async close(agentId: string, sessionId: string, body: CloseAgentSessionRequest): Promise<AgentSessionRecord> {
    return this.client.post<AgentSessionRecord>(customApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/close`), body, undefined, undefined, 'application/json');
  }
}

export class AiAgentsPromptOptimizationsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Create a prompt optimization for one managed agent */
  async create(agentId: string, body: CreateAgentPromptOptimizationRequest): Promise<AgentRuntimeExecutionRecord> {
    return this.client.post<AgentRuntimeExecutionRecord>(customApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/prompt_optimizations`), body, undefined, undefined, 'application/json');
  }
}

export class AiAgentsPreviewResponsesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Create a preview response for one managed agent */
  async create(agentId: string, body: CreateAgentPreviewResponseRequest): Promise<AgentRuntimeExecutionRecord> {
    return this.client.post<AgentRuntimeExecutionRecord>(customApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/preview_responses`), body, undefined, undefined, 'application/json');
  }
}

export interface AiAgentsProviderBindingsListParams {
  page?: number;
  pageSize?: number;
}

export class AiAgentsProviderBindingsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List provider bindings for one managed agent */
  async list(agentId: string, params?: AiAgentsProviderBindingsListParams): Promise<SdkWorkPageData & { items: AgentProviderBindingRecord[]; }> {
    const query = buildQueryString([
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<SdkWorkPageData & { items: AgentProviderBindingRecord[]; }>(appendQueryString(customApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/provider_bindings`), query));
  }

/** Create a provider binding for one managed agent */
  async create(agentId: string, body: CreateAgentProviderBindingRequest): Promise<AgentProviderBindingRecord> {
    return this.client.post<AgentProviderBindingRecord>(customApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/provider_bindings`), body, undefined, undefined, 'application/json');
  }

/** Activate one managed agent provider binding */
  async activate(agentId: string, bindingId: string, body: ActivateAgentProviderBindingRequest): Promise<AgentProviderBindingRecord> {
    return this.client.post<AgentProviderBindingRecord>(customApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/provider_bindings/${serializePathParameter(bindingId, { name: 'bindingId', style: 'simple', explode: false })}/activate`), body, undefined, undefined, 'application/json');
  }
}

export interface AiAgentsListParams {
  includeDeleted?: boolean;
  page?: number;
  pageSize?: number;
  q?: string;
}

export class AiAgentsApi {
  private client: HttpClient;
  public readonly providerBindings: AiAgentsProviderBindingsApi;
  public readonly previewResponses: AiAgentsPreviewResponsesApi;
  public readonly promptOptimizations: AiAgentsPromptOptimizationsApi;
  public readonly sessions: AiAgentsSessionsApi;
  public readonly sessionItems: AiAgentsSessionItemsApi;
  public readonly turns: AiAgentsTurnsApi;
  public readonly interactions: AiAgentsInteractionsApi;
  public readonly checkpoints: AiAgentsCheckpointsApi;
  public readonly sessionRuntimeBindings: AiAgentsSessionRuntimeBindingsApi;
  public readonly tasks: AiAgentsTasksApi;
  public readonly compositionSlots: AiAgentsCompositionSlotsApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.providerBindings = new AiAgentsProviderBindingsApi(client);
    this.previewResponses = new AiAgentsPreviewResponsesApi(client);
    this.promptOptimizations = new AiAgentsPromptOptimizationsApi(client);
    this.sessions = new AiAgentsSessionsApi(client);
    this.sessionItems = new AiAgentsSessionItemsApi(client);
    this.turns = new AiAgentsTurnsApi(client);
    this.interactions = new AiAgentsInteractionsApi(client);
    this.checkpoints = new AiAgentsCheckpointsApi(client);
    this.sessionRuntimeBindings = new AiAgentsSessionRuntimeBindingsApi(client);
    this.tasks = new AiAgentsTasksApi(client);
    this.compositionSlots = new AiAgentsCompositionSlotsApi(client);
  }


/** List managed agents */
  async list(params?: AiAgentsListParams): Promise<SdkWorkPageData & { items: AgentRecord[]; }> {
    const query = buildQueryString([
      { name: 'include_deleted', value: params?.includeDeleted, style: 'form', explode: true, allowReserved: false },
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'q', value: params?.q, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<SdkWorkPageData & { items: AgentRecord[]; }>(appendQueryString(customApiPath(`/ai/agents`), query));
  }

/** Create a managed agent */
  async create(body: CreateAgentRequest): Promise<AgentRecord> {
    return this.client.post<AgentRecord>(customApiPath(`/ai/agents`), body, undefined, undefined, 'application/json');
  }

/** Retrieve one managed agent */
  async retrieve(agentId: string): Promise<AgentRecord> {
    return this.client.get<AgentRecord>(customApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}`));
  }

/** Update one managed agent */
  async update(agentId: string, body: UpdateAgentRequest): Promise<AgentRecord> {
    return this.client.patch<AgentRecord>(customApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}`), body, undefined, undefined, 'application/json');
  }

/** Soft-delete one managed agent */
  async delete(agentId: string): Promise<void> {
    return this.client.delete<void>(customApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}`));
  }
}

export class AiApi {

  public readonly agents: AiAgentsApi;

  constructor(client: HttpClient) {

    this.agents = new AiAgentsApi(client);
  }

}

export function createAiApi(client: HttpClient): AiApi {
  return new AiApi(client);
}

function appendQueryString(path: string, rawQueryString: string): string {
  const query = rawQueryString.replace(/^\?+/, '');
  if (!query) {
    return path;
  }
  return path.includes('?') ? `${path}&${query}` : `${path}?${query}`;
}

interface PathParameterSpec {
  name: string;
  style: string;
  explode: boolean;
}

function serializePathParameter(value: unknown, spec: PathParameterSpec): string {
  if (value === undefined || value === null) {
    return '';
  }

  const style = spec.style || 'simple';
  if (Array.isArray(value)) {
    return serializePathArray(spec.name, value, style, spec.explode);
  }
  if (typeof value === 'object') {
    return serializePathObject(spec.name, value as Record<string, unknown>, style, spec.explode);
  }
  return pathPrefix(spec.name, style, false) + encodePathValue(serializePathPrimitive(value));
}

function serializePathArray(name: string, values: unknown[], style: string, explode: boolean): string {
  const serialized = values
    .filter((item) => item !== undefined && item !== null)
    .map((item) => encodePathValue(serializePathPrimitive(item)));
  if (serialized.length === 0) {
    return pathPrefix(name, style, false);
  }
  if (style === 'matrix') {
    return explode
      ? serialized.map((item) => `;${name}=${item}`).join('')
      : `;${name}=${serialized.join(',')}`;
  }
  return pathPrefix(name, style, false) + serialized.join(explode ? '.' : ',');
}

function serializePathObject(name: string, value: Record<string, unknown>, style: string, explode: boolean): string {
  const entries = Object.entries(value).filter(([, entryValue]) => entryValue !== undefined && entryValue !== null);
  if (entries.length === 0) {
    return pathPrefix(name, style, true);
  }
  if (style === 'matrix') {
    return explode
      ? entries.map(([key, entryValue]) => `;${encodePathValue(key)}=${encodePathValue(serializePathPrimitive(entryValue))}`).join('')
      : `;${name}=${entries.flatMap(([key, entryValue]) => [encodePathValue(key), encodePathValue(serializePathPrimitive(entryValue))]).join(',')}`;
  }
  const serialized = explode
    ? entries.map(([key, entryValue]) => `${encodePathValue(key)}=${encodePathValue(serializePathPrimitive(entryValue))}`).join(style === 'label' ? '.' : ',')
    : entries.flatMap(([key, entryValue]) => [encodePathValue(key), encodePathValue(serializePathPrimitive(entryValue))]).join(',');
  return pathPrefix(name, style, true) + serialized;
}

function pathPrefix(name: string, style: string, _objectValue: boolean): string {
  if (style === 'label') return '.';
  if (style === 'matrix') return `;${name}`;
  return '';
}

function encodePathValue(value: string): string {
  return encodeURIComponent(value);
}

function serializePathPrimitive(value: unknown): string {
  if (value instanceof Date) {
    return value.toISOString();
  }
  if (typeof value === 'object') {
    return JSON.stringify(value);
  }
  return String(value);
}
interface QueryParameterSpec {
  name: string;
  value: unknown;
  style: string;
  explode: boolean;
  allowReserved: boolean;
  contentType?: string;
}

function buildQueryString(parameters: QueryParameterSpec[]): string {
  const pairs: string[] = [];
  for (const parameter of parameters) {
    appendSerializedParameter(pairs, parameter);
  }
  return pairs.join('&');
}

function appendSerializedParameter(pairs: string[], parameter: QueryParameterSpec): void {
  if (parameter.value === undefined || parameter.value === null) {
    return;
  }

  if (parameter.contentType) {
    pairs.push(`${encodeQueryComponent(parameter.name)}=${encodeQueryValue(JSON.stringify(parameter.value), parameter.allowReserved)}`);
    return;
  }

  const style = parameter.style || 'form';
  if (style === 'deepObject') {
    appendDeepObjectParameter(pairs, parameter.name, parameter.value, parameter.allowReserved);
    return;
  }

  if (Array.isArray(parameter.value)) {
    appendArrayParameter(pairs, parameter.name, parameter.value, style, parameter.explode, parameter.allowReserved);
    return;
  }

  if (typeof parameter.value === 'object') {
    appendObjectParameter(pairs, parameter.name, parameter.value as Record<string, unknown>, style, parameter.explode, parameter.allowReserved);
    return;
  }

  pairs.push(`${encodeQueryComponent(parameter.name)}=${encodeQueryValue(serializePrimitive(parameter.value), parameter.allowReserved)}`);
}

function appendArrayParameter(
  pairs: string[],
  name: string,
  value: unknown[],
  style: string,
  explode: boolean,
  allowReserved: boolean,
): void {
  const values = value
    .filter((item) => item !== undefined && item !== null)
    .map((item) => serializePrimitive(item));
  if (values.length === 0) {
    return;
  }

  if (style === 'form' && explode) {
    for (const item of values) {
      pairs.push(`${encodeQueryComponent(name)}=${encodeQueryValue(item, allowReserved)}`);
    }
    return;
  }

  pairs.push(`${encodeQueryComponent(name)}=${encodeQueryValue(values.join(','), allowReserved)}`);
}

function appendObjectParameter(
  pairs: string[],
  name: string,
  value: Record<string, unknown>,
  style: string,
  explode: boolean,
  allowReserved: boolean,
): void {
  const entries = Object.entries(value).filter(([, entryValue]) => entryValue !== undefined && entryValue !== null);
  if (entries.length === 0) {
    return;
  }

  if (style === 'form' && explode) {
    for (const [key, entryValue] of entries) {
      pairs.push(`${encodeQueryComponent(key)}=${encodeQueryValue(serializePrimitive(entryValue), allowReserved)}`);
    }
    return;
  }

  const serialized = entries.flatMap(([key, entryValue]) => [key, serializePrimitive(entryValue)]).join(',');
  pairs.push(`${encodeQueryComponent(name)}=${encodeQueryValue(serialized, allowReserved)}`);
}

function appendDeepObjectParameter(
  pairs: string[],
  name: string,
  value: unknown,
  allowReserved: boolean,
): void {
  if (!value || typeof value !== 'object' || Array.isArray(value)) {
    pairs.push(`${encodeQueryComponent(name)}=${encodeQueryValue(serializePrimitive(value), allowReserved)}`);
    return;
  }

  for (const [key, entryValue] of Object.entries(value as Record<string, unknown>)) {
    if (entryValue === undefined || entryValue === null) {
      continue;
    }
    pairs.push(`${encodeQueryComponent(`${name}[${key}]`)}=${encodeQueryValue(serializePrimitive(entryValue), allowReserved)}`);
  }
}

function serializePrimitive(value: unknown): string {
  if (value instanceof Date) {
    return value.toISOString();
  }
  if (typeof value === 'object') {
    return JSON.stringify(value);
  }
  return String(value);
}

function encodeQueryComponent(value: string): string {
  return encodeURIComponent(value);
}

function encodeQueryValue(value: string, allowReserved: boolean): string {
  const encoded = encodeURIComponent(value);
  if (!allowReserved) {
    return encoded;
  }
  return encoded.replace(/%3A/gi, ':')
    .replace(/%2F/gi, '/')
    .replace(/%3F/gi, '?')
    .replace(/%23/gi, '#')
    .replace(/%5B/gi, '[')
    .replace(/%5D/gi, ']')
    .replace(/%40/gi, '@')
    .replace(/%21/gi, '!')
    .replace(/%24/gi, '$')
    .replace(/%26/gi, '&')
    .replace(/%27/gi, "'")
    .replace(/%28/gi, '(')
    .replace(/%29/gi, ')')
    .replace(/%2A/gi, '*')
    .replace(/%2B/gi, '+')
    .replace(/%2C/gi, ',')
    .replace(/%3B/gi, ';')
    .replace(/%3D/gi, '=');
}
