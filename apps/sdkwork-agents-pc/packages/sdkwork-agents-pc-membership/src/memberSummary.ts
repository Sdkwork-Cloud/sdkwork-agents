import { useEffect, useMemo, useState } from 'react';
import { hasSdkworkMembershipSession } from '@sdkwork/membership-service';
import {
  useSdkworkMembershipController,
  useSdkworkMembershipControllerState,
  type SdkworkMembershipSummary,
} from '@sdkwork/membership-pc-membership';

export function resolveAgentsMembershipTierKey(summary: SdkworkMembershipSummary): string {
  if (!summary.isAuthenticated || summary.status === 'guest' || !summary.isMember) {
    return 'none';
  }
  return summary.currentLevelValue !== null && summary.currentLevelValue >= 2 ? 'peak' : 'pro';
}

export function useAgentsTokenPlanMemberSummary() {
  const controller = useSdkworkMembershipController();
  const state = useSdkworkMembershipControllerState(controller);
  const [tierOverride, setTierOverride] = useState<string | null>(null);

  useEffect(() => {
    if (
      hasSdkworkMembershipSession()
      && !state.isBootstrapped
      && !state.isLoading
      && !state.lastError
    ) {
      void controller.bootstrap().catch(() => undefined);
    }
  }, [controller, state.isBootstrapped, state.isLoading, state.lastError]);

  useEffect(() => {
    const refreshOnFocus = () => {
      if (hasSdkworkMembershipSession()) {
        void controller.refresh().catch(() => undefined);
      }
    };
    window.addEventListener('focus', refreshOnFocus);
    return () => window.removeEventListener('focus', refreshOnFocus);
  }, [controller]);

  const memberSummary = useMemo(() => {
    if (!hasSdkworkMembershipSession()) {
      return null;
    }
    return {
      membershipTierKey: tierOverride ?? resolveAgentsMembershipTierKey(state.dashboard.summary),
      pointBalance: state.dashboard.summary.pointBalance,
    };
  }, [state.dashboard.summary, tierOverride]);

  return {
    memberSummary,
    refreshMembership: () => controller.refresh(),
    setMembershipTierKey: setTierOverride,
  };
}
