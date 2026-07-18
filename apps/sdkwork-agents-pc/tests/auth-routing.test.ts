import assert from 'node:assert/strict';
import test from 'node:test';

import {
  buildLoginRedirect,
  isAuthRoute,
  isSessionReady,
  readSafeRedirect,
} from '../src/authRouting';

test('builds a same-app login redirect with path, query and hash', () => {
  assert.equal(
    buildLoginRedirect('/workspace', '?tab=agents', '#mine'),
    '/auth/login?redirect=%2Fworkspace%3Ftab%3Dagents%23mine',
  );
});

test('accepts only local non-auth return paths', () => {
  assert.equal(readSafeRedirect('?redirect=%2Fworkspace%3Ftab%3Dagents'), '/workspace?tab=agents');
  assert.equal(readSafeRedirect('?redirect=https%3A%2F%2Fevil.example'), '/');
  assert.equal(readSafeRedirect('?redirect=%2F%2Fevil.example'), '/');
  assert.equal(readSafeRedirect('?redirect=%2Fauth%2Flogin'), '/');
  assert.equal(readSafeRedirect('?redirect=%E0%A4%A'), '/');
});

test('recognizes the complete IAM route namespace', () => {
  assert.equal(isAuthRoute('/auth'), true);
  assert.equal(isAuthRoute('/auth/login'), true);
  assert.equal(isAuthRoute('/auth/oauth/callback/github'), true);
  assert.equal(isAuthRoute('/agents'), false);
});

test('requires dual tokens and complete standard AppContext', () => {
  const completeSession = {
    accessToken: 'access-token',
    authToken: 'auth-token',
    context: {
      appId: 'sdkwork-agents',
      authLevel: 'password' as const,
      dataScope: [],
      deploymentMode: 'local' as const,
      environment: 'dev' as const,
      permissionScope: [],
      sessionId: 'session-1',
      tenantId: '100001',
      userId: 'user-1',
    },
  };

  assert.equal(isSessionReady(completeSession), true);
  assert.equal(isSessionReady({ ...completeSession, accessToken: undefined }), false);
  assert.equal(isSessionReady({ ...completeSession, context: undefined }), false);
  assert.equal(isSessionReady({
    ...completeSession,
    context: { ...completeSession.context, userId: '' },
  }), false);
  assert.equal(isSessionReady({ ...completeSession, expiresAt: Date.now() - 1 }), false);
});
