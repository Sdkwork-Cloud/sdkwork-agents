export interface CreateAgentPreviewResponseRequest {
    executionId: string;
    content: string;
    debugMode?: boolean;
    memoryEnabled?: boolean;
    model?: string;
    temperature?: number;
    inputPayload?: Record<string, unknown>;
    requestedAt: string;
}
//# sourceMappingURL=create-agent-preview-response-request.d.ts.map