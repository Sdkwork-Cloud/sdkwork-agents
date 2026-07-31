export interface ClaimNextAgentTurnInputQueueEntryRequest {
  claimOwner: string;
  leaseSeconds?: number;
  requestedAt: string;
}
