/** Provider-neutral kernel runtime event emitted during one agent turn. */
export interface AgentTurnRuntimeEvent {
  eventId: string;
  type: string;
  version: string;
  sequence: number;
  occurredAt: string | null;
  source: 'runtime' | 'manifest' | 'provider' | 'model' | 'tool' | 'context' | 'memory' | 'policy' | 'host' | 'protocol_adapter' | 'kernel_ui' | 'code_kernel' | 'telemetry' | 'unknown';
  severity: 'debug' | 'info' | 'warn' | 'error';
  sessionId: string;
  turnId: string;
  providerSessionId: string | null;
  taskId: string | null;
  runId: string | null;
  itemId: string | null;
  traceContext: { traceId: string; spanId: string; parentSpanId: string | null; } | null;
  correlationId: string | null;
  causationId: string | null;
  redactionClassification: 'public' | 'internal' | 'tenant_sensitive' | 'personal_data' | 'secret' | 'regulated' | 'unknown';
  payloadSchema: string | null;
  payload: Record<string, unknown>;
  replay: boolean;
}
