import { agentApiPath } from './paths';
import type { HttpClient } from '../http/client';

import type { ActivateAgentProviderBindingRequest, AgentCompositionSlotKind, AgentCompositionSlotRecord, AgentInteractionRecord, AgentItemFeedbackRecord, AgentProjectCompositionSlotRecord, AgentProjectMutationRequest, AgentProjectRecord, AgentProjectStatus, AgentProviderBindingRecord, AgentRecord, AgentResourceUserStateRecord, AgentRuntimeExecutionRecord, AgentSessionCheckpointRecord, AgentSessionItemRecord, AgentSessionRecord, AgentSessionRuntimeBindingRecord, AgentTaskRecord, AgentTurnRecord, AnswerAgentInteractionRequest, ApproveAgentInteractionRequest, AppUpdateAgentSessionRequest, CancelAgentTaskRequest, CancelAgentTurnRequest, ChangeAgentSessionRuntimeBindingStatusRequest, ClaimAgentInteractionRequest, CloseAgentSessionRequest, CodeEngineCatalog, CreateAgentCompositionSlotRequest, CreateAgentInteractionRequest, CreateAgentPreviewResponseRequest, CreateAgentProjectCompositionSlotRequest, CreateAgentProjectRequest, CreateAgentPromptOptimizationRequest, CreateAgentProviderBindingRequest, CreateAgentRequest, CreateAgentSessionCheckpointRequest, CreateAgentSessionRequest, CreateAgentSessionRuntimeBindingRequest, CreateAgentTaskRequest, CreateAgentTurnRequest, Int64String, InvalidateAgentSessionCheckpointRequest, McpServerMarketplaceRecord, RestoreAgentSessionCheckpointRequest, SdkWorkPageData, SdkWorkResourceData, UpdateAgentCompositionSlotRequest, UpdateAgentItemFeedbackRequest, UpdateAgentProjectCompositionSlotRequest, UpdateAgentProjectRequest, UpdateAgentRequest, UpdateAgentSessionRuntimeBindingRequest, UpdateAgentSessionUserStateRequest } from '../types';


export interface AiAgentsMcpServersListParams {
  page?: number;
  pageSize?: number;
  q?: string;
}



export interface AiAgentsCompositionSlotsListParams {
  page?: number;
  pageSize?: number;
}

export class AiAgentsCompositionSlotsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List composition slots for one managed agent */
  async list(agentId: string, params?: AiAgentsCompositionSlotsListParams): Promise<SdkWorkPageData & { items: AgentCompositionSlotRecord[]; }> {
    const query = buildQueryString([
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<SdkWorkPageData & { items: AgentCompositionSlotRecord[]; }>(appendQueryString(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/composition_slots`), query));
  }

/** Create a composition slot for one managed agent */
  async create(agentId: string, body: CreateAgentCompositionSlotRequest): Promise<SdkWorkResourceData & { item: AgentCompositionSlotRecord; }> {
    return this.client.post<SdkWorkResourceData & { item: AgentCompositionSlotRecord; }>(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/composition_slots`), body, undefined, undefined, 'application/json');
  }

/** Retrieve one managed agent composition slot */
  async retrieve(agentId: string, slotId: string): Promise<SdkWorkResourceData & { item: AgentCompositionSlotRecord; }> {
    return this.client.get<SdkWorkResourceData & { item: AgentCompositionSlotRecord; }>(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/composition_slots/${serializePathParameter(slotId, { name: 'slotId', style: 'simple', explode: false })}`));
  }

/** Update one managed agent composition slot */
  async update(agentId: string, slotId: string, body: UpdateAgentCompositionSlotRequest): Promise<SdkWorkResourceData & { item: AgentCompositionSlotRecord; }> {
    return this.client.patch<SdkWorkResourceData & { item: AgentCompositionSlotRecord; }>(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/composition_slots/${serializePathParameter(slotId, { name: 'slotId', style: 'simple', explode: false })}`), body, undefined, undefined, 'application/json');
  }

/** Delete one managed agent composition slot */
  async delete(agentId: string, slotId: string): Promise<void> {
    return this.client.delete<void>(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/composition_slots/${serializePathParameter(slotId, { name: 'slotId', style: 'simple', explode: false })}`));
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
    return this.client.get<SdkWorkPageData & { items: AgentTaskRecord[]; }>(appendQueryString(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/tasks`), query));
  }

