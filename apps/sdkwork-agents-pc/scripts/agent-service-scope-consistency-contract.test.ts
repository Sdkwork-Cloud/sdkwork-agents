import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import ts from 'typescript';

const agentServiceText = readFileSync(
  './packages/sdkwork-agents-pc-agents/src/services/AgentService.ts',
  'utf8',
);
const knowledgeAiServiceText = readFileSync(
  '../../../sdkwork-knowledgebase/apps/sdkwork-knowledgebase-pc/packages/sdkwork-knowledgebase-pc-knowledgebase/src/services/ai.ts',
  'utf8',
);
const sourceFile = ts.createSourceFile(
  'AgentService.ts',
  agentServiceText,
  ts.ScriptTarget.Latest,
  true,
  ts.ScriptKind.TS,
);

function findFunctionDeclaration(name: string): ts.FunctionDeclaration | undefined {
  let found: ts.FunctionDeclaration | undefined;
  const visit = (node: ts.Node): void => {
    if (ts.isFunctionDeclaration(node) && node.name?.text === name) {
      found = node;
      return;
    }
    ts.forEachChild(node, visit);
  };
  visit(sourceFile);
  return found;
}

function findClassDeclaration(name: string): ts.ClassDeclaration | undefined {
  let found: ts.ClassDeclaration | undefined;
  const visit = (node: ts.Node): void => {
    if (ts.isClassDeclaration(node) && node.name?.text === name) {
      found = node;
      return;
    }
    ts.forEachChild(node, visit);
  };
  visit(sourceFile);
  return found;
}

function methodBodyText(methodName: string): string {
  const classDeclaration = findClassDeclaration('SdkworkAgentService');
  assert.ok(classDeclaration, 'SdkworkAgentService class must exist');
  const method = classDeclaration.members.find(
    (member): member is ts.MethodDeclaration =>
      ts.isMethodDeclaration(member) &&
      ts.isIdentifier(member.name) &&
      member.name.text === methodName,
  );
  assert.ok(method?.body, `SdkworkAgentService.${methodName} must have a body`);
  return method.body.getText(sourceFile);
}

assert.equal(
  findFunctionDeclaration('readAgentScope'),
  undefined,
  'AgentService must not read tenant/organization/owner scope from frontend session; appbase request context owns scope',
);

for (const forbidden of [
  'readAppSdkSessionTokens',
  'resolveAppSdkTenantId',
  'resolveAppSdkOrganizationId',
  'resolveAppSdkUserId',
  'DEFAULT_AGENT_TENANT_ID',
  'DEFAULT_AGENT_ORGANIZATION_ID',
  'DEFAULT_AGENT_OWNER_USER_ID',
  'tenantId',
  'organizationId',
  'ownerUserId',
  'owner_user_id',
  'organization_id',
  'tenant_id',
]) {
  for (const [serviceName, serviceText] of [
    ['AgentService', agentServiceText],
    ['KnowledgeAiService', knowledgeAiServiceText],
  ] as const) {
    assert.doesNotMatch(
      serviceText,
      new RegExp(`\\b${forbidden}\\b`, 'u'),
      `${serviceName} must not contain client-controlled app-api scope value ${forbidden}`,
    );
  }
}

for (const methodName of [
  'listAgentsPage',
  'getAgent',
  'createAgent',
  'updateAgent',
  'publishAgent',
  'deleteAgent',
  'requestPreviewResponse',
  'optimizePrompt',
]) {
  const body = methodBodyText(methodName);
  assert.doesNotMatch(
    body,
    /readAppSdkSessionTokens\s*\(|readAgentScope\s*\(|tenantId|organizationId|ownerUserId/u,
    `SdkworkAgentService.${methodName} must not resolve or pass app-api scope; backend context derives it`,
  );
}

console.log('sdkwork agents pc agent service context-derived scope contract passed.');
