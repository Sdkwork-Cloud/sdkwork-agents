import { BaseHttpClient, withRetry } from '@sdkwork/sdk-common';
export { DEFAULT_TIMEOUT, DefaultAuthTokenManager, SUCCESS_CODES, createTokenManager } from '@sdkwork/sdk-common';

class HttpClient extends BaseHttpClient {
    constructor(config) {
        super(config);
    }
    getInternalAuthConfig() {
        const self = this;
        self.authConfig = self.authConfig || {};
        return self.authConfig;
    }
    getInternalHeaders() {
        const self = this;
        self.config = self.config || {};
        self.config.headers = self.config.headers || {};
        return self.config.headers;
    }
    buildRequestHeaders(headers, contentType) {
        const mergedHeaders = {
            ...(headers ?? {}),
        };
        if (contentType && contentType.toLowerCase() !== 'multipart/form-data') {
            mergedHeaders['Content-Type'] = contentType;
        }
        return Object.keys(mergedHeaders).length > 0 ? mergedHeaders : undefined;
    }
    buildHeaders(config, skipAuth = false) {
        const headers = super.buildHeaders(config, skipAuth);
        if (!skipAuth && !config?.skipAuth) {
            return headers;
        }
        [
            HttpClient.ACCESS_TOKEN_HEADER,
            'Authorization',
            'Access-Token',
            ['X', 'API', 'Key'].join('-'),
            'X-Tenant-Id',
            'X-Organization-Id',
            'X-Platform',
            'X-User-Id',
            'X-Sdkwork-Tenant-Id',
            'X-Sdkwork-Organization-Id',
            'X-Sdkwork-User-Id',
        ].forEach((key) => {
            delete headers[key];
        });
        return headers;
    }
    buildRequestBody(body, contentType) {
        if (body == null) {
            return body;
        }
        const normalizedContentType = (contentType ?? '').toLowerCase();
        if (normalizedContentType === 'application/x-www-form-urlencoded') {
            return this.encodeFormBody(body);
        }
        if (normalizedContentType === 'multipart/form-data') {
            return this.encodeMultipartBody(body);
        }
        return body;
    }
    encodeMultipartBody(body) {
        if (body instanceof FormData) {
            return body;
        }
        const formData = new FormData();
        if (body instanceof Map) {
            for (const [key, value] of body.entries()) {
                this.appendMultipartValue(formData, String(key), value);
            }
            return formData;
        }
        if (typeof body === 'object') {
            const record = body;
            for (const [key, value] of Object.entries(record)) {
                if (this.isMultipartMetadataField(key)) {
                    continue;
                }
                this.appendMultipartValue(formData, key, value, this.resolveMultipartFileName(record, key));
            }
            return formData;
        }
        this.appendMultipartValue(formData, 'value', body);
        return formData;
    }
    appendMultipartValue(formData, key, value, fileName) {
        if (value == null) {
            return;
        }
        if (Array.isArray(value)) {
            value.forEach((item) => this.appendMultipartValue(formData, key, item, fileName));
            return;
        }
        if (value instanceof Blob) {
            if (fileName) {
                formData.append(key, value, fileName);
                return;
            }
            formData.append(key, value);
            return;
        }
        if (value instanceof Date) {
            formData.append(key, value.toISOString());
            return;
        }
        if (typeof value === 'object') {
            formData.append(key, JSON.stringify(value));
            return;
        }
        formData.append(key, String(value));
    }
    resolveMultipartFileName(record, key) {
        const fieldSpecificName = record[`${key}FileName`];
        if (typeof fieldSpecificName === 'string' && fieldSpecificName.trim()) {
            return fieldSpecificName.trim();
        }
        const genericName = record.fileName;
        if (key === 'file' && typeof genericName === 'string' && genericName.trim()) {
            return genericName.trim();
        }
        return undefined;
    }
    isMultipartMetadataField(key) {
        return key === 'fileName' || key.endsWith('FileName');
    }
    encodeFormBody(body) {
        if (body instanceof URLSearchParams) {
            return body.toString();
        }
        if (typeof body === 'string') {
            return body;
        }
        const params = new URLSearchParams();
        if (body instanceof Map) {
            for (const [key, value] of body.entries()) {
                this.appendFormValue(params, String(key), value);
            }
            return params.toString();
        }
        if (typeof body === 'object') {
            for (const [key, value] of Object.entries(body)) {
                this.appendFormValue(params, key, value);
            }
            return params.toString();
        }
        params.append('value', String(body));
        return params.toString();
    }
    appendFormValue(params, key, value) {
        if (value == null) {
            return;
        }
        if (Array.isArray(value)) {
            value.forEach((item) => this.appendFormValue(params, key, item));
            return;
        }
        if (value instanceof Date) {
            params.append(key, value.toISOString());
            return;
        }
        if (typeof value === 'object') {
            params.append(key, JSON.stringify(value));
            return;
        }
        params.append(key, String(value));
    }
    setAuthToken(token) {
        super.setAuthToken(token);
    }
    setAccessToken(token) {
        const headers = this.getInternalHeaders();
        headers[HttpClient.ACCESS_TOKEN_HEADER] = token;
        super.setAccessToken(token);
    }
    setTokenManager(manager) {
        const baseProto = Object.getPrototypeOf(HttpClient.prototype);
        if (typeof baseProto.setTokenManager === 'function') {
            baseProto.setTokenManager.call(this, manager);
            return;
        }
        this.getInternalAuthConfig().tokenManager = manager;
    }
    applySdkworkAuthHeaders(headers) {
        const authConfig = this.getInternalAuthConfig();
        const tokenManager = authConfig.tokenManager;
        const accessToken = tokenManager?.getAccessToken?.();
        if (!accessToken) {
            return headers;
        }
        return {
            ...(headers ?? {}),
            [HttpClient.ACCESS_TOKEN_HEADER]: accessToken,
        };
    }
    async request(path, options = {}) {
        const execute = this.execute;
        if (typeof execute !== 'function') {
            throw new Error('BaseHttpClient execute method is not available');
        }
        const { body, headers, contentType, method = 'GET', skipAuth, ...rest } = options;
        const requestHeaders = skipAuth ? headers : this.applySdkworkAuthHeaders(headers);
        return withRetry(() => execute.call(this, {
            url: path,
            method,
            ...rest,
            skipAuth,
            body: this.buildRequestBody(body, contentType),
            headers: this.buildRequestHeaders(requestHeaders, body == null ? undefined : contentType),
        }), { maxRetries: 3 });
    }
    async *streamJson(path, options = {}) {
        const stream = BaseHttpClient.prototype.stream;
        if (typeof stream !== 'function') {
            throw new Error('BaseHttpClient stream method is not available');
        }
        const { body, headers, contentType, method = 'GET', skipAuth, ...rest } = options;
        const authHeaders = skipAuth ? headers : this.applySdkworkAuthHeaders(headers);
        const requestHeaders = this.buildRequestHeaders({ Accept: 'text/event-stream', ...(authHeaders ?? {}) }, body == null ? undefined : contentType);
        for await (const data of stream.call(this, path, {
            method,
            ...rest,
            skipAuth,
            body: this.buildRequestBody(body, contentType),
            headers: requestHeaders,
        })) {
            if (data === '[DONE]') {
                return;
            }
            if (typeof data !== 'string' || data.trim().length === 0) {
                continue;
            }
            yield JSON.parse(data);
        }
    }
    async get(path, params, headers) {
        return this.request(path, { method: 'GET', params, headers });
    }
    async post(path, body, params, headers, contentType) {
        return this.request(path, { method: 'POST', body, params, headers, contentType });
    }
    async put(path, body, params, headers, contentType) {
        return this.request(path, { method: 'PUT', body, params, headers, contentType });
    }
    async delete(path, params, headers) {
        return this.request(path, { method: 'DELETE', params, headers });
    }
    async patch(path, body, params, headers, contentType) {
        return this.request(path, { method: 'PATCH', body, params, headers, contentType });
    }
}
HttpClient.ACCESS_TOKEN_HEADER = 'Access-Token';
function createHttpClient(config) {
    return new HttpClient(config);
}

