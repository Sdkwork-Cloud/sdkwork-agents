import { copyFileSync, mkdirSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { execFileSync } from "node:child_process";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const distDir = path.join(repoRoot, "dist", "cloud-config");
const bundleDir = path.join(distDir, "bundle");
const configFiles = [
  "configs/sdkwork-api-cloud-gateway.agents.development.toml",
  "configs/sdkwork-api-cloud-gateway.agents.production.toml",
];

mkdirSync(bundleDir, { recursive: true });
for (const relativePath of configFiles) {
  copyFileSync(path.join(repoRoot, relativePath), path.join(bundleDir, path.basename(relativePath)));
}

const stamp = new Date().toISOString().replace(/[:.]/g, "-");
const archivePath = path.join(distDir, `sdkwork-agents-api-gateway-config-${stamp}.tar.gz`);
execFileSync("tar", ["-czf", archivePath, "-C", bundleDir, "."], { stdio: "inherit" });
console.log(`packaged cloud gateway config bundle: ${archivePath}`);
