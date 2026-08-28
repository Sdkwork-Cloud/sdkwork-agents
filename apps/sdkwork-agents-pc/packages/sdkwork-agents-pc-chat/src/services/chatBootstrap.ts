import { ChatService } from '../services/ChatService';

export interface ChatBootstrapResult {
  sessions: Awaited<ReturnType<typeof ChatService.loadSessions>>;
  currentSessionId: string;
}

let bootstrapPromise: Promise<ChatBootstrapResult> | null = null;

/**
 * Ensures session bootstrap runs once per page load (React Strict Mode safe).
 * Concurrent callers share the same in-flight promise.
 */
export function bootstrapChatSessions(
  model: string,
  newChatTitle: string,
): Promise<ChatBootstrapResult> {
  if (!bootstrapPromise) {
    bootstrapPromise = (async () => {
      const remoteSessions = await ChatService.loadSessions(model);
      if (remoteSessions.length === 0) {
        const created = await ChatService.createSession(model, newChatTitle);
        return {
          sessions: [created],
          currentSessionId: created.id,
        };
      }
      return {
        sessions: remoteSessions,
        currentSessionId: remoteSessions[0].id,
      };
    })().catch((error) => {
      bootstrapPromise = null;
      throw error;
    });
  }
  return bootstrapPromise;
}

/** Test-only reset for isolated unit tests. */
export function resetChatBootstrapForTests(): void {
  bootstrapPromise = null;
}
