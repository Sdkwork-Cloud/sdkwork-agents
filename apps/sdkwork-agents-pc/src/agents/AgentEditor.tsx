import { CreateAgentView, ToastContainer } from '@sdkwork/agents-pc-agents';

interface AgentEditorProps {
  agentId: string;
  onBack: () => void;
}

export function AgentEditor({ agentId, onBack }: AgentEditorProps) {
  return (
    <div className="flex h-full min-h-0 bg-[#111113]">
      <ToastContainer />
      <CreateAgentView initialAgentId={agentId} onBack={onBack} />
    </div>
  );
}
