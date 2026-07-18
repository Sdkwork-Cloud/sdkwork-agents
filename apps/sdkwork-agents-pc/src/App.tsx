import { AuthGate } from './AuthGate';
import { WorkbenchLayout } from './components/WorkbenchLayout';
import { ThemeProvider } from '@sdkwork/agents-pc-commons';
import { AgentStateProvider } from './contexts/AgentStateContext';

export default function App() {
  return (
    <ThemeProvider>
      <AgentStateProvider>
        <AuthGate>
          <WorkbenchLayout />
        </AuthGate>
      </AgentStateProvider>
    </ThemeProvider>
  );
}
