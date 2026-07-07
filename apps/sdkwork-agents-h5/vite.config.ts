import path from "node:path";
import { fileURLToPath } from "node:url";
import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig, loadEnv } from "vite";

const appRoot = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(appRoot, "../..");

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, appRoot, "");
  const define: Record<string, string> = {};
  if (mode === "development" && env.SDKWORK_ACCESS_TOKEN) {
    define["process.env.SDKWORK_ACCESS_TOKEN"] = JSON.stringify(env.SDKWORK_ACCESS_TOKEN);
  }
  return {
    define,
    plugins: [react(), tailwindcss()],
    resolve: {
      alias: {
        "@sdkwork/agents-app-sdk": path.resolve(
          repoRoot,
          "sdks/sdkwork-agents-app-sdk/sdkwork-agents-app-sdk-typescript/src/index.ts",
        ),
        "@sdkwork/agents-h5-core": path.resolve(appRoot, "packages/sdkwork-agents-h5-core/src"),
        "@sdkwork/agents-h5-commons": path.resolve(appRoot, "packages/sdkwork-agents-h5-commons/src/index.ts"),
        "@sdkwork/agents-h5-agents": path.resolve(appRoot, "packages/sdkwork-agents-h5-agents/src/index.ts"),
        "@sdkwork/agents-h5-shell": path.resolve(appRoot, "packages/sdkwork-agents-h5-shell/src/index.ts"),
        "@sdkwork/sdk-common": path.resolve(
          repoRoot,
          "../sdkwork-sdk-commons/sdkwork-sdk-common-typescript/src/index.ts",
        ),
      },
    },
    server: { port: 5196 },
  };
});
