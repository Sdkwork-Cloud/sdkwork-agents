const { bootstrapAgentsMiniProgram } = require("./runtime/agents-app");
const runtimeEnv = require("./runtime/runtime-env");

App({
  globalData: {
    sdkworkProfileId: runtimeEnv.SDKWORK_PROFILE_ID,
    agentsAppApiBaseUrl: runtimeEnv.SDKWORK_AGENTS_APP_API_BASE_URL,
  },
  onLaunch() {
    try {
      bootstrapAgentsMiniProgram({
        appApiBaseUrl: this.globalData.agentsAppApiBaseUrl,
        accessToken: this.globalData.sdkworkAccessToken,
      });
    } catch {
      // Runtime bundle is produced by pnpm run build.
    }
    wx.reLaunch({ url: "/pages/home/index" });
  },
});
