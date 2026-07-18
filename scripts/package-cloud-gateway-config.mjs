import { copyFileSync, existsSync, mkdirSync, readFileSync, rmSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { execFileSync } from "node:child_process";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const distDir = path.join(repoRoot, "dist", "cloud-config");
const bundleDir = path.join(distDir, "bundle");
const topologySpecPath = "specs/topology.spec.json";

function ensureBuildCriticalSource(relativePath) {
  const sourcePath = path.join(repoRoot, relativePath);
  if (!existsSync(sourcePath)) {
    try {
      execFileSync("git", ["checkout", "HEAD", "--", relativePath], {
        cwd: repoRoot,
        stdio: "ignore",
      });
    } catch {
      // The actionable error below names the missing source and recovery command.
    }
  }
  if (!existsSync(sourcePath)) {
    throw new Error(
      `Missing build-critical source file: ${relativePath}. Recover with: git checkout HEAD -- ${relativePath}`,
    );
  }
  return sourcePath;
}

function loadCloudConfigFiles() {
  const topology = JSON.parse(readFileSync(ensureBuildCriticalSource(topologySpecPath), "utf8"));
  const configFiles = topology.packaging?.cloudConfigFiles;
  if (!Array.isArray(configFiles) || configFiles.length === 0) {
    throw new Error(`${topologySpecPath} must declare packaging.cloudConfigFiles`);
  }
  for (const fileName of configFiles) {
    if (
      typeof fileName !== "string"
      || path.basename(fileName) !== fileName
      || !fileName.endsWith(".toml")
    ) {
      throw new Error(`${topologySpecPath} contains an invalid cloud gateway config file name`);
    }
  }
  return configFiles;
}

const configFiles = loadCloudConfigFiles();

rmSync(bundleDir, { force: true, recursive: true });
mkdirSync(bundleDir, { recursive: true });
for (const fileName of configFiles) {
  const relativePath = `etc/${fileName}`;
  const source = ensureBuildCriticalSource(relativePath);
  copyFileSync(source, path.join(bundleDir, fileName));
}

const stamp = new Date().toISOString().replace(/[:.]/g, "-");
const archivePath = path.join(distDir, `sdkwork-agents-api-gateway-config-${stamp}.tar.gz`);
execFileSync("tar", ["-czf", archivePath, "-C", bundleDir, "."], { stdio: "inherit" });
console.log(`packaged cloud gateway config bundle: ${archivePath}`);
