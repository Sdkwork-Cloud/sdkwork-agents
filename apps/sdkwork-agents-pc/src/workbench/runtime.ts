import {
  configureAgentsHomeRuntime,
  type AgentsHomeRuntime,
} from '@sdkwork/agents-pc-agents/home';
import {
  configureCommunityAppSdkClientProvider,
  type SdkworkCommunityAppClient,
} from '@sdkwork/agents-pc-core/sdk/communityAppSdkClient';
import {
  configureGenerationsAppSdkClientProvider,
  type SdkworkGenerationsAppClient,
} from '@sdkwork/agents-pc-core/sdk/generationsAppSdkClient';
import {
  configureMemoryAppSdkClientProvider,
  type SdkworkAgentsMemoryAppClient,
} from '@sdkwork/agents-pc-core/sdk/memoryAppSdkClient';
import {
  configurePromptsAppSdkClientProvider,
  type SdkworkAgentsPromptsAppClient,
} from '@sdkwork/agents-pc-core/sdk/promptsAppSdkClient';
import {
  configureFeedsOpenSdkClientProvider,
  type SdkworkFeedsClient,
} from '@sdkwork/agents-pc-core/sdk/feedsOpenSdkClient';
import {
  configureSkillsAppSdkClientProvider,
  type SdkworkSkillsAppClient,
} from '@sdkwork/agents-pc-core/sdk/skillsAppSdkClient';
import {
  configureAgentsTokenPlanRuntime,
  type AgentsTokenPlanRuntime,
} from '@sdkwork/agents-pc-membership/runtime';
import { configureAgentsWorkbenchPorts } from './ports';

export { configureAgentsWorkbenchPorts } from './ports';

export interface AgentsWorkbenchRuntime extends AgentsHomeRuntime {
  getMemoryAppSdkClient: () => SdkworkAgentsMemoryAppClient;
  getPromptsAppSdkClient: () => SdkworkAgentsPromptsAppClient;
  /** Optional host-injected community SDK client (portal/gateway sessions). */
  getCommunityAppSdkClient?: () => SdkworkCommunityAppClient;
  /** Optional host-injected generations SDK client (portal/gateway sessions). */
  getGenerationsAppSdkClient?: () => SdkworkGenerationsAppClient;
  /** Optional host-injected feeds open SDK client (portal api-assembly gateway). */
  getFeedsOpenSdkClient?: () => SdkworkFeedsClient;
  /** Optional host-injected skills app SDK client (portal/gateway sessions). */
  getSkillsAppSdkClient?: () => SdkworkSkillsAppClient;
  tokenPlan?: AgentsTokenPlanRuntime;
}

export function configureAgentsWorkbenchRuntime(runtime: AgentsWorkbenchRuntime): void {
  configureAgentsHomeRuntime(runtime);
  configureMemoryAppSdkClientProvider(runtime.getMemoryAppSdkClient);
  configurePromptsAppSdkClientProvider(runtime.getPromptsAppSdkClient);
  if (runtime.getGenerationsAppSdkClient) {
    configureGenerationsAppSdkClientProvider(runtime.getGenerationsAppSdkClient);
  }
  if (runtime.getCommunityAppSdkClient) {
    configureCommunityAppSdkClientProvider(runtime.getCommunityAppSdkClient);
  }
  if (runtime.getFeedsOpenSdkClient) {
    configureFeedsOpenSdkClientProvider(runtime.getFeedsOpenSdkClient);
  }
  if (runtime.getSkillsAppSdkClient) {
    configureSkillsAppSdkClientProvider(runtime.getSkillsAppSdkClient);
  }
  configureAgentsTokenPlanRuntime(runtime.tokenPlan ?? null);
  configureAgentsWorkbenchPorts();
}
