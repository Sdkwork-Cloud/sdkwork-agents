import type { SdkworkIamRuntimeAuthRuntimeLike } from '@sdkwork/auth-pc-react';
import {
  createSdkworkAppbasePcAuthRuntime,
  type SdkworkAppbasePcAuthRuntimeComposition,
  type SdkworkAppbasePcAuthRuntimeSdkClient,
} from '@sdkwork/auth-runtime-pc-react';
import { prepareCredentialEntryTokens } from '@sdkwork/iam-credential-entry';
import type { IamAppContext } from '@sdkwork/iam-contracts';
import {
  clearAppSdkSessionTokens,
  getSdkworkChatGlobalTokenManager,
  persistAppSdkSessionTokens,
  readAppSdkSessionTokens,
  type SdkworkChatSession,
} from '@sdkwork/agents-pc-core/session';

import { resolveAgentsPcRuntimeConfig } from './runtimeConfig';

declare const __SDKWORK_CREDENTIAL_ENTRY_BOOTSTRAP_ACCESS_TOKEN__: string;

let composition: SdkworkAppbasePcAuthRuntimeComposition | null = null;

interface AgentsPcIamSession {
  accessToken?: string;
  authToken?: string;
  context?: IamAppContext;
  expiresAt?: number;
  refreshToken?: string;
  sessionId?: string;
  user?: SdkworkChatSession['user'];
}

export function initializeAgentsPcIamRuntime(
  sdkClients: readonly SdkworkAppbasePcAuthRuntimeSdkClient[],
): SdkworkAppbasePcAuthRuntimeComposition {
  if (composition) {
    return composition;
  }

  const config = resolveAgentsPcRuntimeConfig();
  const tokenManager = getSdkworkChatGlobalTokenManager();
  composition = createSdkworkAppbasePcAuthRuntime({
    app: {
      appId: config.appId,
      deploymentMode: config.deploymentMode,
      environment: config.environment,
      platform: 'pc',
    },
    baseUrls: {
      appbaseAppApiBaseUrl: config.appbaseAppApiBaseUrl,
    },
    credentialEntry: {
      prepareTokens: () => prepareCredentialEntryTokens(
        tokenManager,
        () => __SDKWORK_CREDENTIAL_ENTRY_BOOTSTRAP_ACCESS_TOKEN__ || undefined,
      ),
    },
    localeProvider: () => config.locale,
    sdkClients,
    sessionBridge: {
      clearSession: clearAppSdkSessionTokens,
      commitSession: (session) => commitIamSession(session as AgentsPcIamSession),
      readSession: () => toIamSession(readAppSdkSessionTokens()),
    },
    tokenManager,
  });

  return composition;
}

export function getAgentsPcIamRuntime(): SdkworkIamRuntimeAuthRuntimeLike {
  if (!composition) {
    throw new Error('Agents PC IAM runtime has not been initialized.');
  }
  return composition.runtime as SdkworkIamRuntimeAuthRuntimeLike;
}

export async function hydrateAgentsPcIamSession(): Promise<void> {
  if (!composition) {
    throw new Error('Agents PC IAM runtime has not been initialized.');
  }
  await composition.runtime.hydrateTokenManager();
  const session = readAppSdkSessionTokens();
  if (!session?.authToken || !session.accessToken) {
    clearAppSdkSessionTokens();
    return;
  }

  try {
    await composition.runtime.service.auth.sessions.current.retrieve();
  } catch {
    clearAppSdkSessionTokens();
  }
}

export function resetAgentsPcIamRuntimeForTests(): void {
  composition = null;
}

function commitIamSession(session: AgentsPcIamSession): AgentsPcIamSession {
  const committed = persistAppSdkSessionTokens({
    ...(session.accessToken ? { accessToken: session.accessToken } : {}),
    ...(session.authToken ? { authToken: session.authToken } : {}),
    ...(session.context ? { context: session.context } : {}),
    ...(session.expiresAt ? { expiresAt: session.expiresAt } : {}),
    ...(session.refreshToken ? { refreshToken: session.refreshToken } : {}),
    ...(session.sessionId ? { sessionId: session.sessionId } : {}),
    ...(session.user ? { user: session.user } : {}),
  });
  return toIamSession(committed) ?? {};
}

function toIamSession(session: SdkworkChatSession | null): AgentsPcIamSession | null {
  if (!session) {
    return null;
  }
  const context = toIamAppContext(session);
  return {
    ...(session.accessToken ? { accessToken: session.accessToken } : {}),
    ...(session.authToken ? { authToken: session.authToken } : {}),
    ...(context ? { context } : {}),
    ...(session.expiresAt ? { expiresAt: session.expiresAt } : {}),
    ...(session.refreshToken ? { refreshToken: session.refreshToken } : {}),
    ...(session.sessionId ? { sessionId: session.sessionId } : {}),
    ...(session.user ? { user: session.user } : {}),
  };
}

function toIamAppContext(session: SdkworkChatSession): IamAppContext | undefined {
  const context = session.context;
  if (
    !context?.appId
    || !context.authLevel
    || !context.deploymentMode
    || !context.environment
    || !context.tenantId
    || !context.userId
    || !(context.sessionId || session.sessionId)
  ) {
    return undefined;
  }
  return {
    appId: context.appId,
    authLevel: context.authLevel,
    dataScope: [...(context.dataScope ?? [])],
    deploymentMode: context.deploymentMode,
    environment: context.environment,
    ...(context.organizationId ? { organizationId: context.organizationId } : {}),
    permissionScope: [...(context.permissionScope ?? [])],
    sessionId: context.sessionId ?? session.sessionId ?? '',
    tenantId: context.tenantId,
    userId: context.userId,
  };
}
