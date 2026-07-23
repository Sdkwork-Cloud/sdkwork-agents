import { appApiPath } from './paths';
import type { HttpClient } from '../http/client';

import type { ActivateAgentProviderBindingRequest, AgentChatTurnRecord, AgentCompositionSlotKind, AgentCompositionSlotRecord, AgentInteractionRecord, AgentMessageFeedbackRecord, AgentMessageRecord, AgentProjectCompositionSlotRecord, AgentProjectMutationRequest, AgentProjectRecord, AgentProjectStatus, AgentProviderBindingRecord, AgentRecord, AgentResourceUserStateRecord, AgentRuntimeExecutionRecord, AgentSessionRecord, AgentTaskRecord, AnswerAgentInteractionRequest, AppCloseAgentSessionRequest, AppCreateAgentSessionRequest, ApproveAgentInteractionRequest, AppSendAgentChatMessageRequest, AppUpdateAgentSessionRequest, CancelAgentChatTurnRequest, CancelAgentTaskRequest, CodeEngineCatalog, CreateAgentCompositionSlotRequest, CreateAgentInteractionRequest, CreateAgentPreviewResponseRequest, CreateAgentProjectCompositionSlotRequest, CreateAgentProjectRequest, CreateAgentPromptOptimizationRequest, CreateAgentProviderBindingRequest, CreateAgentRequest, CreateAgentTaskRequest, Int64String, McpServerMarketplaceRecord, RestoreAgentRequest, SdkWorkPageData, SdkWorkResourceData, UpdateAgentCompositionSlotRequest, UpdateAgentMessageFeedbackRequest, UpdateAgentProjectCompositionSlotRequest, UpdateAgentProjectRequest, UpdateAgentRequest, UpdateAgentSessionUserStateRequest } from '../types';


export interface AiAgentsMcpServersListParams {
  page?: number;
  pageSize?: number;
  q?: string;
}

export class AiAgentsMcpServersApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List MCP marketplace entries from agent composition slots */
  async list(params?: AiAgentsMcpServersListParams): Promise<SdkWorkPageData & Record<string, unknown>> {
    const query = buildQueryString([
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'q', value: params?.q, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<SdkWorkPageData & Record<string, unknown>>(appendQueryString(appApiPath(`/ai/mcp_servers`), query));
  }
}

export class AiAgentsCodeEnginesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List canonical code-engine catalog */
  async list(): Promise<SdkWorkResourceData & Record<string, unknown>> {
    return this.client.get<SdkWorkResourceData & Record<string, unknown>>(appApiPath(`/ai/code_engines`));
  }
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
  async list(agentId: string, params?: AiAgentsCompositionSlotsListParams): Promise<SdkWorkPageData & Record<string, unknown>> {
    const query = buildQueryString([
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<SdkWorkPageData & Record<string, unknown>>(appendQueryString(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/composition_slots`), query));
  }

/** Create a composition slot for one managed agent */
  async create(agentId: string, body: CreateAgentCompositionSlotRequest): Promise<SdkWorkResourceData & Record<string, unknown>> {
    return this.client.post<SdkWorkResourceData & Record<string, unknown>>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/composition_slots`), body, undefined, undefined, 'application/json');
  }

/** Retrieve one managed agent composition slot */
  async retrieve(agentId: string, slotId: string): Promise<SdkWorkResourceData & Record<string, unknown>> {
    return this.client.get<SdkWorkResourceData & Record<string, unknown>>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/composition_slots/${serializePathParameter(slotId, { name: 'slotId', style: 'simple', explode: false })}`));
  }

/** Update one managed agent composition slot */
  async update(agentId: string, slotId: string, body: UpdateAgentCompositionSlotRequest): Promise<SdkWorkResourceData & Record<string, unknown>> {
    return this.client.patch<SdkWorkResourceData & Record<string, unknown>>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/composition_slots/${serializePathParameter(slotId, { name: 'slotId', style: 'simple', explode: false })}`), body, undefined, undefined, 'application/json');
  }

/** Delete one managed agent composition slot */
  async delete(agentId: string, slotId: string): Promise<void> {
    return this.client.delete<void>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/composition_slots/${serializePathParameter(slotId, { name: 'slotId', style: 'simple', explode: false })}`));
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
  async list(agentId: string, params?: AiAgentsTasksListParams): Promise<SdkWorkPageData & Record<string, unknown>> {
    const query = buildQueryString([
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<SdkWorkPageData & Record<string, unknown>>(appendQueryString(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/tasks`), query));
  }

