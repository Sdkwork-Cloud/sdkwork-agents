import { getSdkworkChatGlobalTokenManager } from '@sdkwork/agents-pc-core/session';
import { configureAgentsTokenPlanRuntime } from '@sdkwork/agents-pc-membership/runtime';
import {
  bootstrapSdkworkMembershipAppService,
  configureSdkworkMembershipSessionTokenProvider,
} from '@sdkwork/membership-service';
import {
  bootstrapSdkworkOrderAppService,
  configureSdkworkOrderSessionTokenProvider,
  createSdkworkCouponRechargeService,
  createSdkworkMembershipCheckoutService,
  createSdkworkPointsRechargeService,
} from '@sdkwork/order-service';
import { buildLoginRedirect } from '../authRouting';
import { resolveAgentsPcRuntimeConfig } from './runtimeConfig';

let prepared = false;

export function prepareStandaloneAgentsTokenPlanRuntime(): void {
  if (prepared) return;
  const config = resolveAgentsPcRuntimeConfig();
  const environment = import.meta.env as Record<string, unknown>;
  const membershipBaseUrl = readString(environment.VITE_SDKWORK_MEMBERSHIP_APP_API_BASE_URL)
    ?? config.appbaseAppApiBaseUrl;
  const orderBaseUrl = readString(environment.VITE_SDKWORK_ORDER_APP_API_BASE_URL)
    ?? config.appbaseAppApiBaseUrl;
  const tokenManager = getSdkworkChatGlobalTokenManager();

  bootstrapSdkworkMembershipAppService({ baseUrl: membershipBaseUrl, tokenManager });
  const orderService = bootstrapSdkworkOrderAppService({ baseUrl: orderBaseUrl, tokenManager });
  const readTokens = () => tokenManager.getTokens();
  configureSdkworkMembershipSessionTokenProvider(readTokens);
  configureSdkworkOrderSessionTokenProvider(readTokens);
  configureAgentsTokenPlanRuntime({
    checkoutService: createSdkworkMembershipCheckoutService({ appService: orderService }),
    couponRechargeService: createSdkworkCouponRechargeService({ appService: orderService }),
    onLoginRequired: () => {
      const { hash, pathname, search } = window.location;
      window.location.assign(buildLoginRedirect(pathname, search, hash));
    },
    pointsRechargeService: createSdkworkPointsRechargeService({ appService: orderService }),
  });
  prepared = true;
}

function readString(value: unknown): string | undefined {
  return typeof value === 'string' && value.trim() ? value.trim() : undefined;
}
