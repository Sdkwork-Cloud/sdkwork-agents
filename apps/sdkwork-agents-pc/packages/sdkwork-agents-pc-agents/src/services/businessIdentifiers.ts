import { uuid } from "@sdkwork/utils/id";

const CLIENT_SURFACE = "pc";
const AGENT_ID_SUFFIX_LENGTH = 12;

function normalizeBusinessToken(value: string): string {
  return value
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/gu, "-")
    .replace(/^-+|-+$/gu, "")
    .slice(0, 48);
}

function compactUuidSuffix(): string {
  return uuid().replace(/-/gu, "").toLowerCase().slice(0, AGENT_ID_SUFFIX_LENGTH);
}

export function createAgentBusinessId(name: string): string {
  const normalizedName = normalizeBusinessToken(name);
  return `agent.${CLIENT_SURFACE}.${normalizedName || "managed"}.${compactUuidSuffix()}`;
}

export function createAgentExecutionId(operation: "preview" | "prompt"): string {
  return `execution.${CLIENT_SURFACE}.agent.${operation}.${uuid().toLowerCase()}`;
}
