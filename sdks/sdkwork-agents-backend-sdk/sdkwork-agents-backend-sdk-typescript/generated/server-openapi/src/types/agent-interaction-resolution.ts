export interface AgentInteractionResolution {
  outcome: 'approved' | 'rejected' | 'answered' | 'expired' | 'cancelled';
  answer?: string;
  selectedOptionValue?: string;
  reason?: string;
}
