import { useMemo, useState } from "react";
import { HashRouter, Navigate, Route, Routes, useLocation, useNavigate, useParams } from "react-router-dom";

import {
  AgentChatView,
  AgentMarketplaceMobileView,
  AgentMarketplaceSearchView,
  AgentView,
  CreateAgentModal,
  CreateAgentView,
  ToastContainer,
  type Agent,
  type AgentConfig,
} from "@sdkwork/agents-h5-agents";
import { CHAT_ROUTE, CREATE_AGENT_ROUTE } from "@sdkwork/agents-h5-shell";

import { AuthGate } from "./components/AuthGate";

interface ChatRouteState {
  agent?: Agent;
}

function AgentChatRoutePage() {
  const navigate = useNavigate();
  const { agentId = "" } = useParams();
  const location = useLocation();
  const state = (location.state ?? {}) as ChatRouteState;

  if (!agentId) {
    return <Navigate to="/" replace />;
  }

  return (
    <AgentChatView
      agentId={agentId}
      agentName={state.agent?.name}
      welcomeMessage={state.agent?.welcomeMessage}
      onBack={() => navigate("/")}
    />
  );
}

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
    navigate(`/${CHAT_ROUTE}/${agent.id}`, { state: { agent } });
  };

  /** Mobile marketplace rows carry `AgentConfig`; map onto the route state shape. */
  const handleMobileStartChat = (agent: AgentConfig) => {
    navigate(`/${CHAT_ROUTE}/${agent.id}`, {
      state: {
        agent: {
          id: agent.id ?? "",
          name: agent.name,
          desc: agent.description,
          author: agent.author,
          users: agent.users,
          avatar: agent.avatar,
          welcomeMessage: agent.welcomeMessage,
        } satisfies Agent,
      },
    });
  };

  return (
    <div className="flex min-h-screen flex-col bg-[#141414] text-gray-100">
      <ToastContainer />
      <header className="flex items-center justify-between border-b border-white/10 px-4 py-3">
        <div>
          <h1 className="text-base font-semibold">SDKWork Agents</h1>
          <p className="text-xs text-gray-400">智能体管理与市场</p>
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
          <Route path={`/${CHAT_ROUTE}/:agentId`} element={<AgentChatRoutePage />} />
          {/* Mobile marketplace surfaces (hosted standalone for verification). */}
          <Route
            path="/mobile/market"
            element={
              <AgentMarketplaceMobileView
                onStartChat={handleMobileStartChat}
                onCreateAgent={navigateToCreate.onCreateAgent}
                onSearch={() => navigate("/mobile/market/search")}
              />
            }
          />
          <Route
            path="/mobile/market/search"
            element={
              <AgentMarketplaceSearchView
                onStartChat={handleMobileStartChat}
                onBack={() => navigate("/mobile/market")}
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
    <AuthGate>
      <HashRouter>
        <AgentsHomePage />
      </HashRouter>
    </AuthGate>
  );
}
