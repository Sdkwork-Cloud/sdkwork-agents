import React, { createContext, useContext, useState, ReactNode, useCallback } from 'react';
import { uuid } from '@sdkwork/utils';

export type ConnectionStatus = 'connected' | 'connecting' | 'disconnected';
export type AgentState = 'idle' | 'active';

export interface AgentEvent {
  id: string;
  timestamp: number;
  type: 'info' | 'success' | 'warning' | 'error';
  message: string;
}

interface AgentStateContextType {
  connectionStatus: ConnectionStatus;
  setConnectionStatus: (status: ConnectionStatus) => void;
  agentState: AgentState;
  setAgentState: (state: AgentState) => void;
  agentName: string;
  setAgentName: (name: string) => void;
  systemPrompt: string;
  setSystemPrompt: (prompt: string) => void;
  activeModel: string;
  setActiveModel: (model: string) => void;
  events: AgentEvent[];
  addEvent: (event: Omit<AgentEvent, 'id' | 'timestamp'>) => void;
  clearEvents: () => void;
}

const AgentStateContext = createContext<AgentStateContextType | undefined>(undefined);

export const AgentStateProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
  const [connectionStatus, setConnectionStatus] = useState<ConnectionStatus>('connected');
  const [agentState, setAgentState] = useState<AgentState>('idle');
  const [agentName, setAgentName] = useState('GPT-4o (Workbench)');
  const [systemPrompt, setSystemPrompt] = useState('You are a helpful assistant.');
  const [activeModel, setActiveModel] = useState('gpt-4o');
  const [events, setEvents] = useState<AgentEvent[]>([
    { id: uuid(), timestamp: Date.now(), type: 'info', message: 'Agent initialized' },
    { id: uuid(), timestamp: Date.now() + 1000, type: 'success', message: 'Agent connected successfully' }
  ]);

  const addEvent = useCallback((event: Omit<AgentEvent, 'id' | 'timestamp'>) => {
    setEvents(prev => [...prev, {
      ...event,
      id: uuid(),
      timestamp: Date.now()
    }]);
  }, []);

  const clearEvents = useCallback(() => {
    setEvents([]);
  }, []);

  return (
    <AgentStateContext.Provider
      value={{
        connectionStatus,
        setConnectionStatus,
        agentState,
        setAgentState,
        agentName,
        setAgentName,
        systemPrompt,
        setSystemPrompt,
        activeModel,
        setActiveModel,
        events,
        addEvent,
        clearEvents,
      }}
    >
      {children}
    </AgentStateContext.Provider>
  );
};

export const useAgentState = () => {
  const context = useContext(AgentStateContext);
  if (context === undefined) {
    throw new Error('useAgentState must be used within an AgentStateProvider');
  }
  return context;
};