/** Create a scheduled task for one managed agent */
  async create(agentId: string, body: CreateAgentTaskRequest): Promise<SdkWorkResourceData & Record<string, unknown>> {
    return this.client.post<SdkWorkResourceData & Record<string, unknown>>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/tasks`), body, undefined, undefined, 'application/json');
  }

/** Retrieve one scheduled task */
  async retrieve(agentId: string, taskId: string): Promise<SdkWorkResourceData & Record<string, unknown>> {
    return this.client.get<SdkWorkResourceData & Record<string, unknown>>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/tasks/${serializePathParameter(taskId, { name: 'taskId', style: 'simple', explode: false })}`));
  }

/** Cancel one scheduled task */
  async cancel(agentId: string, taskId: string, body: CancelAgentTaskRequest): Promise<SdkWorkResourceData & Record<string, unknown>> {
    return this.client.post<SdkWorkResourceData & Record<string, unknown>>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/tasks/${serializePathParameter(taskId, { name: 'taskId', style: 'simple', explode: false })}/cancel`), body, undefined, undefined, 'application/json');
  }

/** Execute one deferred scheduled task */
  async execute(agentId: string, taskId: string, body: CancelAgentTaskRequest): Promise<SdkWorkResourceData & Record<string, unknown>> {
    return this.client.post<SdkWorkResourceData & Record<string, unknown>>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/tasks/${serializePathParameter(taskId, { name: 'taskId', style: 'simple', explode: false })}/execute`), body, undefined, undefined, 'application/json');
  }
}

export interface AiAgentsInteractionsListParams {
  page?: number;
  pageSize?: number;
  status?: string;
}

export class AiAgentsInteractionsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List live interactions for one chat session */
  async list(agentId: string, sessionId: string, params?: AiAgentsInteractionsListParams): Promise<SdkWorkPageData & Record<string, unknown>> {
    const query = buildQueryString([
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'status', value: params?.status, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<SdkWorkPageData & Record<string, unknown>>(appendQueryString(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/interactions`), query));
  }

/** Create a live interaction pause point for one chat session */
  async create(agentId: string, sessionId: string, body: CreateAgentInteractionRequest): Promise<SdkWorkResourceData & Record<string, unknown>> {
    return this.client.post<SdkWorkResourceData & Record<string, unknown>>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/interactions`), body, undefined, undefined, 'application/json');
  }

/** Retrieve one live interaction */
  async retrieve(agentId: string, sessionId: string, interactionId: string): Promise<SdkWorkResourceData & Record<string, unknown>> {
    return this.client.get<SdkWorkResourceData & Record<string, unknown>>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/interactions/${serializePathParameter(interactionId, { name: 'interactionId', style: 'simple', explode: false })}`));
  }

/** Approve or reject an approval interaction */
  async approve(agentId: string, sessionId: string, interactionId: string, body: ApproveAgentInteractionRequest): Promise<SdkWorkResourceData & Record<string, unknown>> {
    return this.client.post<SdkWorkResourceData & Record<string, unknown>>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/interactions/${serializePathParameter(interactionId, { name: 'interactionId', style: 'simple', explode: false })}/approve`), body, undefined, undefined, 'application/json');
  }

/** Answer or reject a user-question interaction */
  async answer(agentId: string, sessionId: string, interactionId: string, body: AnswerAgentInteractionRequest): Promise<SdkWorkResourceData & Record<string, unknown>> {
    return this.client.post<SdkWorkResourceData & Record<string, unknown>>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/interactions/${serializePathParameter(interactionId, { name: 'interactionId', style: 'simple', explode: false })}/answer`), body, undefined, undefined, 'application/json');
  }
}

export class AiAgentsChatTurnsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Retrieve one durable chat turn */
  async retrieve(agentId: string, sessionId: string, turnId: string): Promise<SdkWorkResourceData & Record<string, unknown>> {
    return this.client.get<SdkWorkResourceData & Record<string, unknown>>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/turns/${serializePathParameter(turnId, { name: 'turnId', style: 'simple', explode: false })}`));
  }