/** Create a scheduled task for one managed agent */
  async create(agentId: string, body: CreateAgentTaskRequest): Promise<SdkWorkResourceData & { item: AgentTaskRecord; }> {
    return this.client.post<SdkWorkResourceData & { item: AgentTaskRecord; }>(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/tasks`), body, undefined, undefined, 'application/json');
  }

/** Retrieve one scheduled task */
  async retrieve(agentId: string, taskId: string): Promise<SdkWorkResourceData & { item: AgentTaskRecord; }> {
    return this.client.get<SdkWorkResourceData & { item: AgentTaskRecord; }>(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/tasks/${serializePathParameter(taskId, { name: 'taskId', style: 'simple', explode: false })}`));
  }

/** Cancel one scheduled task */
  async cancel(agentId: string, taskId: string, body: CancelAgentTaskRequest): Promise<SdkWorkResourceData & { item: AgentTaskRecord; }> {
    return this.client.post<SdkWorkResourceData & { item: AgentTaskRecord; }>(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/tasks/${serializePathParameter(taskId, { name: 'taskId', style: 'simple', explode: false })}/cancel`), body, undefined, undefined, 'application/json');
  }

/** Execute one deferred scheduled task */
  async execute(agentId: string, taskId: string, body: CancelAgentTaskRequest): Promise<SdkWorkResourceData & { item: AgentTaskRecord; }> {
    return this.client.post<SdkWorkResourceData & { item: AgentTaskRecord; }>(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/tasks/${serializePathParameter(taskId, { name: 'taskId', style: 'simple', explode: false })}/execute`), body, undefined, undefined, 'application/json');
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
    return this.client.get<SdkWorkPageData & { items: AgentSessionRuntimeBindingRecord[]; }>(appendQueryString(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/runtime_bindings`), query));
  }

/** Create the current runtime binding for one agent session */
  async create(agentId: string, sessionId: string, body: CreateAgentSessionRuntimeBindingRequest): Promise<SdkWorkResourceData & { item: AgentSessionRuntimeBindingRecord; }> {
    return this.client.post<SdkWorkResourceData & { item: AgentSessionRuntimeBindingRecord; }>(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/runtime_bindings`), body, undefined, undefined, 'application/json');
  }

/** Retrieve one agent session runtime binding */
  async retrieve(agentId: string, sessionId: string, runtimeBindingId: string): Promise<SdkWorkResourceData & { item: AgentSessionRuntimeBindingRecord; }> {
    return this.client.get<SdkWorkResourceData & { item: AgentSessionRuntimeBindingRecord; }>(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/runtime_bindings/${serializePathParameter(runtimeBindingId, { name: 'runtimeBindingId', style: 'simple', explode: false })}`));
  }

/** Update one agent session runtime binding */
  async update(agentId: string, sessionId: string, runtimeBindingId: string, body: UpdateAgentSessionRuntimeBindingRequest): Promise<SdkWorkResourceData & { item: AgentSessionRuntimeBindingRecord; }> {
    return this.client.patch<SdkWorkResourceData & { item: AgentSessionRuntimeBindingRecord; }>(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/runtime_bindings/${serializePathParameter(runtimeBindingId, { name: 'runtimeBindingId', style: 'simple', explode: false })}`), body, undefined, undefined, 'application/json');
  }

/** Activate one agent session runtime binding as current */
  async activate(agentId: string, sessionId: string, runtimeBindingId: string, body: ChangeAgentSessionRuntimeBindingStatusRequest): Promise<SdkWorkResourceData & { item: AgentSessionRuntimeBindingRecord; }> {
    return this.client.post<SdkWorkResourceData & { item: AgentSessionRuntimeBindingRecord; }>(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/runtime_bindings/${serializePathParameter(runtimeBindingId, { name: 'runtimeBindingId', style: 'simple', explode: false })}/activate`), body, undefined, undefined, 'application/json');
  }

/** Deactivate one agent session runtime binding */
  async deactivate(agentId: string, sessionId: string, runtimeBindingId: string, body: ChangeAgentSessionRuntimeBindingStatusRequest): Promise<SdkWorkResourceData & { item: AgentSessionRuntimeBindingRecord; }> {
    return this.client.post<SdkWorkResourceData & { item: AgentSessionRuntimeBindingRecord; }>(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/runtime_bindings/${serializePathParameter(runtimeBindingId, { name: 'runtimeBindingId', style: 'simple', explode: false })}/deactivate`), body, undefined, undefined, 'application/json');
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
    return this.client.get<SdkWorkPageData & { items: AgentSessionCheckpointRecord[]; }>(appendQueryString(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/checkpoints`), query));
  }

/** Create one bounded agent session checkpoint */
  async create(agentId: string, sessionId: string, body: CreateAgentSessionCheckpointRequest): Promise<SdkWorkResourceData & { item: AgentSessionCheckpointRecord; }> {
    return this.client.post<SdkWorkResourceData & { item: AgentSessionCheckpointRecord; }>(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/checkpoints`), body, undefined, undefined, 'application/json');
  }

