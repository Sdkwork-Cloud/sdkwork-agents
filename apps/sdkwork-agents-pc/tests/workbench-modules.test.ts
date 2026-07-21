import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import globalI18n from 'i18next';

import agentsWorkbenchI18n from '../packages/sdkwork-agents-pc-commons/src/i18n';

import {
  DEFAULT_WORKBENCH_TAB,
  isWorkbenchTab,
  SIDEBAR_TABS,
  WORKBENCH_TABS,
} from '../src/components/workbenchTabs';

test('registers Chat first and places Agent immediately after Canvas', () => {
  assert.deepEqual(WORKBENCH_TABS, [
    'chat_session',
    'inspiration',
    'creative',
    'assets',
    'presentation',
    'canvas',
    'agents',
  ]);
});

test('opens Chat by default', () => {
  assert.equal(DEFAULT_WORKBENCH_TAB, 'chat_session');
});

test('keeps every registered workbench module reachable from the sidebar', () => {
  assert.equal(isWorkbenchTab('presentation'), true);
  assert.deepEqual(SIDEBAR_TABS, WORKBENCH_TABS);
});

test('opens the Agents-owned Token Plan from the membership sidebar action', () => {
  const layoutSource = readFileSync(
    new URL('../src/components/WorkbenchLayout.tsx', import.meta.url),
    'utf8',
  );
  const sidebarSource = readFileSync(
    new URL('../src/components/GlobalSidebar.tsx', import.meta.url),
    'utf8',
  );
  const tokenPlanSource = readFileSync(
    new URL(
      '../packages/sdkwork-agents-pc-membership/src/AgentsTokenPlanView.tsx',
      import.meta.url,
    ),
    'utf8',
  );

  assert.match(layoutSource, /import\('@sdkwork\/agents-pc-membership'\)/);
  assert.match(layoutSource, /isTokenPlanOpen \? <AgentsTokenPlanView \/> : <ActiveWorkspace \/>/);
  assert.match(layoutSource, /setIsTokenPlanOpen\(false\);\s*setActiveTab\(tab\);/);
  assert.match(sidebarSource, /onClick=\{onOpenTokenPlan\}/);
  assert.match(sidebarSource, /aria-current=\{isTokenPlanOpen \? 'page' : undefined\}/);
  assert.match(tokenPlanSource, /SdkworkSubscriptionCatalogPage/);
  assert.match(tokenPlanSource, /checkoutPort=\{runtime\.checkoutService\}/);
});

