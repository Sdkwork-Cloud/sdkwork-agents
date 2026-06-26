const DEFAULT_AGENTS_H5_URL = "http://127.0.0.1:5196";

Page({
  data: {
    agentsH5Url: DEFAULT_AGENTS_H5_URL,
  },
  onLoad() {
    const configured = getApp()?.globalData?.agentsH5Url;
    if (typeof configured === "string" && configured.trim().length > 0) {
      this.setData({ agentsH5Url: configured.trim() });
    }
  },
});
