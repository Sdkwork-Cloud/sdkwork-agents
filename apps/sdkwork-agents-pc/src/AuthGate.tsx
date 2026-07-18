import type { SdkworkAuthRuntimeConfig } from '@sdkwork/auth-pc-react';
import {
  SDKWORK_AGENTS_PC_SESSION_CHANGED_EVENT,
  readAppSdkSessionTokens,
} from '@sdkwork/agents-pc-core/session';
import { Bot, LoaderCircle, ShieldCheck } from 'lucide-react';
import { lazy, Suspense, type ReactNode, useEffect, useMemo, useState } from 'react';
import { BrowserRouter, useLocation, useNavigate } from 'react-router-dom';

import { AGENTS_AUTH_APPEARANCE } from './authAppearance';
import { getAgentsPcIamRuntime, hydrateAgentsPcIamSession } from './bootstrap/iamRuntime';
import { resolveAgentsPcRuntimeConfig } from './bootstrap/runtimeConfig';
import {
  AUTH_BASE_PATH,
  buildLoginRedirect,
  isAuthRoute,
  isSessionReady,
  readSafeRedirect,
} from './authRouting';

interface AuthGateProps {
  children: ReactNode;
}

const AUTH_RUNTIME_CONFIG: SdkworkAuthRuntimeConfig = {
  leftRailMode: 'qr-only',
  loginMethods: ['password'],
  oauthLoginEnabled: false,
  oauthProviders: [],
  qrLoginEnabled: true,
  recoveryMethods: [],
  registerMethods: ['email', 'phone'],
  verificationPolicy: {
    emailCodeLoginEnabled: false,
    emailRegistrationVerificationRequired: false,
    phoneCodeLoginEnabled: false,
    phoneRegistrationVerificationRequired: false,
  },
};

const SdkworkIamAuthRoutes = lazy(() =>
  import('@sdkwork/auth-pc-react').then((module) => ({
    default: module.SdkworkIamAuthRoutes,
  })),
);

export function AuthGate({ children }: AuthGateProps) {
  return (
    <BrowserRouter>
      <AuthGateContent>{children}</AuthGateContent>
    </BrowserRouter>
  );
}

function AuthGateContent({ children }: AuthGateProps) {
  const location = useLocation();
  const navigate = useNavigate();
  const [hydrated, setHydrated] = useState(false);
  const [authenticated, setAuthenticated] = useState(() => isSessionReady(readAppSdkSessionTokens()));
  const [compactAuthViewport, setCompactAuthViewport] = useState(() =>
    typeof window !== 'undefined' && window.matchMedia('(max-width: 767px)').matches,
  );
  const authRoute = isAuthRoute(location.pathname);

  useEffect(() => {
    let cancelled = false;
    void hydrateAgentsPcIamSession().finally(() => {
      if (!cancelled) {
        setAuthenticated(isSessionReady(readAppSdkSessionTokens()));
        setHydrated(true);
      }
    });
    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    const syncSession = () => setAuthenticated(isSessionReady(readAppSdkSessionTokens()));
    window.addEventListener(SDKWORK_AGENTS_PC_SESSION_CHANGED_EVENT, syncSession);
    return () => window.removeEventListener(SDKWORK_AGENTS_PC_SESSION_CHANGED_EVENT, syncSession);
  }, []);

  useEffect(() => {
    const mediaQuery = window.matchMedia('(max-width: 767px)');
    const syncViewport = () => setCompactAuthViewport(mediaQuery.matches);
    syncViewport();
    mediaQuery.addEventListener('change', syncViewport);
    return () => mediaQuery.removeEventListener('change', syncViewport);
  }, []);

  const authRuntimeConfig = useMemo<SdkworkAuthRuntimeConfig>(
    () => ({
      ...AUTH_RUNTIME_CONFIG,
      qrLoginEnabled: !compactAuthViewport,
    }),
    [compactAuthViewport],
  );

  const loginRedirect = useMemo(
    () => buildLoginRedirect(location.pathname, location.search, location.hash),
    [location.hash, location.pathname, location.search],
  );

  useEffect(() => {
    if (!hydrated) {
      return;
    }
    if (!authenticated && !authRoute) {
      navigate(loginRedirect, { replace: true });
      return;
    }
    if (authenticated && authRoute) {
      navigate(readSafeRedirect(location.search), { replace: true });
    }
  }, [authRoute, authenticated, hydrated, location.search, loginRedirect, navigate]);

  if (!hydrated || (!authenticated && !authRoute) || (authenticated && authRoute)) {
    return <AuthLoadingState />;
  }

  if (authRoute) {
    const config = resolveAgentsPcRuntimeConfig();
    return (
      <Suspense fallback={<AuthLoadingState />}>
        <SdkworkIamAuthRoutes
          appearance={AGENTS_AUTH_APPEARANCE}
          basePath={AUTH_BASE_PATH}
          className="agents-iam-auth-routes"
          getRuntime={getAgentsPcIamRuntime}
          homePath="/"
          locale={config.locale}
          runtimeConfig={authRuntimeConfig}
          viewportMode="fixed"
        />
      </Suspense>
    );
  }

  return <>{children}</>;
}

function AuthLoadingState() {
  return (
    <div
      aria-busy="true"
      aria-live="polite"
      className="agents-auth-loading flex h-[100dvh] w-full items-center justify-center overflow-hidden bg-[#09090b] px-6 text-zinc-100"
      role="status"
    >
      <div className="relative flex w-full max-w-sm flex-col items-center text-center">
        <div className="absolute -top-28 h-56 w-56 rounded-full bg-cyan-500/10 blur-3xl" />
        <div className="relative mb-6 flex h-16 w-16 items-center justify-center rounded-2xl border border-white/10 bg-white/[0.055] shadow-2xl shadow-cyan-950/50 backdrop-blur-xl">
          <Bot className="text-cyan-200" size={30} strokeWidth={1.8} />
          <span className="absolute -bottom-1 -right-1 flex h-6 w-6 items-center justify-center rounded-full border-2 border-[#09090b] bg-cyan-400 text-cyan-950">
            <ShieldCheck size={13} strokeWidth={2.5} />
          </span>
        </div>
        <h1 className="relative text-lg font-semibold tracking-tight text-zinc-100">SDKWork Agents</h1>
        <div className="relative mt-3 flex items-center gap-2 text-sm text-zinc-400">
          <LoaderCircle className="animate-spin text-cyan-300" size={16} />
          <span>正在验证安全会话…</span>
        </div>
        <p className="relative mt-3 max-w-xs text-xs leading-5 text-zinc-600">
          正在通过 IAM 确认身份与工作区访问权限
        </p>
      </div>
    </div>
  );
}