/** Retrieve one agent session checkpoint */
  async retrieve(agentId: string, sessionId: string, checkpointId: string): Promise<SdkWorkResourceData & { item: AgentSessionCheckpointRecord; }> {
    return this.client.get<SdkWorkResourceData & { item: AgentSessionCheckpointRecord; }>(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/checkpoints/${serializePathParameter(checkpointId, { name: 'checkpointId', style: 'simple', explode: false })}`));
  }

/** Restore one resumable agent session checkpoint */
  async restore(agentId: string, sessionId: string, checkpointId: string, body: RestoreAgentSessionCheckpointRequest): Promise<SdkWorkResourceData & { item: AgentSessionCheckpointRecord; }> {
    return this.client.post<SdkWorkResourceData & { item: AgentSessionCheckpointRecord; }>(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/checkpoints/${serializePathParameter(checkpointId, { name: 'checkpointId', style: 'simple', explode: false })}/restore`), body, undefined, undefined, 'application/json');
  }

/** Invalidate one agent session checkpoint */
  async invalidate(agentId: string, sessionId: string, checkpointId: string, body: InvalidateAgentSessionCheckpointRequest): Promise<SdkWorkResourceData & { item: AgentSessionCheckpointRecord; }> {
    return this.client.post<SdkWorkResourceData & { item: AgentSessionCheckpointRecord; }>(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/checkpoints/${serializePathParameter(checkpointId, { name: 'checkpointId', style: 'simple', explode: false })}/invalidate`), body, undefined, undefined, 'application/json');
  }
}

export interface AiAgentsInteractionsListParams {
  page?: number;
  pageSize?: number;
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
    ]);
    return this.client.get<SdkWorkPageData & { items: AgentInteractionRecord[]; }>(appendQueryString(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/interactions`), query));
  }

/** Create one durable approval or user-question interaction */
  async create(agentId: string, sessionId: string, body: CreateAgentInteractionRequest): Promise<SdkWorkResourceData & { item: AgentInteractionRecord; }> {
    return this.client.post<SdkWorkResourceData & { item: AgentInteractionRecord; }>(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/interactions`), body, undefined, undefined, 'application/json');
  }

/** Retrieve one durable agent interaction */
  async retrieve(agentId: string, sessionId: string, interactionId: string): Promise<SdkWorkResourceData & { item: AgentInteractionRecord; }> {
    return this.client.get<SdkWorkResourceData & { item: AgentInteractionRecord; }>(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/interactions/${serializePathParameter(interactionId, { name: 'interactionId', style: 'simple', explode: false })}`));
  }

/** Claim one pending agent interaction for exclusive resolution */
  async claim(agentId: string, sessionId: string, interactionId: string, body: ClaimAgentInteractionRequest): Promise<SdkWorkResourceData & { item: { interaction: AgentInteractionRecord; claimToken: string; claimExpiresAt: string; fencingToken: Int64String; }; }> {
    return this.client.post<SdkWorkResourceData & { item: { interaction: AgentInteractionRecord; claimToken: string; claimExpiresAt: string; fencingToken: Int64String; }; }>(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/interactions/${serializePathParameter(interactionId, { name: 'interactionId', style: 'simple', explode: false })}/claim`), body, undefined, undefined, 'application/json');
  }

/** Approve or reject one approval interaction */
  async approve(agentId: string, sessionId: string, interactionId: string, body: ApproveAgentInteractionRequest): Promise<SdkWorkResourceData & { item: AgentInteractionRecord; }> {
    return this.client.post<SdkWorkResourceData & { item: AgentInteractionRecord; }>(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/interactions/${serializePathParameter(interactionId, { name: 'interactionId', style: 'simple', explode: false })}/approve`), body, undefined, undefined, 'application/json');
  }

