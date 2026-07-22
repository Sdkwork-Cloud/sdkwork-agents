import { ThemeProvider } from '@sdkwork/agents-pc-commons';

import { WorkbenchLayout, type WorkbenchViewportMode } from '../components/WorkbenchLayout';
import { AgentStateProvider } from '../contexts/AgentStateContext';
import './embedded.css';

export interface AgentsWorkbenchProps {
  overlayTopInset?: string;
  showSidebarLogo?: boolean;
  viewportMode?: WorkbenchViewportMode;
}

export function AgentsWorkbench({
  overlayTopInset = '0px',
  showSidebarLogo = true,
  viewportMode = 'embedded',
}: AgentsWorkbenchProps) {
  return (
    <ThemeProvider>
      <AgentStateProvider>
        <WorkbenchLayout
          overlayTopInset={overlayTopInset}
          showSidebarLogo={showSidebarLogo}
          viewportMode={viewportMode}
        />
      </AgentStateProvider>
    </ThemeProvider>
  );
}
