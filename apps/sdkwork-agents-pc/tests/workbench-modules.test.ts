import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';

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

test('exports the same complete workbench used by the standalone app', () => {
  const packageManifest = JSON.parse(
    readFileSync(new URL('../package.json', import.meta.url), 'utf8'),
  ) as { exports: Record<string, { import: string }> };
  const appSource = readFileSync(new URL('../src/App.tsx', import.meta.url), 'utf8');
  const workbenchSource = readFileSync(
    new URL('../src/workbench/AgentsWorkbench.tsx', import.meta.url),
    'utf8',
  );
  const runtimeSource = readFileSync(
    new URL('../src/workbench/runtime.ts', import.meta.url),
    'utf8',
  );

  assert.equal(packageManifest.exports['./workbench']?.import, './src/workbench/index.ts');
  assert.match(appSource, /<AgentsWorkbench viewportMode="fixed" \/>/);
  assert.match(workbenchSource, /<WorkbenchLayout viewportMode=\{viewportMode\} \/>/);
  assert.match(runtimeSource, /configureChatAgentPort/);
  assert.match(runtimeSource, /configureProjectPort/);
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
