import * as esbuild from "esbuild";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.dirname(fileURLToPath(import.meta.url));

await esbuild.build({
  entryPoints: [path.join(root, "../src/bootstrap/runtimeBundle.ts")],
  bundle: true,
  outfile: path.join(root, "../src/runtime/agents-app.js"),
  platform: "browser",
  format: "cjs",
  target: "es2019",
  logLevel: "info",
});