/** Answer or reject one user-question interaction */
  async answer(agentId: string, sessionId: string, interactionId: string, body: AnswerAgentInteractionRequest): Promise<SdkWorkResourceData & { item: AgentInteractionRecord; }> {
    return this.client.post<SdkWorkResourceData & { item: AgentInteractionRecord; }>(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/interactions/${serializePathParameter(interactionId, { name: 'interactionId', style: 'simple', explode: false })}/answer`), body, undefined, undefined, 'application/json');
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
    return this.client.get<SdkWorkPageData & { items: AgentTurnRecord[]; }>(appendQueryString(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/turns`), query));
  }

/** Create one idempotent agent turn */
  async stream(agentId: string, sessionId: string, body: CreateAgentTurnRequest, params?: AiAgentsTurnsStreamParams): Promise<AsyncIterable<string>> {
    const query = buildQueryString([
      { name: 'stream', value: params?.stream, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.streamJson<string>(appendQueryString(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/turns`), query), { method: 'POST' as any, body, contentType: 'application/json' });
  }

/** Retrieve one durable agent turn */
  async retrieve(agentId: string, sessionId: string, turnId: string): Promise<SdkWorkResourceData & { item: AgentTurnRecord; }> {
    return this.client.get<SdkWorkResourceData & { item: AgentTurnRecord; }>(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/turns/${serializePathParameter(turnId, { name: 'turnId', style: 'simple', explode: false })}`));
  }

/** Request cancellation of one agent turn */
  async cancel(agentId: string, sessionId: string, turnId: string, body: CancelAgentTurnRequest): Promise<SdkWorkResourceData & { item: AgentTurnRecord; }> {
    return this.client.post<SdkWorkResourceData & { item: AgentTurnRecord; }>(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/turns/${serializePathParameter(turnId, { name: 'turnId', style: 'simple', explode: false })}/cancel`), body, undefined, undefined, 'application/json');
  }
}

export interface AiAgentsSessionItemsListParams {
  page?: number;
  pageSize?: number;
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
    ]);
    return this.client.get<SdkWorkPageData & { items: AgentSessionItemRecord[]; }>(appendQueryString(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/items`), query));
  }

/** Retrieve one agent session item */
  async retrieve(agentId: string, sessionId: string, itemId: string): Promise<SdkWorkResourceData & { item: AgentSessionItemRecord; }> {
    return this.client.get<SdkWorkResourceData & { item: AgentSessionItemRecord; }>(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/items/${serializePathParameter(itemId, { name: 'itemId', style: 'simple', explode: false })}`));
  }
}

export interface AiAgentsItemFeedbackListParams {
  page?: number;
  pageSize?: number;
}

export class AiAgentsItemFeedbackApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List item feedback for one agent session */
  async list(agentId: string, sessionId: string, params?: AiAgentsItemFeedbackListParams): Promise<SdkWorkPageData & { items: AgentItemFeedbackRecord[]; }> {
    const query = buildQueryString([
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<SdkWorkPageData & { items: AgentItemFeedbackRecord[]; }>(appendQueryString(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/item_feedback`), query));
  }

/** Create, update, or clear feedback for one agent session item */
  async update(agentId: string, sessionId: string, itemId: string, body: UpdateAgentItemFeedbackRequest): Promise<SdkWorkResourceData & { item: AgentItemFeedbackRecord; }> {
    return this.client.patch<SdkWorkResourceData & { item: AgentItemFeedbackRecord; }>(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/items/${serializePathParameter(itemId, { name: 'itemId', style: 'simple', explode: false })}/feedback`), body, undefined, undefined, 'application/json');
  }
}

export interface AiAgentsSessionUserStatesListParams {
  page?: number;
  pageSize?: number;
  pinnedOnly?: boolean;
  includeHidden?: boolean;
}

export class AiAgentsSessionUserStatesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List per-user state for agent sessions owned by the authenticated user */
  async list(agentId: string, params?: AiAgentsSessionUserStatesListParams): Promise<SdkWorkPageData & { items: AgentResourceUserStateRecord[]; }> {
    const query = buildQueryString([
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'pinnedOnly', value: params?.pinnedOnly, style: 'form', explode: true, allowReserved: false },
      { name: 'includeHidden', value: params?.includeHidden, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<SdkWorkPageData & { items: AgentResourceUserStateRecord[]; }>(appendQueryString(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/user_states`), query));
  }

