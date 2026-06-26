import type { HttpClient } from '../http/client';
import type { ActivateAgentProviderBindingRequest, AgentDeploymentListResponse, AgentDeploymentResponse, AgentListResponse, AgentProviderBindingListResponse, AgentProviderBindingResponse, AgentResponse, AgentRuntimeExecutionResponse, CancelKnowledgeSyncJobRequest, CompleteKnowledgeSyncJobRequest, CreateAgentDeploymentRequest, CreateAgentPreviewResponseRequest, CreateAgentPromptOptimizationRequest, CreateAgentProviderBindingRequest, CreateAgentRequest, CreateKnowledgeBaseRequest, CreateKnowledgeBindingRequest, CreateKnowledgeChunkRequest, CreateKnowledgeDocumentRequest, CreateKnowledgeSourceRequest, CreateKnowledgeSyncJobRequest, CreateMemoryBindingRequest, CreateMemoryNamespaceRequest, CreateMemoryProfileRequest, CreateMemoryRecordRequest, CreateMemoryRelationRequest, CreateMemorySourceRequest, CreateMemoryStoreRequest, FailKnowledgeSyncJobRequest, Int64String, KnowledgeBaseListResponse, KnowledgeBaseResponse, KnowledgeBindingListResponse, KnowledgeBindingResponse, KnowledgeChunkListResponse, KnowledgeChunkResponse, KnowledgeDocumentListResponse, KnowledgeDocumentResponse, KnowledgeIndexListResponse, KnowledgeIndexResponse, KnowledgeSearchResponse, KnowledgeSourceListResponse, KnowledgeSourceResponse, KnowledgeSyncJobListResponse, KnowledgeSyncJobResponse, MemoryBindingResponse, MemoryNamespaceResponse, MemoryProfileResponse, MemoryRecordListResponse, MemoryRecordResponse, MemoryRelationListResponse, MemoryRelationResponse, MemoryRetrievalIndexListResponse, MemoryRetrievalIndexResponse, MemorySourceListResponse, MemorySourceResponse, MemoryStoreResponse, RestoreAgentRequest, SearchKnowledgeRequest, StartKnowledgeSyncJobRequest, UpdateAgentRequest, UpdateKnowledgeBaseRequest, UpdateKnowledgeDocumentRequest, UpdateKnowledgeSourceRequest, UpdateMemoryStoreRequest, UpsertKnowledgeIndexRequest, UpsertMemoryRetrievalIndexRequest } from '../types';
export interface AiMemoryRetrievalIndexesListParams {
    page?: number;
    pageSize?: number;
}
export declare class AiMemoryRetrievalIndexesApi {
    private client;
    constructor(client: HttpClient);
    /** List retrieval indexes for one agent memory record */
    list(memoryId: string, params?: AiMemoryRetrievalIndexesListParams): Promise<MemoryRetrievalIndexListResponse>;
    /** Upsert an agent memory retrieval index */
    upsert(body: UpsertMemoryRetrievalIndexRequest): Promise<MemoryRetrievalIndexResponse>;
}
export interface AiMemoryRelationsListParams {
    page?: number;
    pageSize?: number;
}
export declare class AiMemoryRelationsApi {
    private client;
    constructor(client: HttpClient);
    /** List graph relations for one agent memory record */
    list(memoryId: string, params?: AiMemoryRelationsListParams): Promise<MemoryRelationListResponse>;
    /** Create a graph relation for one agent memory record */
    create(memoryId: string, body: CreateMemoryRelationRequest): Promise<MemoryRelationResponse>;
}
export interface AiMemorySourcesListParams {
    page?: number;
    pageSize?: number;
}
export declare class AiMemorySourcesApi {
    private client;
    constructor(client: HttpClient);
    /** List provenance sources for one agent memory record */
    list(memoryId: string, params?: AiMemorySourcesListParams): Promise<MemorySourceListResponse>;
    /** Create a provenance source for one agent memory record */
    create(memoryId: string, body: CreateMemorySourceRequest): Promise<MemorySourceResponse>;
}
export interface AiMemoryRecordsListParams {
    page?: number;
    pageSize?: number;
}
export interface AiMemoryRecordsDeleteParams {
    expectedVersion?: Int64String;
    requestedAt: string;
}
export declare class AiMemoryRecordsApi {
    private client;
    constructor(client: HttpClient);
    /** List agent memory records in one namespace */
    list(memoryNamespaceId: string, params?: AiMemoryRecordsListParams): Promise<MemoryRecordListResponse>;
    /** Create an agent memory record in one namespace */
    create(memoryNamespaceId: string, body: CreateMemoryRecordRequest): Promise<MemoryRecordResponse>;
    /** Retrieve one agent memory record */
    retrieve(memoryId: string): Promise<MemoryRecordResponse>;
    /** Soft-delete one agent memory record */
    delete(memoryId: string, params: AiMemoryRecordsDeleteParams): Promise<MemoryRecordResponse>;
    /** Restore one soft-deleted agent memory record */
    restore(memoryId: string, body: RestoreAgentRequest): Promise<MemoryRecordResponse>;
}
export declare class AiMemoryNamespacesApi {
    private client;
    constructor(client: HttpClient);
    /** Create an agent memory namespace */
    create(body: CreateMemoryNamespaceRequest): Promise<MemoryNamespaceResponse>;
    /** Retrieve one agent memory namespace */
    retrieve(memoryNamespaceId: string): Promise<MemoryNamespaceResponse>;
}
export declare class AiMemoryBindingsApi {
    private client;
    constructor(client: HttpClient);
    /** Create an agent memory profile binding */
    create(memoryProfileId: string, body: CreateMemoryBindingRequest): Promise<MemoryBindingResponse>;
    /** Retrieve one agent memory binding */
    retrieve(memoryBindingId: string): Promise<MemoryBindingResponse>;
}
export declare class AiMemoryProfilesApi {
    private client;
    constructor(client: HttpClient);
    /** Create an agent memory profile for one store */
    create(memoryStoreId: string, body: CreateMemoryProfileRequest): Promise<MemoryProfileResponse>;
    /** Retrieve one agent memory profile */
    retrieve(memoryProfileId: string): Promise<MemoryProfileResponse>;
}
export declare class AiMemoryStoresApi {
    private client;
    constructor(client: HttpClient);
    /** Create an agent memory store */
    create(body: CreateMemoryStoreRequest): Promise<MemoryStoreResponse>;
    /** Retrieve one agent memory store */
    retrieve(memoryStoreId: string): Promise<MemoryStoreResponse>;
    /** Update one agent memory store */
    update(memoryStoreId: string, body: UpdateMemoryStoreRequest): Promise<MemoryStoreResponse>;
}
export interface AiKnowledgeSyncJobsListParams {
    page?: number;
    pageSize?: number;
}
export declare class AiKnowledgeSyncJobsApi {
    private client;
    constructor(client: HttpClient);
    /** List sync jobs for one agent knowledge base */
    list(knowledgeBaseId: string, params?: AiKnowledgeSyncJobsListParams): Promise<KnowledgeSyncJobListResponse>;
    /** Create a sync job for one agent knowledge base */
    create(knowledgeBaseId: string, body: CreateKnowledgeSyncJobRequest): Promise<KnowledgeSyncJobResponse>;
    /** Retrieve one agent knowledge sync job */
    retrieve(syncJobId: string): Promise<KnowledgeSyncJobResponse>;
    /** Start one agent knowledge sync job */
    start(syncJobId: string, body: StartKnowledgeSyncJobRequest): Promise<KnowledgeSyncJobResponse>;
    /** Complete one agent knowledge sync job */
    complete(syncJobId: string, body: CompleteKnowledgeSyncJobRequest): Promise<KnowledgeSyncJobResponse>;
    /** Fail one agent knowledge sync job */
    fail(syncJobId: string, body: FailKnowledgeSyncJobRequest): Promise<KnowledgeSyncJobResponse>;
    /** Cancel one agent knowledge sync job */
    cancel(syncJobId: string, body: CancelKnowledgeSyncJobRequest): Promise<KnowledgeSyncJobResponse>;
}
export interface AiKnowledgeBindingsListParams {
    page?: number;
    pageSize?: number;
}
export declare class AiKnowledgeBindingsApi {
    private client;
    constructor(client: HttpClient);
    /** List bindings for one agent knowledge base */
    list(knowledgeBaseId: string, params?: AiKnowledgeBindingsListParams): Promise<KnowledgeBindingListResponse>;
    /** Create a binding for one agent knowledge base */
    create(knowledgeBaseId: string, body: CreateKnowledgeBindingRequest): Promise<KnowledgeBindingResponse>;
    /** Retrieve one agent knowledge binding */
    retrieve(knowledgeBindingId: string): Promise<KnowledgeBindingResponse>;
}
export interface AiKnowledgeIndexesListParams {
    page?: number;
    pageSize?: number;
}
export declare class AiKnowledgeIndexesApi {
    private client;
    constructor(client: HttpClient);
    /** List indexes for one agent knowledge document */
    list(knowledgeDocumentId: string, params?: AiKnowledgeIndexesListParams): Promise<KnowledgeIndexListResponse>;
    /** Upsert an agent knowledge retrieval index */
    upsert(body: UpsertKnowledgeIndexRequest): Promise<KnowledgeIndexResponse>;
    /** Retrieve one agent knowledge retrieval index */
    retrieve(knowledgeIndexId: string): Promise<KnowledgeIndexResponse>;
}
export interface AiKnowledgeChunksListParams {
    page?: number;
    pageSize?: number;
}
export declare class AiKnowledgeChunksApi {
    private client;
    constructor(client: HttpClient);
    /** List chunks for one agent knowledge document */
    list(knowledgeDocumentId: string, params?: AiKnowledgeChunksListParams): Promise<KnowledgeChunkListResponse>;
    /** Create a chunk for one agent knowledge document */
    create(knowledgeDocumentId: string, body: CreateKnowledgeChunkRequest): Promise<KnowledgeChunkResponse>;
    /** Retrieve one agent knowledge chunk */
    retrieve(knowledgeChunkId: string): Promise<KnowledgeChunkResponse>;
}
export declare class AiKnowledgeReadApi {
    private client;
    constructor(client: HttpClient);
    /** Read one provider-neutral knowledge document */
    read(knowledgeDocumentId: string): Promise<KnowledgeDocumentResponse>;
}
export declare class AiKnowledgeSearchApi {
    private client;
    constructor(client: HttpClient);
    /** Search an agent knowledge base for provider-neutral RAG candidates */
    search(knowledgeBaseId: string, body: SearchKnowledgeRequest): Promise<KnowledgeSearchResponse>;
}
export interface AiKnowledgeDocumentsDeleteParams {
    expectedVersion?: Int64String;
    requestedAt: string;
}
export declare class AiKnowledgeDocumentsApi {
    private client;
    constructor(client: HttpClient);
    /** Create a document for one agent knowledge base */
    create(knowledgeBaseId: string, body: CreateKnowledgeDocumentRequest): Promise<KnowledgeDocumentResponse>;
    /** Update one agent knowledge document */
    update(knowledgeDocumentId: string, body: UpdateKnowledgeDocumentRequest): Promise<KnowledgeDocumentResponse>;
    /** Soft-delete one agent knowledge document */
    delete(knowledgeDocumentId: string, params: AiKnowledgeDocumentsDeleteParams): Promise<KnowledgeDocumentResponse>;
    /** Restore one soft-deleted agent knowledge document */
    restore(knowledgeDocumentId: string, body: RestoreAgentRequest): Promise<KnowledgeDocumentResponse>;
}
export interface AiKnowledgeListListParams {
    page?: number;
    pageSize?: number;
}
export declare class AiKnowledgeListApi {
    private client;
    constructor(client: HttpClient);
    /** List provider-neutral knowledge documents for one agent knowledge base */
    list(knowledgeBaseId: string, params?: AiKnowledgeListListParams): Promise<KnowledgeDocumentListResponse>;
}
export interface AiKnowledgeSourcesListParams {
    page?: number;
    pageSize?: number;
}
export interface AiKnowledgeSourcesDeleteParams {
    expectedVersion?: Int64String;
    requestedAt: string;
}
export declare class AiKnowledgeSourcesApi {
    private client;
    constructor(client: HttpClient);
    /** List sources for one agent knowledge base */
    list(knowledgeBaseId: string, params?: AiKnowledgeSourcesListParams): Promise<KnowledgeSourceListResponse>;
    /** Create a source for one agent knowledge base */
    create(knowledgeBaseId: string, body: CreateKnowledgeSourceRequest): Promise<KnowledgeSourceResponse>;
    /** Retrieve one agent knowledge source */
    retrieve(knowledgeSourceId: string): Promise<KnowledgeSourceResponse>;
    /** Update one agent knowledge source */
    update(knowledgeSourceId: string, body: UpdateKnowledgeSourceRequest): Promise<KnowledgeSourceResponse>;
    /** Soft-delete one agent knowledge source */
    delete(knowledgeSourceId: string, params: AiKnowledgeSourcesDeleteParams): Promise<KnowledgeSourceResponse>;
    /** Restore one soft-deleted agent knowledge source */
    restore(knowledgeSourceId: string, body: RestoreAgentRequest): Promise<KnowledgeSourceResponse>;
}
export interface AiKnowledgeBasesListParams {
    includeDeleted?: boolean;
    page?: number;
    pageSize?: number;
    q?: string;
}
export interface AiKnowledgeBasesDeleteParams {
    expectedVersion?: Int64String;
    requestedAt: string;
}
export declare class AiKnowledgeBasesApi {
    private client;
    constructor(client: HttpClient);
    /** List agent knowledge bases */
    list(params?: AiKnowledgeBasesListParams): Promise<KnowledgeBaseListResponse>;
    /** Create an agent knowledge base */
    create(body: CreateKnowledgeBaseRequest): Promise<KnowledgeBaseResponse>;
    /** Retrieve one agent knowledge base */
    retrieve(knowledgeBaseId: string): Promise<KnowledgeBaseResponse>;
    /** Update one agent knowledge base */
    update(knowledgeBaseId: string, body: UpdateKnowledgeBaseRequest): Promise<KnowledgeBaseResponse>;
    /** Soft-delete one agent knowledge base */
    delete(knowledgeBaseId: string, params: AiKnowledgeBasesDeleteParams): Promise<KnowledgeBaseResponse>;
    /** Restore one soft-deleted agent knowledge base */
    restore(knowledgeBaseId: string, body: RestoreAgentRequest): Promise<KnowledgeBaseResponse>;
}
export declare class AiAgentsPromptOptimizationsApi {
    private client;
    constructor(client: HttpClient);
    /** Create a prompt optimization for one managed agent */
    create(agentId: string, body: CreateAgentPromptOptimizationRequest): Promise<AgentRuntimeExecutionResponse>;
}
export declare class AiAgentsPreviewResponsesApi {
    private client;
    constructor(client: HttpClient);
    /** Create a preview response for one managed agent */
    create(agentId: string, body: CreateAgentPreviewResponseRequest): Promise<AgentRuntimeExecutionResponse>;
}
export interface AiAgentsDeploymentsListParams {
    page?: number;
    pageSize?: number;
}
export declare class AiAgentsDeploymentsApi {
    private client;
    constructor(client: HttpClient);
    /** List deployments for one managed agent */
    list(agentId: string, params?: AiAgentsDeploymentsListParams): Promise<AgentDeploymentListResponse>;
    /** Create a deployment snapshot for one managed agent provider binding */
    create(agentId: string, body: CreateAgentDeploymentRequest): Promise<AgentDeploymentResponse>;
}
export interface AiAgentsProviderBindingsListParams {
    page?: number;
    pageSize?: number;
}
export declare class AiAgentsProviderBindingsApi {
    private client;
    constructor(client: HttpClient);
    /** List provider bindings for one managed agent */
    list(agentId: string, params?: AiAgentsProviderBindingsListParams): Promise<AgentProviderBindingListResponse>;
    /** Create a provider binding for one managed agent */
    create(agentId: string, body: CreateAgentProviderBindingRequest): Promise<AgentProviderBindingResponse>;
    /** Activate one managed agent provider binding */
    activate(agentId: string, bindingId: string, body: ActivateAgentProviderBindingRequest): Promise<AgentProviderBindingResponse>;
}
export interface AiAgentsListParams {
    includeDeleted?: boolean;
    page?: number;
    pageSize?: number;
    q?: string;
}
export declare class AiAgentsApi {
    private client;
    readonly providerBindings: AiAgentsProviderBindingsApi;
    readonly deployments: AiAgentsDeploymentsApi;
    readonly previewResponses: AiAgentsPreviewResponsesApi;
    readonly promptOptimizations: AiAgentsPromptOptimizationsApi;
    constructor(client: HttpClient);
    /** List managed agents */
    list(params?: AiAgentsListParams): Promise<AgentListResponse>;
    /** Create a managed agent */
    create(body: CreateAgentRequest): Promise<AgentResponse>;
    /** Retrieve one managed agent */
    retrieve(agentId: string): Promise<AgentResponse>;
    /** Update one managed agent */
    update(agentId: string, body: UpdateAgentRequest): Promise<AgentResponse>;
    /** Soft-delete one managed agent */
    delete(agentId: string): Promise<AgentResponse>;
    /** Restore one soft-deleted managed agent */
    restore(agentId: string, body: RestoreAgentRequest): Promise<AgentResponse>;
}
export declare class AiApi {
    private client;
    readonly agents: AiAgentsApi;
    readonly knowledgeBases: AiKnowledgeBasesApi;
    readonly knowledgeSources: AiKnowledgeSourcesApi;
    readonly knowledgeList: AiKnowledgeListApi;
    readonly knowledgeDocuments: AiKnowledgeDocumentsApi;
    readonly knowledgeSearch: AiKnowledgeSearchApi;
    readonly knowledgeRead: AiKnowledgeReadApi;
    readonly knowledgeChunks: AiKnowledgeChunksApi;
    readonly knowledgeIndexes: AiKnowledgeIndexesApi;
    readonly knowledgeBindings: AiKnowledgeBindingsApi;
    readonly knowledgeSyncJobs: AiKnowledgeSyncJobsApi;
    readonly memoryStores: AiMemoryStoresApi;
    readonly memoryProfiles: AiMemoryProfilesApi;
    readonly memoryBindings: AiMemoryBindingsApi;
    readonly memoryNamespaces: AiMemoryNamespacesApi;
    readonly memoryRecords: AiMemoryRecordsApi;
    readonly memorySources: AiMemorySourcesApi;
    readonly memoryRelations: AiMemoryRelationsApi;
    readonly memoryRetrievalIndexes: AiMemoryRetrievalIndexesApi;
    constructor(client: HttpClient);
}
export declare function createAiApi(client: HttpClient): AiApi;
//# sourceMappingURL=ai.d.ts.map