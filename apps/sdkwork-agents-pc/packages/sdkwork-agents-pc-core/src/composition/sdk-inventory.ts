export const sdkInventory = [
  "@sdkwork/agents-app-sdk",
  "@sdkwork/drive-app-sdk",
  "@sdkwork/knowledgebase-app-sdk",
  "@sdkwork/skills-app-sdk",
  "@sdkwork/voice-app-sdk",
] as const;

export function listSdkworkCoreSdkInventory() {
  return sdkInventory;
}
