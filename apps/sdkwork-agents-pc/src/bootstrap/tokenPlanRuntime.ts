import {
  configureAgentsTokenPlanRuntime,
  hasAgentsTokenPlanRuntime,
} from '@sdkwork/agents-pc-membership/runtime';
import { buildLoginRedirect } from '../authRouting';

let initialized = false;

export function initializeAgentsTokenPlanRuntime(): void {
  if (initialized || hasAgentsTokenPlanRuntime()) return;

  configureAgentsTokenPlanRuntime({
    onLoginRequired: () => {
      const { hash, pathname, search } = window.location;
      window.location.assign(buildLoginRedirect(pathname, search, hash));
    },
    prepare: async () => {
      const { prepareStandaloneAgentsTokenPlanRuntime } = await import('./tokenPlanSdk');
      prepareStandaloneAgentsTokenPlanRuntime();
    },
  });
  initialized = true;
}
