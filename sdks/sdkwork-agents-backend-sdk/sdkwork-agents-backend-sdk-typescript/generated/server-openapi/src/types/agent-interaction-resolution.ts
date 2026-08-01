import type { LegacyAgentInteractionResolution } from './legacy-agent-interaction-resolution';
import type { TypedAgentInteractionResolution } from './typed-agent-interaction-resolution';

export type AgentInteractionResolution = LegacyAgentInteractionResolution | TypedAgentInteractionResolution;
