import assert from 'node:assert/strict';
import test from 'node:test';

import {
  DEFAULT_WORKBENCH_TAB,
  isWorkbenchTab,
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

test('accepts only registered workbench tab events', () => {
  for (const tab of WORKBENCH_TABS) {
    assert.equal(isWorkbenchTab(tab), true);
  }

  assert.equal(isWorkbenchTab('settings'), false);
  assert.equal(isWorkbenchTab(undefined), false);
});
