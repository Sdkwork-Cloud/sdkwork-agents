const { bootstrapAgentsMiniProgram } = require("./runtime/agents-app");

App({
  onLaunch() {
    try {
      bootstrapAgentsMiniProgram();
    } catch {
      // Runtime bundle is produced by pnpm run build.
    }
    wx.reLaunch({ url: "/pages/home/index" });
  },
});
