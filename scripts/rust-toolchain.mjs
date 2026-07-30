import { spawnSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import process from 'node:process';

function platformPath(platform) {
  return platform === 'win32' ? path.win32 : path.posix;
}

function pathDelimiter(platform) {
  return platform === 'win32' ? ';' : ':';
}

function pathKey(env, platform) {
  return Object.keys(env).find((key) => key.toUpperCase() === 'PATH')
    ?? (platform === 'win32' ? 'Path' : 'PATH');
}

function executableName(command, platform) {
  return platform === 'win32' ? `${command}.exe` : command;
}

function normalizePathEntry(entry, platform) {
  const normalized = platformPath(platform).normalize(String(entry ?? '').trim())
    .replace(/[\\/]+$/u, '');
  return platform === 'win32' ? normalized.toLowerCase() : normalized;
}

function uniquePathEntries(entries, platform) {
  const seen = new Set();
  return entries.filter((entry) => {
    const normalized = normalizePathEntry(entry, platform);
    if (!normalized || seen.has(normalized)) return false;
    seen.add(normalized);
    return true;
  });
}

function cargoBinCandidates(env, platform, homeDir) {
  const pathModule = platformPath(platform);
  const candidates = [];
  const cargoHome = String(env.CARGO_HOME ?? '').trim();
  if (cargoHome) {
    candidates.push(
      pathModule.basename(cargoHome).toLowerCase() === 'bin'
        ? cargoHome
        : pathModule.join(cargoHome, 'bin'),
    );
  }
  if (homeDir) candidates.push(pathModule.join(homeDir, '.cargo', 'bin'));
  return uniquePathEntries(candidates, platform);
}

export function withRustToolchainPath(baseEnv = process.env, options = {}) {
  const platform = options.platform ?? process.platform;
  const env = { ...baseEnv };
  const selectedPathKey = pathKey(env, platform);
  const currentPath = env[selectedPathKey] ?? env.PATH ?? env.Path ?? '';
  const entries = String(currentPath)
    .split(pathDelimiter(platform))
    .map((entry) => entry.trim())
    .filter(Boolean);
  const homeDir = String(
    options.homeDir
      ?? (platform === 'win32' ? env.USERPROFILE : env.HOME)
      ?? os.homedir(),
  ).trim();
  const requiredCommands = options.requiredCommands ?? ['cargo', 'rustc'];
  const pathExists = options.pathExists ?? existsSync;

  for (const candidate of cargoBinCandidates(env, platform, homeDir).reverse()) {
    const hasToolchain = requiredCommands.every((command) => {
      return pathExists(platformPath(platform).join(candidate, executableName(command, platform)));
    });
    if (hasToolchain) entries.unshift(candidate);
  }

  for (const key of Object.keys(env)) {
    if (key !== selectedPathKey && key.toUpperCase() === 'PATH') delete env[key];
  }
  env[selectedPathKey] = uniquePathEntries(entries, platform).join(pathDelimiter(platform));
  return env;
}

export function ensureRustToolchain(options = {}) {
  const platform = options.platform ?? process.platform;
  const env = withRustToolchainPath(options.env ?? process.env, options);
  const command = executableName('cargo', platform);
  const result = (options.runProcess ?? spawnSync)(command, ['--version'], {
    cwd: options.cwd ?? process.cwd(),
    env,
    encoding: 'utf8',
    shell: false,
    windowsHide: true,
  });

  if (result.error) {
    const installHint = platform === 'win32'
      ? 'Install Rust with rustup, or ensure %USERPROFILE%\\.cargo\\bin contains cargo.exe and rustc.exe.'
      : 'Install Rust with rustup, or ensure $HOME/.cargo/bin contains cargo and rustc.';
    throw new Error(
      `Rust/Cargo toolchain is required for SDKWork Agents standalone development. ${result.error.message}\n${installHint}`,
    );
  }
  if (result.status !== 0) {
    const detail = String(result.stderr ?? result.stdout ?? '').trim();
    throw new Error(`cargo --version exited with code ${result.status ?? 1}${detail ? `: ${detail}` : ''}`);
  }
  return env;
}
