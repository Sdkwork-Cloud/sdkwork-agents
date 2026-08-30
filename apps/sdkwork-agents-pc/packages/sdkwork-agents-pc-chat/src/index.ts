export {
  ChatService,
  callerScopeGrantsAgentManage,
  configureChatAgentPermissionScopeReader,
  configureChatAgentPort,
  createChatAgentScope,
  DEFAULT_CHAT_AGENT_ID,
  DEFAULT_CHAT_AGENT_SCOPE,
  isDefaultChatAgentScope,
} from './services/ChatService';
export type {
  ChatAgentConfig,
  ChatAgentPermissionScopeReader,
  ChatAgentPort,
  ChatAgentScope,
  ChatServiceOptions,
} from './services/ChatService';
export { ProjectService, configureProjectPort } from './services/ProjectService';
export type {
  ChatMemorySpaceOption,
  ChatProject,
  ChatProjectCompositionSlot,
  ProjectDetails,
  ProjectPort,
  ProjectSettingsData,
} from './services/ProjectService';
export type { ChatMessage, ChatSession, ChatToolCall, ChatToolStreamEvent, MessageRole } from './types';
export type { ChatViewProps } from './ChatView';
export type { ChatPcSession } from './session';
