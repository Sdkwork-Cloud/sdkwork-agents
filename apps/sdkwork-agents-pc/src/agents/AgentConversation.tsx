import { AgentChatView } from '@sdkwork/agents-pc-agents';

interface AgentConversationProps {
  agentId: string;
  agentName: string;
  welcomeMessage?: string;
  onBack: () => void;
}

export function AgentConversation(props: AgentConversationProps) {
  return (
    <AgentChatView
      agentId={props.agentId}
      agentName={props.agentName}
      welcomeMessage={props.welcomeMessage}
      onBack={props.onBack}
    />
  );
}