test('Token Plan uses public SDKWork services and the shared global TokenManager', () => {
  const bootstrapSource = readFileSync(
    new URL('../src/bootstrap/tokenPlanRuntime.ts', import.meta.url),
    'utf8',
  );
  const sdkBootstrapSource = readFileSync(
    new URL('../src/bootstrap/tokenPlanSdk.ts', import.meta.url),
    'utf8',
  );
  const membershipPackageRoot = new URL(
    '../packages/sdkwork-agents-pc-membership/src/',
    import.meta.url,
  );
  const featureSource = [
    'AgentsTokenPlanView.tsx',
    'TokenPlanModals.tsx',
    'memberSummary.ts',
    'runtime.ts',
  ].map((file) => readFileSync(new URL(file, membershipPackageRoot), 'utf8')).join('\n');

  assert.match(bootstrapSource, /import\('\.\/tokenPlanSdk'\)/);
  assert.match(sdkBootstrapSource, /getSdkworkChatGlobalTokenManager/);
  assert.match(sdkBootstrapSource, /bootstrapSdkworkMembershipAppService/);
  assert.match(sdkBootstrapSource, /bootstrapSdkworkOrderAppService/);
  assert.match(sdkBootstrapSource, /configureSdkworkOrderSessionTokenProvider\(readTokens\)/);
  assert.match(featureSource, /@sdkwork\/membership-pc-subscription\/catalog/);
  assert.match(featureSource, /@sdkwork\/order-pc-checkout/);
  assert.match(featureSource, /@sdkwork\/order-pc-recharge/);
  assert.doesNotMatch(featureSource, /\bfetch\s*\(/);
  assert.doesNotMatch(featureSource, /axios/);
  assert.doesNotMatch(featureSource, /Authorization|Access-Token/);
});

test('exports the same complete workbench used by the standalone app', () => {
  const packageManifest = JSON.parse(
    readFileSync(new URL('../package.json', import.meta.url), 'utf8'),
  ) as { exports: Record<string, { import: string }> };
  const appSource = readFileSync(new URL('../src/App.tsx', import.meta.url), 'utf8');
  const workbenchSource = readFileSync(
    new URL('../src/workbench/AgentsWorkbench.tsx', import.meta.url),
    'utf8',
  );
  const layoutSource = readFileSync(
    new URL('../src/components/WorkbenchLayout.tsx', import.meta.url),
    'utf8',
  );
  const sidebarSource = readFileSync(
    new URL('../src/components/GlobalSidebar.tsx', import.meta.url),
    'utf8',
  );
  const runtimeSource = readFileSync(
    new URL('../src/workbench/runtime.ts', import.meta.url),
    'utf8',
  );
  const portsSource = readFileSync(
    new URL('../src/workbench/ports.ts', import.meta.url),
    'utf8',
  );

  assert.equal(packageManifest.exports['./workbench']?.import, './src/workbench/index.ts');
  assert.match(appSource, /<AgentsWorkbench viewportMode="fixed" \/>/);
  assert.match(workbenchSource, /import '\.\/embedded\.css';/);
  assert.match(workbenchSource, /<AgentsWorkbenchI18nProvider>/);
  assert.match(workbenchSource, /showSidebarLogo = true/);
  assert.match(workbenchSource, /showSidebarLogo=\{showSidebarLogo\}/);
  assert.match(workbenchSource, /viewportMode=\{viewportMode\}/);
  assert.match(layoutSource, /sdkwork-agents-workbench/);
  assert.match(layoutSource, /showSidebarLogo=\{showSidebarLogo\}/);
  assert.match(sidebarSource, /\{showSidebarLogo && \(/);
  assert.match(runtimeSource, /configureAgentsWorkbenchPorts/);
  assert.match(portsSource, /configureChatAgentPort/);
  assert.match(portsSource, /configureProjectPort/);
});

test('keeps Agents translations isolated from a composing host', () => {
  assert.notEqual(agentsWorkbenchI18n, globalI18n);
  assert.equal(agentsWorkbenchI18n.t('newChat', { ns: 'chat' }), 'New Chat');
  assert.equal(
    agentsWorkbenchI18n.t('welcomeDescription', { ns: 'chat' }),
    'I can write code, answer questions, analyze images, and help you explore any topic.',
  );
});

test('sizes the empty Chat state from the workbench container', () => {
  const embeddedStyles = readFileSync(
    new URL('../src/workbench/embedded.css', import.meta.url),
    'utf8',
  );
  const messageListSource = readFileSync(
    new URL(
      '../packages/sdkwork-agents-pc-chat/src/components/MessageList.tsx',
      import.meta.url,
    ),
    'utf8',
  );

  assert.match(embeddedStyles, /container-type: size/);
  assert.match(messageListSource, /100cqh-200px/);
  assert.doesNotMatch(messageListSource, /100vh-200px/);
});

test('keeps Presentation reachable without rendering a sidebar icon', () => {
  const sidebarSource = readFileSync(
    new URL('../src/components/GlobalSidebar.tsx', import.meta.url),
    'utf8',
  );

  assert.match(sidebarSource, /presentation: \{ label: '演示' \}/);
  assert.doesNotMatch(sidebarSource, /presentation: \{ icon:/);
});

test('resolves packaged Assets media without depending on the host source root', () => {
  const assetsServiceSource = readFileSync(
    new URL(
      '../packages/sdkwork-agents-pc-assets/src/services/AssetsService.ts',
      import.meta.url,
    ),
    'utf8',
  );

  assert.match(assetsServiceSource, /new URL\(/);
  assert.match(assetsServiceSource, /import\.meta\.url/);
  assert.doesNotMatch(assetsServiceSource, /['"]\/src\/assets\//);
});

test('accepts only registered workbench tab events', () => {
  for (const tab of WORKBENCH_TABS) {
    assert.equal(isWorkbenchTab(tab), true);
  }

  assert.equal(isWorkbenchTab('settings'), false);
  assert.equal(isWorkbenchTab(undefined), false);
});

test('keeps Reveal history disabled inside the embedded srcDoc preview', () => {
  const presentationHtmlSource = readFileSync(
    new URL(
      '../packages/sdkwork-agents-pc-presentation/src/utils/pptUtils.ts',
      import.meta.url,
    ),
    'utf8',
  );

  assert.match(
    presentationHtmlSource,
    /hash: window\.location\.protocol !== 'about:'/,
  );
});
