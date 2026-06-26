import { useMemo, useState } from "react";
import { HashRouter, Navigate, Route, Routes, useNavigate } from "react-router-dom";

import {
  AgentView,
  CreateAgentModal,
  CreateAgentView,
  ToastContainer,
  type Agent,
} from "@sdkwork/agents-pc-agents";
import { CREATE_AGENT_ROUTE } from "@sdkwork/agents-pc-shell";

function AgentsHomePage() {
  const navigate = useNavigate();
  const [isCreateAgentModalOpen, setIsCreateAgentModalOpen] = useState(false);
  const [editAgentId, setEditAgentId] = useState<string | undefined>();

  const navigateToCreate = useMemo(
    () => ({
      onCreateAgent: () => setIsCreateAgentModalOpen(true),
      onEditAgent: (id: string) => {
        setEditAgentId(id);
        navigate(`/${CREATE_AGENT_ROUTE}`);
      },
    }),
    [navigate],
  );

  const handleStandaloneStartChat = (agent: Agent) => {
    setEditAgentId(agent.id);
    navigate(`/${CREATE_AGENT_ROUTE}`);
  };

  return (
    <div className="flex min-h-screen flex-col bg-[#141414] text-gray-100">
      <ToastContainer />
      <header className="flex items-center justify-between border-b border-white/10 px-6 py-4">
        <div>
          <h1 className="text-lg font-semibold">SDKWork Agents</h1>
          <p className="text-sm text-gray-400">智能体管理与市场</p>
        </div>
      </header>
      <main className="flex min-h-0 flex-1">
        <Routes>
          <Route
            path="/"
            element={
              <AgentView
                onStartChat={handleStandaloneStartChat}
                onCreateAgent={navigateToCreate.onCreateAgent}
                onEditAgent={navigateToCreate.onEditAgent}
              />
            }
          />
          <Route
            path={`/${CREATE_AGENT_ROUTE}`}
            element={
              <CreateAgentView
                initialAgentId={editAgentId}
                onBack={() => {
                  setEditAgentId(undefined);
                  navigate("/");
                }}
              />
            }
          />
          <Route path="*" element={<Navigate to="/" replace />} />
        </Routes>
      </main>
      <CreateAgentModal
        isOpen={isCreateAgentModalOpen}
        onClose={() => setIsCreateAgentModalOpen(false)}
        onSuccess={(agentId) => {
          setIsCreateAgentModalOpen(false);
          setEditAgentId(agentId);
          navigate(`/${CREATE_AGENT_ROUTE}`);
        }}
      />
    </div>
  );
}

export default function App() {
  return (
    <HashRouter>
      <AgentsHomePage />
    </HashRouter>
  );
}
