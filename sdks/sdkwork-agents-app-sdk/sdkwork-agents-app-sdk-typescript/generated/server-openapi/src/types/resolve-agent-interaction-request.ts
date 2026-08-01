import type { Int64String } from './int64-string';
import type { TypedAgentInteractionResolution } from './typed-agent-interaction-resolution';

export interface ResolveAgentInteractionRequest {
  resolution: TypedAgentInteractionResolution;
  claimToken: string;
  fencingToken: Int64String;
  expectedVersion: Int64String;
  requestedAt: string;
}
