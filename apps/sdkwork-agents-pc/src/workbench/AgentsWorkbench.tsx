import { AgentsWorkbenchI18nProvider } from '@sdkwork/agents-pc-commons/i18n';
import { ThemeProvider } from '@sdkwork/agents-pc-commons';

import { WorkbenchLayout, type WorkbenchViewportMode } from '../components/WorkbenchLayout';
import { AgentStateProvider } from '../contexts/AgentStateContext';
import './embedded.css';

export interface AgentsWorkbenchProps {
  showSidebarLogo?: boolean;
  viewportMode?: WorkbenchViewportMode;
}

export function AgentsWorkbench({
  showSidebarLogo = true,
  viewportMode = 'embedded',
}: AgentsWorkbenchProps) {
  return (
    <AgentsWorkbenchI18nProvider>
      <ThemeProvider>
        <AgentStateProvider>
          <WorkbenchLayout
            showSidebarLogo={showSidebarLogo}
            viewportMode={viewportMode}
          />
        </AgentStateProvider>
      </ThemeProvider>
    </AgentsWorkbenchI18nProvider>
  );
}
