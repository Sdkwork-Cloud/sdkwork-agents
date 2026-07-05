import path from "node:path";
import { fileURLToPath } from "node:url";
import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig, loadEnv } from "vite";

const appRoot = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(appRoot, "../..");

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, appRoot, "");
  return {
    define: {
      "process.env.SDKWORK_ACCESS_TOKEN": JSON.stringify(env.SDKWORK_ACCESS_TOKEN ?? ""),
    },
    plugins: [react(), tailwindcss()],
    resolve: {
      alias: {
        "@sdkwork/agents-app-sdk": path.resolve(
          repoRoot,
          "sdks/sdkwork-agents-app-sdk/sdkwork-agents-app-sdk-typescript/src/index.ts",
        ),
        "@sdkwork/agents-pc-core": path.resolve(appRoot, "packages/sdkwork-agents-pc-core/src"),
        "@sdkwork/agents-pc-commons": path.resolve(appRoot, "packages/sdkwork-agents-pc-commons/src/index.ts"),
        "@sdkwork/agents-pc-agents": path.resolve(appRoot, "packages/sdkwork-agents-pc-agents/src/index.ts"),
        "@sdkwork/agents-pc-shell": path.resolve(appRoot, "packages/sdkwork-agents-pc-shell/src/index.ts"),
        "@sdkwork/sdk-common": path.resolve(
          repoRoot,
          "../sdkwork-sdk-commons/sdkwork-sdk-common-typescript/src/index.ts",
        ),
      },
    },
    server: { port: 5195 },
  };
});