/** Cancel one requested or running chat turn */
  async cancel(agentId: string, sessionId: string, turnId: string, body: CancelAgentChatTurnRequest): Promise<SdkWorkResourceData & Record<string, unknown>> {
    return this.client.post<SdkWorkResourceData & Record<string, unknown>>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/turns/${serializePathParameter(turnId, { name: 'turnId', style: 'simple', explode: false })}/cancel`), body, undefined, undefined, 'application/json');
  }
}

export interface AiAgentsMessagesListParams {
  page?: number;
  pageSize?: number;
}

export interface AiAgentsMessagesStreamParams {
  stream?: boolean;
}

export class AiAgentsMessagesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List messages in one chat session */
  async list(agentId: string, sessionId: string, params?: AiAgentsMessagesListParams): Promise<SdkWorkPageData & Record<string, unknown>> {
    const query = buildQueryString([
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<SdkWorkPageData & Record<string, unknown>>(appendQueryString(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/messages`), query));
  }

/** Send a user chat message and receive an assistant reply */
  async stream(agentId: string, sessionId: string, body: AppSendAgentChatMessageRequest, params?: AiAgentsMessagesStreamParams): Promise<AsyncIterable<string>> {
    const query = buildQueryString([
      { name: 'stream', value: params?.stream, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.streamJson<string>(appendQueryString(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/messages`), query), { method: 'POST' as any, body, contentType: 'application/json' });
  }

/** Retrieve one chat message */
  async retrieve(agentId: string, sessionId: string, messageId: string): Promise<SdkWorkResourceData & Record<string, unknown>> {
    return this.client.get<SdkWorkResourceData & Record<string, unknown>>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/messages/${serializePathParameter(messageId, { name: 'messageId', style: 'simple', explode: false })}`));
  }

/** Send a chat message and return one complete JSON response */
  async complete(agentId: string, sessionId: string, body: AppSendAgentChatMessageRequest): Promise<SdkWorkResourceData & Record<string, unknown>> {
    return this.client.post<SdkWorkResourceData & Record<string, unknown>>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/messages/complete`), body, undefined, undefined, 'application/json');
  }
}

export interface AiAgentsMessageFeedbackListParams {
  page?: number;
  pageSize?: number;
}

export class AiAgentsMessageFeedbackApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List the authenticated user's feedback for assistant messages in a session */
  async list(agentId: string, sessionId: string, params?: AiAgentsMessageFeedbackListParams): Promise<SdkWorkPageData & Record<string, unknown>> {
    const query = buildQueryString([
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<SdkWorkPageData & Record<string, unknown>>(appendQueryString(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/message_feedback`), query));
  }

/** Submit, change, or clear assistant message feedback */
  async update(agentId: string, sessionId: string, messageId: string, body: UpdateAgentMessageFeedbackRequest): Promise<SdkWorkResourceData & Record<string, unknown>> {
    return this.client.patch<SdkWorkResourceData & Record<string, unknown>>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/messages/${serializePathParameter(messageId, { name: 'messageId', style: 'simple', explode: false })}/feedback`), body, undefined, undefined, 'application/json');
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


/** List per-user state for chat sessions owned by the authenticated user */
  async list(agentId: string, params?: AiAgentsSessionUserStatesListParams): Promise<SdkWorkPageData & Record<string, unknown>> {
    const query = buildQueryString([
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'pinnedOnly', value: params?.pinnedOnly, style: 'form', explode: true, allowReserved: false },
      { name: 'includeHidden', value: params?.includeHidden, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<SdkWorkPageData & Record<string, unknown>>(appendQueryString(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/user_states`), query));
  }

