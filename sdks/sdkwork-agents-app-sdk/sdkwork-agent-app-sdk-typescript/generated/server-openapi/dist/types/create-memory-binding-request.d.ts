import type { MemoryBindingScopeKind } from './memory-binding-scope-kind';
export interface CreateMemoryBindingRequest {
    memoryBindingId: string;
    agentId?: string | null;
    deploymentId?: string | null;
    scopeKind: MemoryBindingScopeKind;
    scopeRef: string;
    active?: boolean;
    defaultBinding?: boolean;
    requestedAt: string;
}
//# sourceMappingURL=create-memory-binding-request.d.ts.map