const APP_API_PREFIX = '/app/v3/api';
function appApiPath(path) {
    if (!path) {
        return APP_API_PREFIX;
    }
    if (/^https?:\/\//i.test(path)) {
        return path;
    }
    const normalizedPrefixRaw = (APP_API_PREFIX).trim();
    const normalizedPrefix = normalizedPrefixRaw
        ? `/${normalizedPrefixRaw.replace(/^\/+|\/+$/g, '')}`
        : '';
    const normalizedPath = path.startsWith('/') ? path : `/${path}`;
    if (!normalizedPrefix || normalizedPrefix === '/') {
        return normalizedPath;
    }
    if (normalizedPath === normalizedPrefix || normalizedPath.startsWith(`${normalizedPrefix}/`)) {
        return normalizedPath;
    }
    return `${normalizedPrefix}${normalizedPath}`;
}

class AiMemoryRetrievalIndexesApi {
    constructor(client) {
        this.client = client;
    }
    /** List retrieval indexes for one agent memory record */
    async list(memoryId, params) {
        const query = buildQueryString([
            { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
            { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
        ]);
        return this.client.get(appendQueryString(appApiPath(`/ai/memory_records/${serializePathParameter(memoryId, { name: 'memoryId', style: 'simple', explode: false })}/retrieval_indexes`), query));
    }
    /** Upsert an agent memory retrieval index */
    async upsert(body) {
        return this.client.post(appApiPath(`/ai/memory_retrieval_indexes`), body, undefined, undefined, 'application/json');
    }
}
class AiMemoryRelationsApi {
    constructor(client) {
        this.client = client;
    }
    /** List graph relations for one agent memory record */
    async list(memoryId, params) {
        const query = buildQueryString([
            { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
            { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
        ]);
        return this.client.get(appendQueryString(appApiPath(`/ai/memory_records/${serializePathParameter(memoryId, { name: 'memoryId', style: 'simple', explode: false })}/relations`), query));
    }
    /** Create a graph relation for one agent memory record */
    async create(memoryId, body) {
        return this.client.post(appApiPath(`/ai/memory_records/${serializePathParameter(memoryId, { name: 'memoryId', style: 'simple', explode: false })}/relations`), body, undefined, undefined, 'application/json');
    }
}
class AiMemorySourcesApi {
    constructor(client) {
        this.client = client;
    }
    /** List provenance sources for one agent memory record */
    async list(memoryId, params) {
        const query = buildQueryString([
            { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
            { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
        ]);
        return this.client.get(appendQueryString(appApiPath(`/ai/memory_records/${serializePathParameter(memoryId, { name: 'memoryId', style: 'simple', explode: false })}/sources`), query));
    }
    /** Create a provenance source for one agent memory record */
    async create(memoryId, body) {
        return this.client.post(appApiPath(`/ai/memory_records/${serializePathParameter(memoryId, { name: 'memoryId', style: 'simple', explode: false })}/sources`), body, undefined, undefined, 'application/json');
    }
}
class AiMemoryRecordsApi {
    constructor(client) {
        this.client = client;
    }
    /** List agent memory records in one namespace */
    async list(memoryNamespaceId, params) {
        const query = buildQueryString([
            { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
            { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
        ]);
        return this.client.get(appendQueryString(appApiPath(`/ai/memory_namespaces/${serializePathParameter(memoryNamespaceId, { name: 'memoryNamespaceId', style: 'simple', explode: false })}/records`), query));
    }
    /** Create an agent memory record in one namespace */
    async create(memoryNamespaceId, body) {
        return this.client.post(appApiPath(`/ai/memory_namespaces/${serializePathParameter(memoryNamespaceId, { name: 'memoryNamespaceId', style: 'simple', explode: false })}/records`), body, undefined, undefined, 'application/json');
    }
    /** Retrieve one agent memory record */
    async retrieve(memoryId) {
        return this.client.get(appApiPath(`/ai/memory_records/${serializePathParameter(memoryId, { name: 'memoryId', style: 'simple', explode: false })}`));
    }
    /** Soft-delete one agent memory record */
    async delete(memoryId, params) {
        const query = buildQueryString([
            { name: 'expected_version', value: params.expectedVersion, style: 'form', explode: true, allowReserved: false },
            { name: 'requested_at', value: params.requestedAt, style: 'form', explode: true, allowReserved: false },
        ]);
        return this.client.delete(appendQueryString(appApiPath(`/ai/memory_records/${serializePathParameter(memoryId, { name: 'memoryId', style: 'simple', explode: false })}`), query));
    }
    /** Restore one soft-deleted agent memory record */
    async restore(memoryId, body) {
        return this.client.post(appApiPath(`/ai/memory_records/${serializePathParameter(memoryId, { name: 'memoryId', style: 'simple', explode: false })}/restore`), body, undefined, undefined, 'application/json');
    }
}
class AiMemoryNamespacesApi {
    constructor(client) {
        this.client = client;
    }
    /** Create an agent memory namespace */
    async create(body) {
        return this.client.post(appApiPath(`/ai/memory_namespaces`), body, undefined, undefined, 'application/json');
    }
    /** Retrieve one agent memory namespace */
    async retrieve(memoryNamespaceId) {
        return this.client.get(appApiPath(`/ai/memory_namespaces/${serializePathParameter(memoryNamespaceId, { name: 'memoryNamespaceId', style: 'simple', explode: false })}`));
    }
}
class AiMemoryBindingsApi {
    constructor(client) {
        this.client = client;
    }
    /** Create an agent memory profile binding */
    async create(memoryProfileId, body) {
        return this.client.post(appApiPath(`/ai/memory_profiles/${serializePathParameter(memoryProfileId, { name: 'memoryProfileId', style: 'simple', explode: false })}/bindings`), body, undefined, undefined, 'application/json');
    }
    /** Retrieve one agent memory binding */
    async retrieve(memoryBindingId) {
        return this.client.get(appApiPath(`/ai/memory_bindings/${serializePathParameter(memoryBindingId, { name: 'memoryBindingId', style: 'simple', explode: false })}`));
    }
}
class AiMemoryProfilesApi {
    constructor(client) {
        this.client = client;
    }
    /** Create an agent memory profile for one store */
    async create(memoryStoreId, body) {
        return this.client.post(appApiPath(`/ai/memory_stores/${serializePathParameter(memoryStoreId, { name: 'memoryStoreId', style: 'simple', explode: false })}/profiles`), body, undefined, undefined, 'application/json');
    }
    /** Retrieve one agent memory profile */
    async retrieve(memoryProfileId) {
        return this.client.get(appApiPath(`/ai/memory_profiles/${serializePathParameter(memoryProfileId, { name: 'memoryProfileId', style: 'simple', explode: false })}`));
    }
}
class AiMemoryStoresApi {
    constructor(client) {
        this.client = client;
    }
    /** Create an agent memory store */
    async create(body) {
        return this.client.post(appApiPath(`/ai/memory_stores`), body, undefined, undefined, 'application/json');
    }
    /** Retrieve one agent memory store */
    async retrieve(memoryStoreId) {
        return this.client.get(appApiPath(`/ai/memory_stores/${serializePathParameter(memoryStoreId, { name: 'memoryStoreId', style: 'simple', explode: false })}`));
    }
    /** Update one agent memory store */
    async update(memoryStoreId, body) {
        return this.client.patch(appApiPath(`/ai/memory_stores/${serializePathParameter(memoryStoreId, { name: 'memoryStoreId', style: 'simple', explode: false })}`), body, undefined, undefined, 'application/json');
    }
}
class AiKnowledgeSyncJobsApi {
    constructor(client) {
        this.client = client;
    }
    /** List sync jobs for one agent knowledge base */
    async list(knowledgeBaseId, params) {
        const query = buildQueryString([
            { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
            { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
        ]);
        return this.client.get(appendQueryString(appApiPath(`/ai/knowledge_bases/${serializePathParameter(knowledgeBaseId, { name: 'knowledgeBaseId', style: 'simple', explode: false })}/sync_jobs`), query));
    }
    /** Create a sync job for one agent knowledge base */
    async create(knowledgeBaseId, body) {
        return this.client.post(appApiPath(`/ai/knowledge_bases/${serializePathParameter(knowledgeBaseId, { name: 'knowledgeBaseId', style: 'simple', explode: false })}/sync_jobs`), body, undefined, undefined, 'application/json');
    }
    /** Retrieve one agent knowledge sync job */
    async retrieve(syncJobId) {
        return this.client.get(appApiPath(`/ai/knowledge_sync_jobs/${serializePathParameter(syncJobId, { name: 'syncJobId', style: 'simple', explode: false })}`));
    }
    /** Start one agent knowledge sync job */
    async start(syncJobId, body) {
        return this.client.post(appApiPath(`/ai/knowledge_sync_jobs/${serializePathParameter(syncJobId, { name: 'syncJobId', style: 'simple', explode: false })}/start`), body, undefined, undefined, 'application/json');
    }
    /** Complete one agent knowledge sync job */
    async complete(syncJobId, body) {
        return this.client.post(appApiPath(`/ai/knowledge_sync_jobs/${serializePathParameter(syncJobId, { name: 'syncJobId', style: 'simple', explode: false })}/complete`), body, undefined, undefined, 'application/json');
    }
    /** Fail one agent knowledge sync job */
    async fail(syncJobId, body) {
        return this.client.post(appApiPath(`/ai/knowledge_sync_jobs/${serializePathParameter(syncJobId, { name: 'syncJobId', style: 'simple', explode: false })}/fail`), body, undefined, undefined, 'application/json');
    }
    /** Cancel one agent knowledge sync job */
    async cancel(syncJobId, body) {
        return this.client.post(appApiPath(`/ai/knowledge_sync_jobs/${serializePathParameter(syncJobId, { name: 'syncJobId', style: 'simple', explode: false })}/cancel`), body, undefined, undefined, 'application/json');
    }
}
class AiKnowledgeBindingsApi {
    constructor(client) {
        this.client = client;
    }
    /** List bindings for one agent knowledge base */
    async list(knowledgeBaseId, params) {
        const query = buildQueryString([
            { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
            { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
        ]);
        return this.client.get(appendQueryString(appApiPath(`/ai/knowledge_bases/${serializePathParameter(knowledgeBaseId, { name: 'knowledgeBaseId', style: 'simple', explode: false })}/bindings`), query));
    }
    /** Create a binding for one agent knowledge base */
    async create(knowledgeBaseId, body) {
        return this.client.post(appApiPath(`/ai/knowledge_bases/${serializePathParameter(knowledgeBaseId, { name: 'knowledgeBaseId', style: 'simple', explode: false })}/bindings`), body, undefined, undefined, 'application/json');
    }
    /** Retrieve one agent knowledge binding */
    async retrieve(knowledgeBindingId) {
        return this.client.get(appApiPath(`/ai/knowledge_bindings/${serializePathParameter(knowledgeBindingId, { name: 'knowledgeBindingId', style: 'simple', explode: false })}`));
    }
}
class AiKnowledgeIndexesApi {
    constructor(client) {
        this.client = client;
    }
    /** List indexes for one agent knowledge document */
    async list(knowledgeDocumentId, params) {
        const query = buildQueryString([
            { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
            { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
        ]);
        return this.client.get(appendQueryString(appApiPath(`/ai/knowledge_documents/${serializePathParameter(knowledgeDocumentId, { name: 'knowledgeDocumentId', style: 'simple', explode: false })}/indexes`), query));
    }
    /** Upsert an agent knowledge retrieval index */
    async upsert(body) {
        return this.client.post(appApiPath(`/ai/knowledge_indexes`), body, undefined, undefined, 'application/json');
    }
    /** Retrieve one agent knowledge retrieval index */
    async retrieve(knowledgeIndexId) {
        return this.client.get(appApiPath(`/ai/knowledge_indexes/${serializePathParameter(knowledgeIndexId, { name: 'knowledgeIndexId', style: 'simple', explode: false })}`));
    }
}
class AiKnowledgeChunksApi {
    constructor(client) {
        this.client = client;
    }
    /** List chunks for one agent knowledge document */
    async list(knowledgeDocumentId, params) {
        const query = buildQueryString([
            { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
            { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
        ]);
        return this.client.get(appendQueryString(appApiPath(`/ai/knowledge_documents/${serializePathParameter(knowledgeDocumentId, { name: 'knowledgeDocumentId', style: 'simple', explode: false })}/chunks`), query));
    }
    /** Create a chunk for one agent knowledge document */
    async create(knowledgeDocumentId, body) {
        return this.client.post(appApiPath(`/ai/knowledge_documents/${serializePathParameter(knowledgeDocumentId, { name: 'knowledgeDocumentId', style: 'simple', explode: false })}/chunks`), body, undefined, undefined, 'application/json');
    }
    /** Retrieve one agent knowledge chunk */
    async retrieve(knowledgeChunkId) {
        return this.client.get(appApiPath(`/ai/knowledge_chunks/${serializePathParameter(knowledgeChunkId, { name: 'knowledgeChunkId', style: 'simple', explode: false })}`));
    }
}
class AiKnowledgeReadApi {
    constructor(client) {
        this.client = client;
    }
    /** Read one provider-neutral knowledge document */
    async read(knowledgeDocumentId) {
        return this.client.get(appApiPath(`/ai/knowledge_documents/${serializePathParameter(knowledgeDocumentId, { name: 'knowledgeDocumentId', style: 'simple', explode: false })}`));
    }
}
class AiKnowledgeSearchApi {
    constructor(client) {
        this.client = client;
    }
    /** Search an agent knowledge base for provider-neutral RAG candidates */
    async search(knowledgeBaseId, body) {
        return this.client.post(appApiPath(`/ai/knowledge_bases/${serializePathParameter(knowledgeBaseId, { name: 'knowledgeBaseId', style: 'simple', explode: false })}/search`), body, undefined, undefined, 'application/json');
    }
}
class AiKnowledgeDocumentsApi {
    constructor(client) {
        this.client = client;
    }
    /** Create a document for one agent knowledge base */
    async create(knowledgeBaseId, body) {
        return this.client.post(appApiPath(`/ai/knowledge_bases/${serializePathParameter(knowledgeBaseId, { name: 'knowledgeBaseId', style: 'simple', explode: false })}/documents`), body, undefined, undefined, 'application/json');
    }
    /** Update one agent knowledge document */
    async update(knowledgeDocumentId, body) {
        return this.client.patch(appApiPath(`/ai/knowledge_documents/${serializePathParameter(knowledgeDocumentId, { name: 'knowledgeDocumentId', style: 'simple', explode: false })}`), body, undefined, undefined, 'application/json');
    }
    /** Soft-delete one agent knowledge document */
    async delete(knowledgeDocumentId, params) {
        const query = buildQueryString([
            { name: 'expected_version', value: params.expectedVersion, style: 'form', explode: true, allowReserved: false },
            { name: 'requested_at', value: params.requestedAt, style: 'form', explode: true, allowReserved: false },
        ]);
        return this.client.delete(appendQueryString(appApiPath(`/ai/knowledge_documents/${serializePathParameter(knowledgeDocumentId, { name: 'knowledgeDocumentId', style: 'simple', explode: false })}`), query));
    }
    /** Restore one soft-deleted agent knowledge document */
    async restore(knowledgeDocumentId, body) {
        return this.client.post(appApiPath(`/ai/knowledge_documents/${serializePathParameter(knowledgeDocumentId, { name: 'knowledgeDocumentId', style: 'simple', explode: false })}/restore`), body, undefined, undefined, 'application/json');
    }
}
class AiKnowledgeListApi {
    constructor(client) {
        this.client = client;
    }
    /** List provider-neutral knowledge documents for one agent knowledge base */
    async list(knowledgeBaseId, params) {
        const query = buildQueryString([
            { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
            { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
        ]);
        return this.client.get(appendQueryString(appApiPath(`/ai/knowledge_bases/${serializePathParameter(knowledgeBaseId, { name: 'knowledgeBaseId', style: 'simple', explode: false })}/documents`), query));
    }
}
class AiKnowledgeSourcesApi {
    constructor(client) {
        this.client = client;
    }
    /** List sources for one agent knowledge base */
    async list(knowledgeBaseId, params) {
        const query = buildQueryString([
            { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
            { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
        ]);
        return this.client.get(appendQueryString(appApiPath(`/ai/knowledge_bases/${serializePathParameter(knowledgeBaseId, { name: 'knowledgeBaseId', style: 'simple', explode: false })}/sources`), query));
    }
    /** Create a source for one agent knowledge base */
    async create(knowledgeBaseId, body) {
        return this.client.post(appApiPath(`/ai/knowledge_bases/${serializePathParameter(knowledgeBaseId, { name: 'knowledgeBaseId', style: 'simple', explode: false })}/sources`), body, undefined, undefined, 'application/json');
    }
    /** Retrieve one agent knowledge source */
    async retrieve(knowledgeSourceId) {
        return this.client.get(appApiPath(`/ai/knowledge_sources/${serializePathParameter(knowledgeSourceId, { name: 'knowledgeSourceId', style: 'simple', explode: false })}`));
    }
    /** Update one agent knowledge source */
    async update(knowledgeSourceId, body) {
        return this.client.patch(appApiPath(`/ai/knowledge_sources/${serializePathParameter(knowledgeSourceId, { name: 'knowledgeSourceId', style: 'simple', explode: false })}`), body, undefined, undefined, 'application/json');
    }
    /** Soft-delete one agent knowledge source */
    async delete(knowledgeSourceId, params) {
        const query = buildQueryString([
            { name: 'expected_version', value: params.expectedVersion, style: 'form', explode: true, allowReserved: false },
            { name: 'requested_at', value: params.requestedAt, style: 'form', explode: true, allowReserved: false },
        ]);
        return this.client.delete(appendQueryString(appApiPath(`/ai/knowledge_sources/${serializePathParameter(knowledgeSourceId, { name: 'knowledgeSourceId', style: 'simple', explode: false })}`), query));
    }
    /** Restore one soft-deleted agent knowledge source */
    async restore(knowledgeSourceId, body) {
        return this.client.post(appApiPath(`/ai/knowledge_sources/${serializePathParameter(knowledgeSourceId, { name: 'knowledgeSourceId', style: 'simple', explode: false })}/restore`), body, undefined, undefined, 'application/json');
    }
}
class AiKnowledgeBasesApi {
    constructor(client) {
        this.client = client;
    }
    /** List agent knowledge bases */
    async list(params) {
        const query = buildQueryString([
            { name: 'include_deleted', value: params?.includeDeleted, style: 'form', explode: true, allowReserved: false },
            { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
            { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
            { name: 'q', value: params?.q, style: 'form', explode: true, allowReserved: false },
        ]);
        return this.client.get(appendQueryString(appApiPath(`/ai/knowledge_bases`), query));
    }
    /** Create an agent knowledge base */
    async create(body) {
        return this.client.post(appApiPath(`/ai/knowledge_bases`), body, undefined, undefined, 'application/json');
    }
    /** Retrieve one agent knowledge base */
    async retrieve(knowledgeBaseId) {
        return this.client.get(appApiPath(`/ai/knowledge_bases/${serializePathParameter(knowledgeBaseId, { name: 'knowledgeBaseId', style: 'simple', explode: false })}`));
    }
    /** Update one agent knowledge base */
    async update(knowledgeBaseId, body) {
        return this.client.patch(appApiPath(`/ai/knowledge_bases/${serializePathParameter(knowledgeBaseId, { name: 'knowledgeBaseId', style: 'simple', explode: false })}`), body, undefined, undefined, 'application/json');
    }
    /** Soft-delete one agent knowledge base */
    async delete(knowledgeBaseId, params) {
        const query = buildQueryString([
            { name: 'expected_version', value: params.expectedVersion, style: 'form', explode: true, allowReserved: false },
            { name: 'requested_at', value: params.requestedAt, style: 'form', explode: true, allowReserved: false },
        ]);
        return this.client.delete(appendQueryString(appApiPath(`/ai/knowledge_bases/${serializePathParameter(knowledgeBaseId, { name: 'knowledgeBaseId', style: 'simple', explode: false })}`), query));
    }
    /** Restore one soft-deleted agent knowledge base */
    async restore(knowledgeBaseId, body) {
        return this.client.post(appApiPath(`/ai/knowledge_bases/${serializePathParameter(knowledgeBaseId, { name: 'knowledgeBaseId', style: 'simple', explode: false })}/restore`), body, undefined, undefined, 'application/json');
    }
}
class AiAgentsPromptOptimizationsApi {
    constructor(client) {
        this.client = client;
    }
    /** Create a prompt optimization for one managed agent */
    async create(agentId, body) {
        return this.client.post(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/prompt_optimizations`), body, undefined, undefined, 'application/json');
    }
}
class AiAgentsPreviewResponsesApi {
    constructor(client) {
        this.client = client;
    }
    /** Create a preview response for one managed agent */
    async create(agentId, body) {
        return this.client.post(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/preview_responses`), body, undefined, undefined, 'application/json');
    }
}
class AiAgentsDeploymentsApi {
    constructor(client) {
        this.client = client;
    }
    /** List deployments for one managed agent */
    async list(agentId, params) {
        const query = buildQueryString([
            { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
            { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
        ]);
        return this.client.get(appendQueryString(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/deployments`), query));
    }
    /** Create a deployment snapshot for one managed agent provider binding */
    async create(agentId, body) {
        return this.client.post(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/deployments`), body, undefined, undefined, 'application/json');
    }
}
class AiAgentsProviderBindingsApi {
    constructor(client) {
        this.client = client;
    }
    /** List provider bindings for one managed agent */
    async list(agentId, params) {
        const query = buildQueryString([
            { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
            { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
        ]);
        return this.client.get(appendQueryString(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/provider_bindings`), query));
    }
    /** Create a provider binding for one managed agent */
    async create(agentId, body) {
        return this.client.post(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/provider_bindings`), body, undefined, undefined, 'application/json');
    }
    /** Activate one managed agent provider binding */
    async activate(agentId, bindingId, body) {
        return this.client.post(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/provider_bindings/${serializePathParameter(bindingId, { name: 'bindingId', style: 'simple', explode: false })}/activate`), body, undefined, undefined, 'application/json');
    }
}
class AiAgentsApi {
    constructor(client) {
        this.client = client;
        this.providerBindings = new AiAgentsProviderBindingsApi(client);
        this.deployments = new AiAgentsDeploymentsApi(client);
        this.previewResponses = new AiAgentsPreviewResponsesApi(client);
        this.promptOptimizations = new AiAgentsPromptOptimizationsApi(client);
    }
    /** List managed agents */
    async list(params) {
        const query = buildQueryString([
            { name: 'include_deleted', value: params?.includeDeleted, style: 'form', explode: true, allowReserved: false },
            { name: 'page', value: params?.page, style: 'form', explode: true, allowReserved: false },
            { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
            { name: 'q', value: params?.q, style: 'form', explode: true, allowReserved: false },
        ]);
        return this.client.get(appendQueryString(appApiPath(`/ai/agents`), query));
    }
    /** Create a managed agent */
    async create(body) {
        return this.client.post(appApiPath(`/ai/agents`), body, undefined, undefined, 'application/json');
    }
    /** Retrieve one managed agent */
    async retrieve(agentId) {
        return this.client.get(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}`));
    }
    /** Update one managed agent */
    async update(agentId, body) {
        return this.client.patch(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}`), body, undefined, undefined, 'application/json');
    }
    /** Soft-delete one managed agent */
    async delete(agentId) {
        return this.client.delete(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}`));
    }
    /** Restore one soft-deleted managed agent */
    async restore(agentId, body) {
        return this.client.post(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: 'agentId', style: 'simple', explode: false })}/restore`), body, undefined, undefined, 'application/json');
    }
}
class AiApi {
    constructor(client) {
        this.client = client;
        this.agents = new AiAgentsApi(client);
        this.knowledgeBases = new AiKnowledgeBasesApi(client);
        this.knowledgeSources = new AiKnowledgeSourcesApi(client);
        this.knowledgeList = new AiKnowledgeListApi(client);
        this.knowledgeDocuments = new AiKnowledgeDocumentsApi(client);
        this.knowledgeSearch = new AiKnowledgeSearchApi(client);
        this.knowledgeRead = new AiKnowledgeReadApi(client);
        this.knowledgeChunks = new AiKnowledgeChunksApi(client);
        this.knowledgeIndexes = new AiKnowledgeIndexesApi(client);
        this.knowledgeBindings = new AiKnowledgeBindingsApi(client);
        this.knowledgeSyncJobs = new AiKnowledgeSyncJobsApi(client);
        this.memoryStores = new AiMemoryStoresApi(client);
        this.memoryProfiles = new AiMemoryProfilesApi(client);
        this.memoryBindings = new AiMemoryBindingsApi(client);
        this.memoryNamespaces = new AiMemoryNamespacesApi(client);
        this.memoryRecords = new AiMemoryRecordsApi(client);
        this.memorySources = new AiMemorySourcesApi(client);
        this.memoryRelations = new AiMemoryRelationsApi(client);
        this.memoryRetrievalIndexes = new AiMemoryRetrievalIndexesApi(client);
    }
}
function createAiApi(client) {
    return new AiApi(client);
}
function appendQueryString(path, rawQueryString) {
    const query = rawQueryString.replace(/^\?+/, '');
    if (!query) {
        return path;
    }
    return path.includes('?') ? `${path}&${query}` : `${path}?${query}`;
}
function serializePathParameter(value, spec) {
    if (value === undefined || value === null) {
        return '';
    }
    const style = spec.style || 'simple';
    if (Array.isArray(value)) {
        return serializePathArray(spec.name, value, style, spec.explode);
    }
    if (typeof value === 'object') {
        return serializePathObject(spec.name, value, style, spec.explode);
    }
    return pathPrefix(spec.name, style) + encodePathValue(serializePathPrimitive(value));
}
function serializePathArray(name, values, style, explode) {
    const serialized = values
        .filter((item) => item !== undefined && item !== null)
        .map((item) => encodePathValue(serializePathPrimitive(item)));
    if (serialized.length === 0) {
        return pathPrefix(name, style);
    }
    if (style === 'matrix') {
        return explode
            ? serialized.map((item) => `;${name}=${item}`).join('')
            : `;${name}=${serialized.join(',')}`;
    }
    return pathPrefix(name, style) + serialized.join(explode ? '.' : ',');
}
function serializePathObject(name, value, style, explode) {
    const entries = Object.entries(value).filter(([, entryValue]) => entryValue !== undefined && entryValue !== null);
    if (entries.length === 0) {
        return pathPrefix(name, style);
    }
    if (style === 'matrix') {
        return explode
            ? entries.map(([key, entryValue]) => `;${encodePathValue(key)}=${encodePathValue(serializePathPrimitive(entryValue))}`).join('')
            : `;${name}=${entries.flatMap(([key, entryValue]) => [encodePathValue(key), encodePathValue(serializePathPrimitive(entryValue))]).join(',')}`;
    }
    const serialized = explode
        ? entries.map(([key, entryValue]) => `${encodePathValue(key)}=${encodePathValue(serializePathPrimitive(entryValue))}`).join(style === 'label' ? '.' : ',')
        : entries.flatMap(([key, entryValue]) => [encodePathValue(key), encodePathValue(serializePathPrimitive(entryValue))]).join(',');
    return pathPrefix(name, style) + serialized;
}
function pathPrefix(name, style, _objectValue) {
    if (style === 'label')
        return '.';
    if (style === 'matrix')
        return `;${name}`;
    return '';
}
function encodePathValue(value) {
    return encodeURIComponent(value);
}
function serializePathPrimitive(value) {
    if (value instanceof Date) {
        return value.toISOString();
    }
    if (typeof value === 'object') {
        return JSON.stringify(value);
    }
    return String(value);
}
function buildQueryString(parameters) {
    const pairs = [];
    for (const parameter of parameters) {
        appendSerializedParameter(pairs, parameter);
    }
    return pairs.join('&');
}
function appendSerializedParameter(pairs, parameter) {
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
        appendObjectParameter(pairs, parameter.name, parameter.value, style, parameter.explode, parameter.allowReserved);
        return;
    }
    pairs.push(`${encodeQueryComponent(parameter.name)}=${encodeQueryValue(serializePrimitive(parameter.value), parameter.allowReserved)}`);
}
function appendArrayParameter(pairs, name, value, style, explode, allowReserved) {
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
function appendObjectParameter(pairs, name, value, style, explode, allowReserved) {
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
function appendDeepObjectParameter(pairs, name, value, allowReserved) {
    if (!value || typeof value !== 'object' || Array.isArray(value)) {
        pairs.push(`${encodeQueryComponent(name)}=${encodeQueryValue(serializePrimitive(value), allowReserved)}`);
        return;
    }
    for (const [key, entryValue] of Object.entries(value)) {
        if (entryValue === undefined || entryValue === null) {
            continue;
        }
        pairs.push(`${encodeQueryComponent(`${name}[${key}]`)}=${encodeQueryValue(serializePrimitive(entryValue), allowReserved)}`);
    }
}
function serializePrimitive(value) {
    if (value instanceof Date) {
        return value.toISOString();
    }
    if (typeof value === 'object') {
        return JSON.stringify(value);
    }
    return String(value);
}
function encodeQueryComponent(value) {
    return encodeURIComponent(value);
}
function encodeQueryValue(value, allowReserved) {
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

class SdkworkAppClient {
    constructor(config) {
        this.httpClient = createHttpClient(config);
        this.ai = createAiApi(this.httpClient);
    }
    setAuthToken(token) {
        this.httpClient.setAuthToken(token);
        return this;
    }
    setAccessToken(token) {
        this.httpClient.setAccessToken(token);
        return this;
    }
    setTokenManager(manager) {
        this.httpClient.setTokenManager(manager);
        return this;
    }
    get http() {
        return this.httpClient;
    }
}
function createClient(config) {
    return new SdkworkAppClient(config);
}

class BaseApi {
    constructor(http, basePath) {
        this.http = http;
        this.basePath = basePath;
    }
    async get(path, params, headers) {
        return this.http.get(`${this.basePath}${path}`, params, headers);
    }
    async post(path, body, params, headers, contentType) {
        return this.http.post(`${this.basePath}${path}`, body, params, headers, contentType);
    }
    async put(path, body, params, headers, contentType) {
        return this.http.put(`${this.basePath}${path}`, body, params, headers, contentType);
    }
    async delete(path, params, headers) {
        return this.http.delete(`${this.basePath}${path}`, params, headers);
    }
    async patch(path, body, params, headers, contentType) {
        return this.http.patch(`${this.basePath}${path}`, body, params, headers, contentType);
    }
    async request(method, path, body, params, headers, contentType) {
        return this.http.request(`${this.basePath}${path}`, { method: method, body, params, headers, contentType });
    }
}

export { AiApi, BaseApi, HttpClient, SdkworkAppClient, appApiPath, createAiApi, createClient, createHttpClient };