/** Retrieve the authenticated user's state for one chat session */
  async retrieve(agentId: string, sessionId: string): Promise<SdkWorkResourceData & Record<string, unknown>> {
    return this.client.get<SdkWorkResourceData & Record<string, unknown>>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/user_state`));
  }

/** Update the authenticated user's state for one chat session */
  async update(agentId: string, sessionId: string, body: UpdateAgentSessionUserStateRequest): Promise<SdkWorkResourceData & Record<string, unknown>> {
    return this.client.patch<SdkWorkResourceData & Record<string, unknown>>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/user_state`), body, undefined, undefined, 'application/json');
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


/** List chat sessions for one managed agent */
  async list(agentId: string, params?: AiAgentsSessionsListParams): Promise<SdkWorkPageData & Record<string, unknown>> {
    const query = buildQueryString([
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'projectId', value: params?.projectId, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<SdkWorkPageData & Record<string, unknown>>(appendQueryString(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions`), query));
  }

/** Create a chat session for one managed agent */
  async create(agentId: string, body: AppCreateAgentSessionRequest): Promise<SdkWorkResourceData & Record<string, unknown>> {
    return this.client.post<SdkWorkResourceData & Record<string, unknown>>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions`), body, undefined, undefined, 'application/json');
  }

/** Retrieve one chat session */
  async retrieve(agentId: string, sessionId: string): Promise<SdkWorkResourceData & Record<string, unknown>> {
    return this.client.get<SdkWorkResourceData & Record<string, unknown>>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}`));
  }

/** Rename or move one chat session */
  async update(agentId: string, sessionId: string, body: AppUpdateAgentSessionRequest): Promise<SdkWorkResourceData & Record<string, unknown>> {
    return this.client.patch<SdkWorkResourceData & Record<string, unknown>>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}`), body, undefined, undefined, 'application/json');
  }

/** Soft delete one chat session */
  async delete(agentId: string, sessionId: string): Promise<void> {
    return this.client.delete<void>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}`));
  }

/** Close one chat session */
  async close(agentId: string, sessionId: string, body: AppCloseAgentSessionRequest): Promise<SdkWorkResourceData & Record<string, unknown>> {
    return this.client.post<SdkWorkResourceData & Record<string, unknown>>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/close`), body, undefined, undefined, 'application/json');
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


