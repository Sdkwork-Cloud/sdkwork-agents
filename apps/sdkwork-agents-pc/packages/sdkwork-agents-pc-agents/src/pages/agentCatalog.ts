import type { AgentConfig } from '../services/AgentService';

export const AGENT_MARKET_CATEGORIES = [
  { id: 'all', label: '全部 Agent' },
  { id: 'tech', label: '技术开发' },
  { id: 'writing', label: '文案创作' },
  { id: 'design', label: 'UI/UX 设计' },
  { id: 'office', label: '效率办公' },
  { id: 'device', label: '硬件管理' },
] as const;

export type AgentMarketCategoryId = typeof AGENT_MARKET_CATEGORIES[number]['id'];

export function filterMarketAgents(
  agents: readonly AgentConfig[],
  categoryId: AgentMarketCategoryId,
): AgentConfig[] {
  if (categoryId === 'all') {
    return [...agents];
  }
  return agents.filter((agent) => agent.categoryId === categoryId);
}
