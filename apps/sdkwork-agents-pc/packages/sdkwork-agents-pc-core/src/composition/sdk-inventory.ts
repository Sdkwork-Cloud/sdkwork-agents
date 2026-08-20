export const sdkInventory = [
  "@sdkwork/agents-app-sdk",
  "@sdkwork/assets-app-sdk",
  "@sdkwork/community-app-sdk",
  "@sdkwork/feeds-sdk",
  "@sdkwork/drive-app-sdk",
  "@sdkwork/generations-app-sdk",
  "@sdkwork/knowledgebase-app-sdk",
  "@sdkwork/memory-app-sdk",
  "@sdkwork/prompts-app-sdk",
  "@sdkwork/skills-app-sdk",
  "@sdkwork/voice-app-sdk",
] as const;

export function listSdkworkCoreSdkInventory() {
  return sdkInventory;
}
