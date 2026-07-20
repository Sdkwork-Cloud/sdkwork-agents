import { AuthGate } from './AuthGate';
import { AgentsWorkbench } from './workbench';

export default function App() {
  return (
    <AuthGate>
      <AgentsWorkbench viewportMode="fixed" />
    </AuthGate>
  );
}