/** List composition slots for a commercial chat project */
  async list(projectId: string, params?: AiAgentsProjectCompositionSlotsListParams): Promise<SdkWorkPageData & Record<string, unknown>> {
    const query = buildQueryString([
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'slotKind', value: params?.slotKind, style: 'form', explode: true, allowReserved: false },
      { name: 'enabled', value: params?.enabled, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<SdkWorkPageData & Record<string, unknown>>(appendQueryString(appApiPath(`/ai/projects/${serializePathParameter(projectId, { name: 'projectId', style: 'simple', explode: false })}/composition_slots`), query));
  }

/** Add a composition slot to a commercial chat project */
  async create(projectId: string, body: CreateAgentProjectCompositionSlotRequest): Promise<SdkWorkResourceData & Record<string, unknown>> {
    return this.client.post<SdkWorkResourceData & Record<string, unknown>>(appApiPath(`/ai/projects/${serializePathParameter(projectId, { name: 'projectId', style: 'simple', explode: false })}/composition_slots`), body, undefined, undefined, 'application/json');
  }

/** Retrieve a project composition slot */
  async retrieve(projectId: string, slotId: string): Promise<SdkWorkResourceData & Record<string, unknown>> {
    return this.client.get<SdkWorkResourceData & Record<string, unknown>>(appApiPath(`/ai/projects/${serializePathParameter(projectId, { name: 'projectId', style: 'simple', explode: false })}/composition_slots/${serializePathParameter(slotId, { name: 'slotId', style: 'simple', explode: false })}`));
  }

/** Update a project composition slot */
  async update(projectId: string, slotId: string, body: UpdateAgentProjectCompositionSlotRequest): Promise<SdkWorkResourceData & Record<string, unknown>> {
    return this.client.patch<SdkWorkResourceData & Record<string, unknown>>(appApiPath(`/ai/projects/${serializePathParameter(projectId, { name: 'projectId', style: 'simple', explode: false })}/composition_slots/${serializePathParameter(slotId, { name: 'slotId', style: 'simple', explode: false })}`), body, undefined, undefined, 'application/json');
  }

/** Soft-delete a project composition slot */
  async delete(projectId: string, slotId: string, params: AiAgentsProjectCompositionSlotsDeleteParams): Promise<void> {
    const query = buildQueryString([
      { name: 'expected_version', value: params.expectedVersion, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.delete<void>(appendQueryString(appApiPath(`/ai/projects/${serializePathParameter(projectId, { name: 'projectId', style: 'simple', explode: false })}/composition_slots/${serializePathParameter(slotId, { name: 'slotId', style: 'simple', explode: false })}`), query));
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


/** List commercial chat projects for the current user */
  async list(params?: AiAgentsProjectsListParams): Promise<SdkWorkPageData & Record<string, unknown>> {
    const query = buildQueryString([
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'q', value: params?.q, style: 'form', explode: true, allowReserved: false },
      { name: 'status', value: params?.status, style: 'form', explode: true, allowReserved: false },
      { name: 'includeDeleted', value: params?.includeDeleted, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<SdkWorkPageData & Record<string, unknown>>(appendQueryString(appApiPath(`/ai/projects`), query));
  }

/** Create a commercial chat project */
  async create(body: CreateAgentProjectRequest): Promise<SdkWorkResourceData & Record<string, unknown>> {
    return this.client.post<SdkWorkResourceData & Record<string, unknown>>(appApiPath(`/ai/projects`), body, undefined, undefined, 'application/json');
  }

/** Retrieve a commercial chat project */
  async retrieve(projectId: string): Promise<SdkWorkResourceData & Record<string, unknown>> {
    return this.client.get<SdkWorkResourceData & Record<string, unknown>>(appApiPath(`/ai/projects/${serializePathParameter(projectId, { name: 'projectId', style: 'simple', explode: false })}`));
  }

/** Update a commercial chat project */
  async update(projectId: string, body: UpdateAgentProjectRequest): Promise<SdkWorkResourceData & Record<string, unknown>> {
    return this.client.patch<SdkWorkResourceData & Record<string, unknown>>(appApiPath(`/ai/projects/${serializePathParameter(projectId, { name: 'projectId', style: 'simple', explode: false })}`), body, undefined, undefined, 'application/json');
  }

/** Soft-delete a commercial chat project */
  async delete(projectId: string): Promise<void> {
    return this.client.delete<void>(appApiPath(`/ai/projects/${serializePathParameter(projectId, { name: 'projectId', style: 'simple', explode: false })}`));
  }

/** Archive a commercial chat project */
  async archive(projectId: string, body: AgentProjectMutationRequest): Promise<SdkWorkResourceData & Record<string, unknown>> {
    return this.client.post<SdkWorkResourceData & Record<string, unknown>>(appApiPath(`/ai/projects/${serializePathParameter(projectId, { name: 'projectId', style: 'simple', explode: false })}/archive`), body, undefined, undefined, 'application/json');
  }
}

export class AiAgentsPromptOptimizationsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Create a prompt optimization for one managed agent */
  async create(agentId: string, body: CreateAgentPromptOptimizationRequest): Promise<SdkWorkResourceData & Record<string, unknown>> {
    return this.client.post<SdkWorkResourceData & Record<string, unknown>>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/prompt_optimizations`), body, undefined, undefined, 'application/json');
  }
}

export class AiAgentsPreviewResponsesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Create a preview response for one managed agent */
  async create(agentId: string, body: CreateAgentPreviewResponseRequest): Promise<SdkWorkResourceData & Record<string, unknown>> {
    return this.client.post<SdkWorkResourceData & Record<string, unknown>>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/preview_responses`), body, undefined, undefined, 'application/json');
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
  async list(agentId: string, params?: AiAgentsProviderBindingsListParams): Promise<SdkWorkPageData & Record<string, unknown>> {
    const query = buildQueryString([
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<SdkWorkPageData & Record<string, unknown>>(appendQueryString(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/provider_bindings`), query));
  }

/** Create a provider binding for one managed agent */
  async create(agentId: string, body: CreateAgentProviderBindingRequest): Promise<SdkWorkResourceData & Record<string, unknown>> {
    return this.client.post<SdkWorkResourceData & Record<string, unknown>>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/provider_bindings`), body, undefined, undefined, 'application/json');
  }

/** Activate one managed agent provider binding */
  async activate(agentId: string, bindingId: string, body: ActivateAgentProviderBindingRequest): Promise<SdkWorkResourceData & Record<string, unknown>> {
    return this.client.post<SdkWorkResourceData & Record<string, unknown>>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/provider_bindings/${serializePathParameter(bindingId, { name: 'bindingId', style: 'simple', explode: false })}/activate`), body, undefined, undefined, 'application/json');
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
  public readonly messageFeedback: AiAgentsMessageFeedbackApi;
  public readonly messages: AiAgentsMessagesApi;
  public readonly chatTurns: AiAgentsChatTurnsApi;
  public readonly interactions: AiAgentsInteractionsApi;
  public readonly tasks: AiAgentsTasksApi;
  public readonly compositionSlots: AiAgentsCompositionSlotsApi;
  public readonly codeEngines: AiAgentsCodeEnginesApi;
  public readonly mcpServers: AiAgentsMcpServersApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.providerBindings = new AiAgentsProviderBindingsApi(client);
    this.previewResponses = new AiAgentsPreviewResponsesApi(client);
    this.promptOptimizations = new AiAgentsPromptOptimizationsApi(client);
    this.projects = new AiAgentsProjectsApi(client);
    this.projectCompositionSlots = new AiAgentsProjectCompositionSlotsApi(client);
    this.sessions = new AiAgentsSessionsApi(client);
    this.sessionUserStates = new AiAgentsSessionUserStatesApi(client);
    this.messageFeedback = new AiAgentsMessageFeedbackApi(client);
    this.messages = new AiAgentsMessagesApi(client);
    this.chatTurns = new AiAgentsChatTurnsApi(client);
    this.interactions = new AiAgentsInteractionsApi(client);
    this.tasks = new AiAgentsTasksApi(client);
    this.compositionSlots = new AiAgentsCompositionSlotsApi(client);
    this.codeEngines = new AiAgentsCodeEnginesApi(client);
    this.mcpServers = new AiAgentsMcpServersApi(client);
  }


/** List managed agents */
  async list(params?: AiAgentsListParams): Promise<SdkWorkPageData & Record<string, unknown>> {
    const query = buildQueryString([
      { name: 'include_deleted', value: params?.includeDeleted, style: 'form', explode: true, allowReserved: false },
      { name: 'scope', value: params?.scope, style: 'form', explode: true, allowReserved: false },
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'q', value: params?.q, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<SdkWorkPageData & Record<string, unknown>>(appendQueryString(appApiPath(`/ai/agents`), query));
  }

/** Create a managed agent */
  async create(body: CreateAgentRequest): Promise<SdkWorkResourceData & Record<string, unknown>> {
    return this.client.post<SdkWorkResourceData & Record<string, unknown>>(appApiPath(`/ai/agents`), body, undefined, undefined, 'application/json');
  }

/** Retrieve one managed agent */
  async retrieve(agentId: string): Promise<SdkWorkResourceData & Record<string, unknown>> {
    return this.client.get<SdkWorkResourceData & Record<string, unknown>>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}`));
  }

/** Update one managed agent */
  async update(agentId: string, body: UpdateAgentRequest): Promise<SdkWorkResourceData & Record<string, unknown>> {
    return this.client.patch<SdkWorkResourceData & Record<string, unknown>>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}`), body, undefined, undefined, 'application/json');
  }

/** Soft-delete one managed agent */
  async delete(agentId: string): Promise<void> {
    return this.client.delete<void>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}`));
  }

/** Restore one soft-deleted managed agent */
  async restore(agentId: string, body: RestoreAgentRequest): Promise<SdkWorkResourceData & Record<string, unknown>> {
    return this.client.post<SdkWorkResourceData & Record<string, unknown>>(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/restore`), body, undefined, undefined, 'application/json');
  }
}

export class AiApi {
  private client: HttpClient;
  public readonly agents: AiAgentsApi;

  constructor(client: HttpClient) {
    this.client = client;
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
