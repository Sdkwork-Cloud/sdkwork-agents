import {
  initSkillsAppSdkClient,
  isSkillsAppSdkConfigured,
} from "@sdkwork/agents-pc-core/sdk/skillsAppSdkClient";
import {
  initVoiceAppSdkClient,
  isVoiceAppSdkConfigured,
} from "@sdkwork/agents-pc-core/sdk/voiceAppSdkClient";

/** Eagerly initialize optional sibling app SDK clients when env is present. */
export function bootstrapSiblingAppSdks(): void {
  if (isSkillsAppSdkConfigured()) {
    initSkillsAppSdkClient();
  }
  if (isVoiceAppSdkConfigured()) {
    initVoiceAppSdkClient();
  }
}
