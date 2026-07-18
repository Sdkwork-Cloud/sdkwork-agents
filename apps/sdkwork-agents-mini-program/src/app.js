const { bootstrapAgentsMiniProgram } = require("./runtime/agents-app");

App({
  globalData: {
    agentsAppApiBaseUrl: "http://127.0.0.1:8095/app/v3/api",
    agentsH5Url: "http://127.0.0.1:5196",
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
