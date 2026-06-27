import { backendApiPath } from './paths';
import type { HttpClient } from '../http/client';

import type { ActivateAgentProviderBindingRequest, AgentAuditEventListResponse, AgentCompositionSlotListResponse, AgentCompositionSlotResponse, AgentListResponse, AgentProviderBindingListResponse, AgentProviderBindingResponse, AgentResponse, AuditAction, CreateAgentCompositionSlotRequest, CreateAgentProviderBindingRequest, CreateAgentRequest, Int64String, RestoreAgentRequest, UpdateAgentCompositionSlotRequest, UpdateAgentRequest, UpdateAgentStatusRequest } from '../types';


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
    return this.client.get<AgentCompositionSlotListResponse>(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/composition_slots`));
  }

/** Create a composition slot for one managed agent */
  async create(agentId: string, body: CreateAgentCompositionSlotRequest): Promise<AgentCompositionSlotResponse> {
    return this.client.post<AgentCompositionSlotResponse>(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/composition_slots`), body, undefined, undefined, 'application/json');
  }

/** Retrieve one managed agent composition slot */
  async retrieve(agentId: string, slotId: string): Promise<AgentCompositionSlotResponse> {
    return this.client.get<AgentCompositionSlotResponse>(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/composition_slots/${serializePathParameter(slotId, { name: 'slotId', style: 'simple', explode: false })}`));
  }

/** Update one managed agent composition slot */
  async update(agentId: string, slotId: string, body: UpdateAgentCompositionSlotRequest): Promise<AgentCompositionSlotResponse> {
    return this.client.patch<AgentCompositionSlotResponse>(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/composition_slots/${serializePathParameter(slotId, { name: 'slotId', style: 'simple', explode: false })}`), body, undefined, undefined, 'application/json');
  }

