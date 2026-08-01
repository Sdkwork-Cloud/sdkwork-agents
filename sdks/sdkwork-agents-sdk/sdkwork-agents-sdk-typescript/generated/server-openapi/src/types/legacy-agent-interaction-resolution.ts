export interface LegacyAgentInteractionResolution {
  outcome: 'approved' | 'rejected' | 'answered' | 'expired' | 'cancelled';
  answer?: string;
  selectedOptionValue?: string;
  reason?: string;
}
