import { agentApiPath } from './paths';
import type { HttpClient } from '../http/client';

import type { ActivateAgentProviderBindingRequest, AgentCompositionSlotListResponse, AgentCompositionSlotResponse, AgentListResponse, AgentMessageListResponse, AgentMessageResponse, AgentProviderBindingListResponse, AgentProviderBindingResponse, AgentResponse, AgentRuntimeExecutionResponse, AgentSessionListResponse, AgentSessionResponse, AppCloseAgentSessionRequest, AppCreateAgentSessionRequest, AppSendAgentChatMessageRequest, CreateAgentCompositionSlotRequest, CreateAgentPreviewResponseRequest, CreateAgentPromptOptimizationRequest, CreateAgentProviderBindingRequest, CreateAgentRequest, Int64String, UpdateAgentCompositionSlotRequest, UpdateAgentRequest } from '../types';




export interface AiAgentsCompositionSlotsDeleteParams {
  expectedVersion?: Int64String;
  requestedAt: string;
}

export class AiAgentsCompositionSlotsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List composition slots for one managed agent */
  async list(agentId: string): Promise<AgentCompositionSlotListResponse> {
    return this.client.get<AgentCompositionSlotListResponse>(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/composition_slots`));
  }

/** Create a composition slot for one managed agent */
  async create(agentId: string, body: CreateAgentCompositionSlotRequest): Promise<AgentCompositionSlotResponse> {
    return this.client.post<AgentCompositionSlotResponse>(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/composition_slots`), body, undefined, undefined, 'application/json');
  }

/** Retrieve one managed agent composition slot */
  async retrieve(agentId: string, slotId: string): Promise<AgentCompositionSlotResponse> {
    return this.client.get<AgentCompositionSlotResponse>(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/composition_slots/${serializePathParameter(slotId, { name: 'slotId', style: 'simple', explode: false })}`));
  }

/** Update one managed agent composition slot */
  async update(agentId: string, slotId: string, body: UpdateAgentCompositionSlotRequest): Promise<AgentCompositionSlotResponse> {
    return this.client.patch<AgentCompositionSlotResponse>(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/composition_slots/${serializePathParameter(slotId, { name: 'slotId', style: 'simple', explode: false })}`), body, undefined, undefined, 'application/json');
  }

