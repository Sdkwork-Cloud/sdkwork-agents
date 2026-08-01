import type { AgentInteractionAction } from './agent-interaction-action';

export interface TypedAgentInteractionResolution {
  action: AgentInteractionAction;
  answers?: Record<string, string[]>;
  selectedOptions?: string[];
  freeformAnswer?: string | null;
  selectedSources?: string[];
  selectedRoles?: string[];
  execPolicyAmendment?: Record<string, unknown>;
  networkPolicyAmendment?: Record<string, unknown>;
  permissions?: Record<string, unknown>;
  scope?: 'turn' | 'session';
  strictAutoReview?: boolean;
  content?: unknown;
  metadata?: Record<string, unknown>;
}
