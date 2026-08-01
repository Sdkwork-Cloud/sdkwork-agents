import type { AgentInteractionQuestionOption } from './agent-interaction-question-option';

export interface AgentInteractionQuestion {
  id: string;
  header: string;
  prompt: string;
  allowOther: boolean;
  secret: boolean;
  options: AgentInteractionQuestionOption[] | null;
}
