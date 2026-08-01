export interface AgentInteractionCorrelation {
  modelRequestId: string;
  providerId: string;
  providerInteractionId: string | null;
  providerItemId: string | null;
  providerRequestId: string | number;
  providerRequestIdType: 'string' | 'number';
  providerSessionId: string;
  providerToolCallId: string | null;
  providerToolName: string | null;
  providerToolNamespace: string | null;
  providerTurnId: string;
  protocolMethod: string;
}
