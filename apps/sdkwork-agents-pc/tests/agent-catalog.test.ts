import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';

import {
  AGENT_MARKET_CATEGORIES,
  filterMarketAgents,
} from '../packages/sdkwork-agents-pc-agents/src/pages/agentCatalog';

const agents = [
  { name: 'Developer', description: '', type: 'normal' as const, categoryId: 'tech' },
  { name: 'Writer', description: '', type: 'normal' as const, categoryId: 'writing' },
  { name: 'Uncategorized', description: '', type: 'normal' as const },
];

test('preserves the complete legacy Agent market category set', () => {
  assert.deepEqual(
    AGENT_MARKET_CATEGORIES.map(({ id }) => id),
    ['all', 'tech', 'writing', 'design', 'office', 'device'],
  );
});

test('filters only matching market agents while all preserves the page', () => {
  assert.deepEqual(filterMarketAgents(agents, 'all').map(({ name }) => name), [
    'Developer',
    'Writer',
    'Uncategorized',
  ]);
  assert.deepEqual(filterMarketAgents(agents, 'tech').map(({ name }) => name), ['Developer']);
  assert.deepEqual(filterMarketAgents(agents, 'device'), []);
});

test('Agent workspace renders and applies the market category controls', () => {
  const source = readFileSync(
    new URL('../packages/sdkwork-agents-pc-agents/src/pages/AgentsHomePage.tsx', import.meta.url),
    'utf8',
  );
  const publicExport = readFileSync(
    new URL('../packages/sdkwork-agents-pc-agents/src/index.ts', import.meta.url),
    'utf8',
  );
  assert.match(source, /AGENT_MARKET_CATEGORIES\.map/);
  assert.match(source, /filterMarketAgents\(catalog\.items, marketCategory\)/);
  assert.match(source, /scope === 'market'/);
  assert.match(publicExport, /export \{ AgentsHomePage \}/);
  assert.match(publicExport, /configureAgentsHomeRuntime/);
});