/** Delete one managed agent composition slot */
  async delete(agentId: string, slotId: string, params: AiAgentsCompositionSlotsDeleteParams): Promise<AgentCompositionSlotResponse> {
    const query = buildQueryString([
      { name: 'expected_version', value: params.expectedVersion, style: 'form', explode: true, allowReserved: false },
      { name: 'requested_at', value: params.requestedAt, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.delete<AgentCompositionSlotResponse>(appendQueryString(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/composition_slots/${serializePathParameter(slotId, { name: 'slotId', style: 'simple', explode: false })}`), query));
  }
}

export interface AiAgentsProviderBindingsListParams {
  tenantId: Int64String;
  page?: number;
  pageSize?: number;
}

export interface AiAgentsProviderBindingsCreateParams {
  tenantId: Int64String;
}

export interface AiAgentsProviderBindingsActivateParams {
  tenantId: Int64String;
}

export class AiAgentsProviderBindingsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List provider bindings for one managed agent */
  async list(agentId: string, params: AiAgentsProviderBindingsListParams): Promise<AgentProviderBindingListResponse> {
    const query = buildQueryString([
      { name: 'tenant_id', value: params.tenantId, style: 'form', explode: true, allowReserved: false },
      { name: 'page', value: params.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<AgentProviderBindingListResponse>(appendQueryString(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/provider_bindings`), query));
  }

/** Create a provider binding for one managed agent */
  async create(agentId: string, body: CreateAgentProviderBindingRequest, params: AiAgentsProviderBindingsCreateParams): Promise<AgentProviderBindingResponse> {
    const query = buildQueryString([
      { name: 'tenant_id', value: params.tenantId, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.post<AgentProviderBindingResponse>(appendQueryString(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/provider_bindings`), query), body, undefined, undefined, 'application/json');
  }

/** Activate one managed agent provider binding */
  async activate(agentId: string, bindingId: string, body: ActivateAgentProviderBindingRequest, params: AiAgentsProviderBindingsActivateParams): Promise<AgentProviderBindingResponse> {
    const query = buildQueryString([
      { name: 'tenant_id', value: params.tenantId, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.post<AgentProviderBindingResponse>(appendQueryString(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/provider_bindings/${serializePathParameter(bindingId, { name: 'bindingId', style: 'simple', explode: false })}/activate`), query), body, undefined, undefined, 'application/json');
  }
}

export interface AiAgentsAuditEventsListParams {
  tenantId: Int64String;
  page?: number;
  pageSize?: number;
  action?: AuditAction;
  from_?: string;
  to?: string;
}

export class AiAgentsAuditEventsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List managed agent audit events */
  async list(agentId: string, params: AiAgentsAuditEventsListParams): Promise<AgentAuditEventListResponse> {
    const query = buildQueryString([
      { name: 'tenant_id', value: params.tenantId, style: 'form', explode: true, allowReserved: false },
      { name: 'page', value: params.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'action', value: params.action, style: 'form', explode: true, allowReserved: false },
      { name: 'from', value: params.from_, style: 'form', explode: true, allowReserved: false },
      { name: 'to', value: params.to, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<AgentAuditEventListResponse>(appendQueryString(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/audit_events`), query));
  }
}

export interface AiAgentsStatusUpdateParams {
  tenantId: Int64String;
}

export class AiAgentsStatusApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Update managed agent status */
  async update(agentId: string, body: UpdateAgentStatusRequest, params: AiAgentsStatusUpdateParams): Promise<AgentResponse> {
    const query = buildQueryString([
      { name: 'tenant_id', value: params.tenantId, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.post<AgentResponse>(appendQueryString(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/status`), query), body, undefined, undefined, 'application/json');
  }
}

export interface AiAgentsListParams {
  tenantId: Int64String;
  organizationId?: Int64String;
  ownerUserId?: Int64String;
  includeDeleted?: boolean;
  page?: number;
  pageSize?: number;
  q?: string;
}

export interface AiAgentsCreateParams {
  tenantId: Int64String;
}

export interface AiAgentsRetrieveParams {
  tenantId: Int64String;
}

export interface AiAgentsUpdateParams {
  tenantId: Int64String;
}

export interface AiAgentsRestoreParams {
  tenantId: Int64String;
}

export class AiAgentsApi {
  private client: HttpClient;
  public readonly status: AiAgentsStatusApi;
  public readonly auditEvents: AiAgentsAuditEventsApi;
  public readonly providerBindings: AiAgentsProviderBindingsApi;
  public readonly compositionSlots: AiAgentsCompositionSlotsApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.status = new AiAgentsStatusApi(client);
    this.auditEvents = new AiAgentsAuditEventsApi(client);
    this.providerBindings = new AiAgentsProviderBindingsApi(client);
    this.compositionSlots = new AiAgentsCompositionSlotsApi(client);
  }


/** List managed agents for backend administration */
  async list(params: AiAgentsListParams): Promise<AgentListResponse> {
    const query = buildQueryString([
      { name: 'tenant_id', value: params.tenantId, style: 'form', explode: true, allowReserved: false },
      { name: 'organization_id', value: params.organizationId, style: 'form', explode: true, allowReserved: false },
      { name: 'owner_user_id', value: params.ownerUserId, style: 'form', explode: true, allowReserved: false },
      { name: 'include_deleted', value: params.includeDeleted, style: 'form', explode: true, allowReserved: false },
      { name: 'page', value: params.page, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'q', value: params.q, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<AgentListResponse>(appendQueryString(backendApiPath(`/ai/agents`), query));
  }

/** Create a managed agent */
  async create(body: CreateAgentRequest, params: AiAgentsCreateParams): Promise<AgentResponse> {
    const query = buildQueryString([
      { name: 'tenant_id', value: params.tenantId, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.post<AgentResponse>(appendQueryString(backendApiPath(`/ai/agents`), query), body, undefined, undefined, 'application/json');
  }

/** Retrieve one managed agent */
  async retrieve(agentId: string, params: AiAgentsRetrieveParams): Promise<AgentResponse> {
    const query = buildQueryString([
      { name: 'tenant_id', value: params.tenantId, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<AgentResponse>(appendQueryString(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}`), query));
  }

/** Update one managed agent */
  async update(agentId: string, body: UpdateAgentRequest, params: AiAgentsUpdateParams): Promise<AgentResponse> {
    const query = buildQueryString([
      { name: 'tenant_id', value: params.tenantId, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.patch<AgentResponse>(appendQueryString(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}`), query), body, undefined, undefined, 'application/json');
  }

/** Restore one soft-deleted managed agent */
  async restore(agentId: string, body: RestoreAgentRequest, params: AiAgentsRestoreParams): Promise<AgentResponse> {
    const query = buildQueryString([
      { name: 'tenant_id', value: params.tenantId, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.post<AgentResponse>(appendQueryString(backendApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/restore`), query), body, undefined, undefined, 'application/json');
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
