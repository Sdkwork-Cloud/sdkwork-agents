const DEFAULT_APP_API_BASE_URL = "http://127.0.0.1:8095/app/v3/api";

function mapAgentRecord(record) {
  if (!record || typeof record !== "object") {
    return null;
  }
  const id = record.agentId ?? record.id ?? record.code;
  const name = record.displayName ?? record.code ?? "Agent";
  if (!id || typeof id !== "string") {
    return null;
  }
  return {
    id,
    name: typeof name === "string" ? name : String(name),
    description: typeof record.description === "string" ? record.description : "",
  };
}

function readHasMore(response) {
  const pageInfo =
    (response && typeof response === "object" && response.pageInfo) ||
    (response &&
      typeof response === "object" &&
      response.data &&
      typeof response.data === "object" &&
      response.data.pageInfo) ||
    {};
  const page = Number(pageInfo.page || 1);
  const totalPages = Number(pageInfo.totalPages || pageInfo.total_pages || 0);
  if (pageInfo.hasMore === true) {
    return true;
  }
  return totalPages > 0 && page < totalPages;
}

function extractAgentItems(response) {
  if (Array.isArray(response?.items)) {
    return response.items;
  }
  if (Array.isArray(response?.data?.items)) {
    return response.data.items;
  }
  return [];
}

Page({
  data: {
    agents: [],
    loading: true,
    error: "",
    loadingMore: false,
    page: 1,
    hasMore: false,
  },
  onLoad() {
    this.loadAgents();
  },
  onPullDownRefresh() {
    this.loadAgents(() => wx.stopPullDownRefresh());
  },
  loadAgents(done) {
    this.setData({ loading: true, error: "", page: 1, hasMore: false });
    this.fetchAgentPage(1, false, done);
  },
  loadMoreAgents() {
    if (this.data.loadingMore || !this.data.hasMore) {
      return;
    }
    this.fetchAgentPage(this.data.page + 1, true);
  },
  fetchAgentPage(page, append, done) {
    if (append) {
      this.setData({ loadingMore: true, error: "" });
    } else {
      this.setData({ loading: true, error: "" });
    }
    try {
      const runtime = require("../../runtime/agents-app");
      const app = getApp();
      const baseUrl =
        (typeof app?.globalData?.agentsAppApiBaseUrl === "string" &&
          app.globalData.agentsAppApiBaseUrl.trim()) ||
        DEFAULT_APP_API_BASE_URL;
      runtime.bootstrapAgentsMiniProgram({ appApiBaseUrl: baseUrl });
      const client = runtime.getAgentsMpSdkClient();
      client.ai.agents
        .list({ page, pageSize: 20 })
        .then((response) => {
          const items = extractAgentItems(response).map(mapAgentRecord).filter(Boolean);
          const agents = append ? this.data.agents.concat(items) : items;
          this.setData({
            agents,
            loading: false,
            loadingMore: false,
            error: "",
            page,
            hasMore: readHasMore(response),
          });
          if (typeof done === "function") {
            done();
          }
        })
        .catch((error) => {
          const message = error?.message ? String(error.message) : String(error);
          this.setData({ loading: false, loadingMore: false, error: message });
          if (typeof done === "function") {
            done();
          }
        });
    } catch (error) {
      const message = error?.message ? String(error.message) : String(error);
      this.setData({ loading: false, loadingMore: false, error: message });
      if (typeof done === "function") {
        done();
      }
    }
  },
});
