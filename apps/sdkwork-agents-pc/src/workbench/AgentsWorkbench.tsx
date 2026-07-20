import '@sdkwork/agents-pc-commons/i18n';

import { ThemeProvider } from '@sdkwork/agents-pc-commons';

import { WorkbenchLayout, type WorkbenchViewportMode } from '../components/WorkbenchLayout';
import { AgentStateProvider } from '../contexts/AgentStateContext';

export interface AgentsWorkbenchProps {
  viewportMode?: WorkbenchViewportMode;
}

export function AgentsWorkbench({ viewportMode = 'embedded' }: AgentsWorkbenchProps) {
  return (
    <ThemeProvider>
      <AgentStateProvider>
        <WorkbenchLayout viewportMode={viewportMode} />
      </AgentStateProvider>
    </ThemeProvider>
  );
}
