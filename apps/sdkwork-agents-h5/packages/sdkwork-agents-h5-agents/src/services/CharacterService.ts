/**
 * Character library backed by the shared agents domain service.
 *
 * A character is the same agent record the "My Agents" surface manages
 * (`agentService` injected by the host through `configureAgentService`);
 * the character views present a focused subset (name / description / avatar /
 * prompt / voice) of that record. The original IM H5 mock (localStorage)
 * has been fully replaced — data is now read from and written to the Agents
 * backend through the shared SDK client.
 */

import { agentService, type AgentConfig } from './AgentService';
import { createDefaultAvatar } from './DefaultAvatarService';

export interface Character {
  id: string;
  name: string;
  desc: string;
  avatar: string;
  prompt?: string;
  voice?: string;
}

/** Create input: avatar is optional and falls back to the default agent avatar. */
export type CharacterDraft = Omit<Character, 'id' | 'avatar'> & { avatar?: string };

function toCharacter(agent: AgentConfig): Character {
  return {
    id: agent.id ?? '',
    name: agent.name,
    desc: agent.description,
    avatar: agent.avatar || createDefaultAvatar('agent'),
    prompt: agent.systemPrompt,
    voice: agent.voiceIds?.[0],
  };
}

function toAgentConfig(character: CharacterDraft): AgentConfig {
  return {
    name: character.name,
    description: character.desc,
    avatar: character.avatar || undefined,
    type: 'normal',
    systemPrompt: character.prompt,
    voiceIds: character.voice ? [character.voice] : [],
  };
}

export const characterService = {
  async getCharacters(): Promise<Character[]> {
    const page = await agentService.listAgentsPage({
      scope: 'mine',
      includeDeleted: false,
    });
    return page.items.map(toCharacter);
  },

  async addCharacter(character: CharacterDraft): Promise<Character> {
    const saved = await agentService.createAgent(toAgentConfig(character));
    return toCharacter(saved);
  },

  async editCharacter(id: string, character: Partial<Character>): Promise<Character> {
    const saved = await agentService.updateAgent(id, {
      name: character.name,
      description: character.desc,
      avatar: character.avatar,
      systemPrompt: character.prompt,
      voiceIds: character.voice ? [character.voice] : [],
      type: 'normal',
    });
    return toCharacter(saved);
  },

  async deleteCharacter(id: string): Promise<void> {
    await agentService.deleteAgent(id);
  },
};