/** Retrieve the authenticated user's state for one agent session */
  async retrieve(agentId: string, sessionId: string): Promise<SdkWorkResourceData & { item: AgentResourceUserStateRecord; }> {
    return this.client.get<SdkWorkResourceData & { item: AgentResourceUserStateRecord; }>(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/user_state`));
  }

/** Update the authenticated user's state for one agent session */
  async update(agentId: string, sessionId: string, body: UpdateAgentSessionUserStateRequest): Promise<SdkWorkResourceData & { item: AgentResourceUserStateRecord; }> {
    return this.client.patch<SdkWorkResourceData & { item: AgentResourceUserStateRecord; }>(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/user_state`), body, undefined, undefined, 'application/json');
  }
}

export interface AiAgentsSessionsListParams {
  page?: number;
  pageSize?: number;
  projectId?: string;
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
      { name: 'projectId', value: params?.projectId, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<SdkWorkPageData & { items: AgentSessionRecord[]; }>(appendQueryString(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions`), query));
  }

/** Create a agent session for one managed agent */
  async create(agentId: string, body: CreateAgentSessionRequest): Promise<SdkWorkResourceData & { item: AgentSessionRecord; }> {
    return this.client.post<SdkWorkResourceData & { item: AgentSessionRecord; }>(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions`), body, undefined, undefined, 'application/json');
  }

/** Retrieve one agent session */
  async retrieve(agentId: string, sessionId: string): Promise<SdkWorkResourceData & { item: AgentSessionRecord; }> {
    return this.client.get<SdkWorkResourceData & { item: AgentSessionRecord; }>(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}`));
  }

/** Rename or move one agent session */
  async update(agentId: string, sessionId: string, body: AppUpdateAgentSessionRequest): Promise<SdkWorkResourceData & { item: AgentSessionRecord; }> {
    return this.client.patch<SdkWorkResourceData & { item: AgentSessionRecord; }>(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}`), body, undefined, undefined, 'application/json');
  }

/** Soft delete one agent session */
  async delete(agentId: string, sessionId: string): Promise<void> {
    return this.client.delete<void>(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}`));
  }

/** Close one agent session */
  async close(agentId: string, sessionId: string, body: CloseAgentSessionRequest): Promise<SdkWorkResourceData & { item: AgentSessionRecord; }> {
    return this.client.post<SdkWorkResourceData & { item: AgentSessionRecord; }>(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/close`), body, undefined, undefined, 'application/json');
  }
}

export interface AiAgentsProjectCompositionSlotsListParams {
  page?: number;
  pageSize?: number;
  slotKind?: AgentCompositionSlotKind;
  enabled?: boolean;
}

export interface AiAgentsProjectCompositionSlotsDeleteParams {
  expectedVersion: Int64String;
}

export class AiAgentsProjectCompositionSlotsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List composition slots for an agent project */
  async list(projectId: string, params?: AiAgentsProjectCompositionSlotsListParams): Promise<SdkWorkPageData & { items: AgentProjectCompositionSlotRecord[]; }> {
    const query = buildQueryString([
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'slotKind', value: params?.slotKind, style: 'form', explode: true, allowReserved: false },
      { name: 'enabled', value: params?.enabled, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<SdkWorkPageData & { items: AgentProjectCompositionSlotRecord[]; }>(appendQueryString(agentApiPath(`/ai/projects/${serializePathParameter(projectId, { name: 'projectId', style: 'simple', explode: false })}/composition_slots`), query));
  }

/** Add a composition slot to an agent project */
  async create(projectId: string, body: CreateAgentProjectCompositionSlotRequest): Promise<SdkWorkResourceData & { item: AgentProjectCompositionSlotRecord; }> {
    return this.client.post<SdkWorkResourceData & { item: AgentProjectCompositionSlotRecord; }>(agentApiPath(`/ai/projects/${serializePathParameter(projectId, { name: 'projectId', style: 'simple', explode: false })}/composition_slots`), body, undefined, undefined, 'application/json');
  }

/** Retrieve a project composition slot */
  async retrieve(projectId: string, slotId: string): Promise<SdkWorkResourceData & { item: AgentProjectCompositionSlotRecord; }> {
    return this.client.get<SdkWorkResourceData & { item: AgentProjectCompositionSlotRecord; }>(agentApiPath(`/ai/projects/${serializePathParameter(projectId, { name: 'projectId', style: 'simple', explode: false })}/composition_slots/${serializePathParameter(slotId, { name: 'slotId', style: 'simple', explode: false })}`));
  }

/** Update a project composition slot */
  async update(projectId: string, slotId: string, body: UpdateAgentProjectCompositionSlotRequest): Promise<SdkWorkResourceData & { item: AgentProjectCompositionSlotRecord; }> {
    return this.client.patch<SdkWorkResourceData & { item: AgentProjectCompositionSlotRecord; }>(agentApiPath(`/ai/projects/${serializePathParameter(projectId, { name: 'projectId', style: 'simple', explode: false })}/composition_slots/${serializePathParameter(slotId, { name: 'slotId', style: 'simple', explode: false })}`), body, undefined, undefined, 'application/json');
  }

/** Soft-delete a project composition slot */
  async delete(projectId: string, slotId: string, params: AiAgentsProjectCompositionSlotsDeleteParams): Promise<void> {
    const query = buildQueryString([
      { name: 'expected_version', value: params.expectedVersion, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.delete<void>(appendQueryString(agentApiPath(`/ai/projects/${serializePathParameter(projectId, { name: 'projectId', style: 'simple', explode: false })}/composition_slots/${serializePathParameter(slotId, { name: 'slotId', style: 'simple', explode: false })}`), query));
  }
}

export interface AiAgentsProjectsListParams {
  page?: number;
  pageSize?: number;
  q?: string;
  status?: AgentProjectStatus;
  includeDeleted?: boolean;
}

export class AiAgentsProjectsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List agent projects for the current user */
  async list(params?: AiAgentsProjectsListParams): Promise<SdkWorkPageData & { items: AgentProjectRecord[]; }> {
    const query = buildQueryString([
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'q', value: params?.q, style: 'form', explode: true, allowReserved: false },
      { name: 'status', value: params?.status, style: 'form', explode: true, allowReserved: false },
      { name: 'includeDeleted', value: params?.includeDeleted, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<SdkWorkPageData & { items: AgentProjectRecord[]; }>(appendQueryString(agentApiPath(`/ai/projects`), query));
  }

/** Create an agent project */
  async create(body: CreateAgentProjectRequest): Promise<SdkWorkResourceData & { item: AgentProjectRecord; }> {
    return this.client.post<SdkWorkResourceData & { item: AgentProjectRecord; }>(agentApiPath(`/ai/projects`), body, undefined, undefined, 'application/json');
  }

/** Retrieve an agent project */
  async retrieve(projectId: string): Promise<SdkWorkResourceData & { item: AgentProjectRecord; }> {
    return this.client.get<SdkWorkResourceData & { item: AgentProjectRecord; }>(agentApiPath(`/ai/projects/${serializePathParameter(projectId, { name: 'projectId', style: 'simple', explode: false })}`));
  }

/** Update an agent project */
  async update(projectId: string, body: UpdateAgentProjectRequest): Promise<SdkWorkResourceData & { item: AgentProjectRecord; }> {
    return this.client.patch<SdkWorkResourceData & { item: AgentProjectRecord; }>(agentApiPath(`/ai/projects/${serializePathParameter(projectId, { name: 'projectId', style: 'simple', explode: false })}`), body, undefined, undefined, 'application/json');
  }

/** Soft-delete an agent project */
  async delete(projectId: string): Promise<void> {
    return this.client.delete<void>(agentApiPath(`/ai/projects/${serializePathParameter(projectId, { name: 'projectId', style: 'simple', explode: false })}`));
  }

/** Archive an agent project */
  async archive(projectId: string, body: AgentProjectMutationRequest): Promise<SdkWorkResourceData & { item: AgentProjectRecord; }> {
    return this.client.post<SdkWorkResourceData & { item: AgentProjectRecord; }>(agentApiPath(`/ai/projects/${serializePathParameter(projectId, { name: 'projectId', style: 'simple', explode: false })}/archive`), body, undefined, undefined, 'application/json');
  }
}

export class AiAgentsPromptOptimizationsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Create a prompt optimization for one managed agent */
  async create(agentId: string, body: CreateAgentPromptOptimizationRequest): Promise<SdkWorkResourceData & { item: AgentRuntimeExecutionRecord; }> {
    return this.client.post<SdkWorkResourceData & { item: AgentRuntimeExecutionRecord; }>(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/prompt_optimizations`), body, undefined, undefined, 'application/json');
  }
}

export class AiAgentsPreviewResponsesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Create a preview response for one managed agent */
  async create(agentId: string, body: CreateAgentPreviewResponseRequest): Promise<SdkWorkResourceData & { item: AgentRuntimeExecutionRecord; }> {
    return this.client.post<SdkWorkResourceData & { item: AgentRuntimeExecutionRecord; }>(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/preview_responses`), body, undefined, undefined, 'application/json');
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
    return this.client.get<SdkWorkPageData & { items: AgentProviderBindingRecord[]; }>(appendQueryString(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/provider_bindings`), query));
  }

/** Create a provider binding for one managed agent */
  async create(agentId: string, body: CreateAgentProviderBindingRequest): Promise<SdkWorkResourceData & { item: AgentProviderBindingRecord; }> {
    return this.client.post<SdkWorkResourceData & { item: AgentProviderBindingRecord; }>(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/provider_bindings`), body, undefined, undefined, 'application/json');
  }

/** Activate one managed agent provider binding */
  async activate(agentId: string, bindingId: string, body: ActivateAgentProviderBindingRequest): Promise<SdkWorkResourceData & { item: AgentProviderBindingRecord; }> {
    return this.client.post<SdkWorkResourceData & { item: AgentProviderBindingRecord; }>(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/provider_bindings/${serializePathParameter(bindingId, { name: 'bindingId', style: 'simple', explode: false })}/activate`), body, undefined, undefined, 'application/json');
  }
}

export interface AiAgentsListParams {
  includeDeleted?: boolean;
  scope?: 'market' | 'public' | 'published' | 'mine' | 'workspace';
  page?: number;
  pageSize?: number;
  q?: string;
}

export class AiAgentsApi {
  private client: HttpClient;
  public readonly providerBindings: AiAgentsProviderBindingsApi;
  public readonly previewResponses: AiAgentsPreviewResponsesApi;
  public readonly promptOptimizations: AiAgentsPromptOptimizationsApi;
  public readonly projects: AiAgentsProjectsApi;
  public readonly projectCompositionSlots: AiAgentsProjectCompositionSlotsApi;
  public readonly sessions: AiAgentsSessionsApi;
  public readonly sessionUserStates: AiAgentsSessionUserStatesApi;
  public readonly itemFeedback: AiAgentsItemFeedbackApi;
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
    this.projects = new AiAgentsProjectsApi(client);
    this.projectCompositionSlots = new AiAgentsProjectCompositionSlotsApi(client);
    this.sessions = new AiAgentsSessionsApi(client);
    this.sessionUserStates = new AiAgentsSessionUserStatesApi(client);
    this.itemFeedback = new AiAgentsItemFeedbackApi(client);
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
      { name: 'scope', value: params?.scope, style: 'form', explode: true, allowReserved: false },
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'q', value: params?.q, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<SdkWorkPageData & { items: AgentRecord[]; }>(appendQueryString(agentApiPath(`/ai/agents`), query));
  }

/** Create a managed agent */
  async create(body: CreateAgentRequest): Promise<SdkWorkResourceData & { item: AgentRecord; }> {
    return this.client.post<SdkWorkResourceData & { item: AgentRecord; }>(agentApiPath(`/ai/agents`), body, undefined, undefined, 'application/json');
  }

/** Retrieve one managed agent */
  async retrieve(agentId: string): Promise<SdkWorkResourceData & { item: AgentRecord; }> {
    return this.client.get<SdkWorkResourceData & { item: AgentRecord; }>(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}`));
  }

/** Update one managed agent */
  async update(agentId: string, body: UpdateAgentRequest): Promise<SdkWorkResourceData & { item: AgentRecord; }> {
    return this.client.patch<SdkWorkResourceData & { item: AgentRecord; }>(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}`), body, undefined, undefined, 'application/json');
  }

/** Soft-delete one managed agent */
  async delete(agentId: string): Promise<void> {
    return this.client.delete<void>(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}`));
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
