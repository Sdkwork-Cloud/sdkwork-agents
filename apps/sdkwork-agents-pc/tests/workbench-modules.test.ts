import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import {
  agentsWorkbenchChatCatalog,
  agentsWorkbenchI18nCatalogs,
} from '../packages/sdkwork-agents-pc-commons/src/i18n';

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
    'canvas',
    'agents',
  ]);
});

test('opens Chat by default', () => {
  assert.equal(DEFAULT_WORKBENCH_TAB, 'chat_session');
});

test('keeps every registered workbench module reachable from the sidebar', () => {
  assert.equal(isWorkbenchTab('presentation'), false);
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

test('keeps the sidebar footer free of unused actions and membership counts', () => {
  const sidebarSource = readFileSync(
    new URL('../src/components/GlobalSidebar.tsx', import.meta.url),
    'utf8',
  );

  assert.match(sidebarSource, /<Gem /);
  assert.doesNotMatch(sidebarSource, />60<\/span>/);
  assert.doesNotMatch(sidebarSource, /\bBell\b/);
  assert.doesNotMatch(sidebarSource, /\bTerminal\b/);
  assert.doesNotMatch(sidebarSource, /\bSlidersHorizontal\b/);
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
  assert.equal(packageManifest.exports['./workbench/i18n']?.import, './src/workbench/i18n.ts');
  assert.match(appSource, /<AgentsWorkbench viewportMode="fixed" \/>/);
  assert.match(workbenchSource, /import '\.\/embedded\.css';/);
  assert.doesNotMatch(workbenchSource, /AgentsWorkbenchI18nProvider/);
  assert.match(workbenchSource, /overlayTopInset = '0px'/);
  assert.match(workbenchSource, /overlayTopInset=\{overlayTopInset\}/);
  assert.match(workbenchSource, /showSidebarLogo = true/);
  assert.match(workbenchSource, /showSidebarLogo=\{showSidebarLogo\}/);
  assert.match(workbenchSource, /viewportMode=\{viewportMode\}/);
  assert.match(layoutSource, /sdkwork-agents-workbench/);
  assert.match(layoutSource, /--sdkwork-agents-overlay-top-inset/);
  assert.match(layoutSource, /showSidebarLogo=\{showSidebarLogo\}/);
  assert.match(sidebarSource, /\{showSidebarLogo && \(/);
  assert.match(runtimeSource, /configureAgentsWorkbenchPorts/);
  assert.match(portsSource, /configureChatAgentPort/);
  assert.match(portsSource, /configureProjectPort/);
});

test('exports Agents catalogs without registering a global i18next instance', () => {
  const i18nSource = readFileSync(
    new URL('../packages/sdkwork-agents-pc-commons/src/i18n/index.ts', import.meta.url),
    'utf8',
  );

  assert.deepEqual(
    agentsWorkbenchI18nCatalogs.map((catalog) => catalog.namespace),
    ['chat', 'settings', 'common'],
  );
  assert.equal(agentsWorkbenchChatCatalog.resolveMessages('en-US').newChat, 'New Chat');
  assert.equal(
    agentsWorkbenchChatCatalog.resolveMessages('en-US').welcomeDescription,
    'I can write code, answer questions, analyze images, and help you explore any topic.',
  );
  assert.doesNotMatch(i18nSource, /initReactI18next|createInstance|I18nextProvider/);
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

test('keeps full-screen workbench overlays below a configured host header', () => {
  const embeddedStyles = readFileSync(
    new URL('../src/workbench/embedded.css', import.meta.url),
    'utf8',
  );
  const previewSources = [
    '../packages/sdkwork-agents-pc-assets/src/components/AssetDetailModal.tsx',
    '../packages/sdkwork-agents-pc-inspiration/src/components/ImageDetailModal.tsx',
    '../packages/sdkwork-agents-pc-inspiration/src/components/VideoDetailModal.tsx',
    '../packages/sdkwork-agents-pc-commons/src/components/creative/ImageLightbox.tsx',
  ].map((path) => readFileSync(new URL(path, import.meta.url), 'utf8'));

  assert.match(embeddedStyles, /\.sdkwork-agents-workbench \.fixed\.inset-0/);
  assert.match(
    embeddedStyles,
    /top: var\(--sdkwork-agents-overlay-top-inset, 0px\)/,
  );
  for (const previewSource of previewSources) {
    assert.match(previewSource, /fixed inset-0/);
  }
});

test('removes Presentation from the workbench application composition', () => {
  const sidebarSource = readFileSync(
    new URL('../src/components/GlobalSidebar.tsx', import.meta.url),
    'utf8',
  );
  const layoutSource = readFileSync(
    new URL('../src/components/WorkbenchLayout.tsx', import.meta.url),
    'utf8',
  );
  const packageManifest = JSON.parse(
    readFileSync(new URL('../package.json', import.meta.url), 'utf8'),
  );
  const componentSpec = JSON.parse(
    readFileSync(new URL('../specs/component.spec.json', import.meta.url), 'utf8'),
  );
  const moduleRegistrySource = readFileSync(
    new URL(
      '../packages/sdkwork-agents-pc-core/src/composition/module-registry.ts',
      import.meta.url,
    ),
    'utf8',
  );

  assert.doesNotMatch(sidebarSource, /presentation/);
  assert.doesNotMatch(layoutSource, /agents-pc-presentation|PresentationView/);
  assert.equal(packageManifest.dependencies['@sdkwork/agents-pc-presentation'], undefined);
  assert.equal(componentSpec.contracts.requiredPorts.includes('agents.presentation.view'), false);
  assert.doesNotMatch(moduleRegistrySource, /agents-pc-presentation|presentation:/);
});

test('loads Assets through the composed Drive SDK service', () => {
  const assetsServiceSource = readFileSync(
    new URL(
      '../packages/sdkwork-agents-pc-assets/src/services/AssetsService.ts',
      import.meta.url,
    ),
    'utf8',
  );

  assert.match(assetsServiceSource, /getDriveAppSdkClientWithSession/);
  assert.match(assetsServiceSource, /drive\.assets\.list/);
  assert.match(assetsServiceSource, /agentsDriveUploadService\.resolvePreviewUrl/);
  assert.doesNotMatch(assetsServiceSource, /new URL\(|import\.meta\.url/);
  assert.doesNotMatch(assetsServiceSource, /MOCK_|picsum|unsplash|mixkit/i);
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