/** Delete one managed agent composition slot */
  async delete(agentId: string, slotId: string, params: AiAgentsCompositionSlotsDeleteParams): Promise<AgentCompositionSlotResponse> {
    const query = buildQueryString([
      { name: 'expected_version', value: params.expectedVersion, style: 'form', explode: true, allowReserved: false },
      { name: 'requested_at', value: params.requestedAt, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.delete<AgentCompositionSlotResponse>(appendQueryString(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/composition_slots/${serializePathParameter(slotId, { name: 'slotId', style: 'simple', explode: false })}`), query));
  }
}

export interface AiAgentsMessagesListParams {
  page?: number;
  pageSize?: number;
}

export interface AiAgentsMessagesCreateParams {
  stream?: boolean;
}

export class AiAgentsMessagesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List messages in one chat session */
  async list(agentId: string, sessionId: string, params?: AiAgentsMessagesListParams): Promise<AgentMessageListResponse> {
    const query = buildQueryString([
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<AgentMessageListResponse>(appendQueryString(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/messages`), query));
  }

/** Send a user chat message and receive an assistant reply */
  async create(agentId: string, sessionId: string, body: AppSendAgentChatMessageRequest, params?: AiAgentsMessagesCreateParams): Promise<AsyncIterable<string>> {
    const query = buildQueryString([
      { name: 'stream', value: params?.stream, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.streamJson<string>(appendQueryString(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/messages`), query), { method: 'POST' as any, body, contentType: 'application/json' });
  }

/** Retrieve one chat message */
  async retrieve(agentId: string, sessionId: string, messageId: string): Promise<AgentMessageResponse> {
    return this.client.get<AgentMessageResponse>(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/messages/${serializePathParameter(messageId, { name: 'messageId', style: 'simple', explode: false })}`));
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


/** List chat sessions for one managed agent */
  async list(agentId: string, params?: AiAgentsSessionsListParams): Promise<AgentSessionListResponse> {
    const query = buildQueryString([
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<AgentSessionListResponse>(appendQueryString(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions`), query));
  }

/** Create a chat session for one managed agent */
  async create(agentId: string, body: AppCreateAgentSessionRequest): Promise<AgentSessionResponse> {
    return this.client.post<AgentSessionResponse>(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions`), body, undefined, undefined, 'application/json');
  }

/** Retrieve one chat session */
  async retrieve(agentId: string, sessionId: string): Promise<AgentSessionResponse> {
    return this.client.get<AgentSessionResponse>(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}`));
  }

/** Close one chat session */
  async close(agentId: string, sessionId: string, body: AppCloseAgentSessionRequest): Promise<AgentSessionResponse> {
    return this.client.post<AgentSessionResponse>(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/close`), body, undefined, undefined, 'application/json');
  }
}

export class AiAgentsPromptOptimizationsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Create a prompt optimization for one managed agent */
  async create(agentId: string, body: CreateAgentPromptOptimizationRequest): Promise<AgentRuntimeExecutionResponse> {
    return this.client.post<AgentRuntimeExecutionResponse>(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/prompt_optimizations`), body, undefined, undefined, 'application/json');
  }
}

export class AiAgentsPreviewResponsesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Create a preview response for one managed agent */
  async create(agentId: string, body: CreateAgentPreviewResponseRequest): Promise<AgentRuntimeExecutionResponse> {
    return this.client.post<AgentRuntimeExecutionResponse>(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/preview_responses`), body, undefined, undefined, 'application/json');
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
  async list(agentId: string, params?: AiAgentsProviderBindingsListParams): Promise<AgentProviderBindingListResponse> {
    const query = buildQueryString([
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<AgentProviderBindingListResponse>(appendQueryString(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/provider_bindings`), query));
  }

/** Create a provider binding for one managed agent */
  async create(agentId: string, body: CreateAgentProviderBindingRequest): Promise<AgentProviderBindingResponse> {
    return this.client.post<AgentProviderBindingResponse>(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/provider_bindings`), body, undefined, undefined, 'application/json');
  }

/** Activate one managed agent provider binding */
  async activate(agentId: string, bindingId: string, body: ActivateAgentProviderBindingRequest): Promise<AgentProviderBindingResponse> {
    return this.client.post<AgentProviderBindingResponse>(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/provider_bindings/${serializePathParameter(bindingId, { name: 'bindingId', style: 'simple', explode: false })}/activate`), body, undefined, undefined, 'application/json');
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
  public readonly messages: AiAgentsMessagesApi;
  public readonly compositionSlots: AiAgentsCompositionSlotsApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.providerBindings = new AiAgentsProviderBindingsApi(client);
    this.previewResponses = new AiAgentsPreviewResponsesApi(client);
    this.promptOptimizations = new AiAgentsPromptOptimizationsApi(client);
    this.sessions = new AiAgentsSessionsApi(client);
    this.messages = new AiAgentsMessagesApi(client);
    this.compositionSlots = new AiAgentsCompositionSlotsApi(client);
  }


/** List managed agents */
  async list(params?: AiAgentsListParams): Promise<AgentListResponse> {
    const query = buildQueryString([
      { name: 'include_deleted', value: params?.includeDeleted, style: 'form', explode: true, allowReserved: false },
      { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'q', value: params?.q, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<AgentListResponse>(appendQueryString(agentApiPath(`/ai/agents`), query));
  }

/** Create a managed agent */
  async create(body: CreateAgentRequest): Promise<AgentResponse> {
    return this.client.post<AgentResponse>(agentApiPath(`/ai/agents`), body, undefined, undefined, 'application/json');
  }

/** Retrieve one managed agent */
  async retrieve(agentId: string): Promise<AgentResponse> {
    return this.client.get<AgentResponse>(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}`));
  }

/** Update one managed agent */
  async update(agentId: string, body: UpdateAgentRequest): Promise<AgentResponse> {
    return this.client.patch<AgentResponse>(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}`), body, undefined, undefined, 'application/json');
  }

/** Soft-delete one managed agent */
  async delete(agentId: string): Promise<AgentResponse> {
    return this.client.delete<AgentResponse>(agentApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}`));
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
