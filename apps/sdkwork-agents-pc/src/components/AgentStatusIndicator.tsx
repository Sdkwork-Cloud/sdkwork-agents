import React, { useState, useEffect, useRef } from 'react';
import { Bot, Wifi, Activity, Clock, Settings, List } from 'lucide-react';
import { cn } from '@sdkwork/agents-pc-commons';
import { useAgentState } from '@/src/contexts/AgentStateContext';
import { AgentConfigModal } from './AgentConfigModal';
import { AgentHistoryLog } from './AgentHistoryLog';

export const AgentStatusIndicator = () => {
  const { connectionStatus, agentState, agentName, addEvent } = useAgentState();
  const [isConfigOpen, setIsConfigOpen] = useState(false);
  const [isHistoryOpen, setIsHistoryOpen] = useState(false);
  
  // Track previous states to emit events on change
  const prevConnectionStatus = useRef(connectionStatus);
  const prevAgentState = useRef(agentState);

  useEffect(() => {
    if (prevConnectionStatus.current !== connectionStatus) {
      if (connectionStatus === 'connected') addEvent({ type: 'success', message: 'Agent connected' });
      else if (connectionStatus === 'disconnected') addEvent({ type: 'error', message: 'Agent disconnected' });
      else if (connectionStatus === 'connecting') addEvent({ type: 'info', message: 'Agent connecting...' });
      prevConnectionStatus.current = connectionStatus;
    }
  }, [connectionStatus, addEvent]);

  useEffect(() => {
    if (prevAgentState.current !== agentState) {
      if (agentState === 'active') addEvent({ type: 'info', message: 'Agent processing started' });
      else if (agentState === 'idle') addEvent({ type: 'success', message: 'Agent processing completed' });
      prevAgentState.current = agentState;
    }
  }, [agentState, addEvent]);

  return null;
};
