import type {
  SdkworkCouponRechargeService,
  SdkworkMembershipCheckoutService,
  SdkworkPointsRechargeService,
} from '@sdkwork/order-service';

export interface AgentsTokenPlanRuntime {
  checkoutService?: SdkworkMembershipCheckoutService;
  couponRechargeService?: SdkworkCouponRechargeService;
  onLoginRequired?: () => void;
  pointsRechargeService?: SdkworkPointsRechargeService;
  prepare?: () => Promise<void> | void;
}

const TOKEN_PLAN_RUNTIME_KEY = Symbol.for('sdkwork.agents.tokenPlanRuntime');

type GlobalTokenPlanRuntime = typeof globalThis & {
  [TOKEN_PLAN_RUNTIME_KEY]?: AgentsTokenPlanRuntime;
};

export function configureAgentsTokenPlanRuntime(runtime: AgentsTokenPlanRuntime | null): void {
  const globalRegistry = globalThis as GlobalTokenPlanRuntime;
  if (runtime) {
    globalRegistry[TOKEN_PLAN_RUNTIME_KEY] = runtime;
  } else {
    delete globalRegistry[TOKEN_PLAN_RUNTIME_KEY];
  }
}

export function getAgentsTokenPlanRuntime(): AgentsTokenPlanRuntime {
  return (globalThis as GlobalTokenPlanRuntime)[TOKEN_PLAN_RUNTIME_KEY] ?? {};
}

export function hasAgentsTokenPlanRuntime(): boolean {
  return Boolean((globalThis as GlobalTokenPlanRuntime)[TOKEN_PLAN_RUNTIME_KEY]);
}
