import { ThemeProvider } from '@sdkwork/agents-pc-commons';

import { WorkbenchLayout, type WorkbenchViewportMode } from '../components/WorkbenchLayout';
import type { WorkbenchTab } from '../components/workbenchTabs';
import { AgentStateProvider } from '../contexts/AgentStateContext';
import './embedded.css';

export interface AgentsWorkbenchProps {
  /** Tabs hidden by the embedding host (e.g. surfaces with no backend yet). */
  hiddenTabs?: readonly WorkbenchTab[];
  overlayTopInset?: string;
  showSidebarLogo?: boolean;
  viewportMode?: WorkbenchViewportMode;
}

export function AgentsWorkbench({
  hiddenTabs,
  overlayTopInset = '0px',
  showSidebarLogo = true,
  viewportMode = 'embedded',
}: AgentsWorkbenchProps) {
  return (
    <ThemeProvider>
      <AgentStateProvider>
        <WorkbenchLayout
          hiddenTabs={hiddenTabs}
          overlayTopInset={overlayTopInset}
          showSidebarLogo={showSidebarLogo}
          viewportMode={viewportMode}
        />
      </AgentStateProvider>
    </ThemeProvider>
  );
}
