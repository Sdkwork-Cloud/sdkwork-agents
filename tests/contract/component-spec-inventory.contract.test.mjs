import assert from "node:assert/strict";
import { existsSync, readdirSync, readFileSync } from "node:fs";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

const skippedDirectories = new Set([
  ".git",
  ".runtime",
  "dist",
  "node_modules",
  "target",
]);

function toPosixPath(filePath) {
  return filePath.replaceAll(path.sep, "/");
}

function collectComponentSpecs(directory = repoRoot, results = []) {
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    if (entry.isDirectory()) {
      if (!skippedDirectories.has(entry.name)) {
        collectComponentSpecs(path.join(directory, entry.name), results);
      }
      continue;
    }

    if (entry.name === "component.spec.json" && path.basename(directory) === "specs") {
      results.push(path.join(directory, entry.name));
    }
  }
  return results;
}

function readJson(filePath) {
  return JSON.parse(readFileSync(filePath, "utf8"));
}

function collectMarkdownFiles(directory, results = []) {
  if (!existsSync(directory)) {
    return results;
  }

  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const entryPath = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      if (!skippedDirectories.has(entry.name)) {
        collectMarkdownFiles(entryPath, results);
      }
      continue;
    }

    if (entry.name.endsWith(".md")) {
      results.push(entryPath);
    }
  }

  return results;
}

function tokenizeCommand(command) {
  return Array.from(command.matchAll(/"([^"]*)"|'([^']*)'|([^\s]+)/gu), (match) => {
    return match[1] ?? match[2] ?? match[3];
  });
}

function findNodeScriptTarget(command) {
  const tokens = tokenizeCommand(command);
  if (tokens[0] !== "node") {
    return null;
  }

  for (let index = 1; index < tokens.length; index += 1) {
    const token = tokens[index];
    if (token === "--") {
      return null;
    }
    if (token === "-e" || token === "--eval" || token === "-p" || token === "--print") {
      return null;
    }
    if (token === "--test") {
      continue;
    }
    if (token.startsWith("-")) {
      continue;
    }
    if (/\.[cm]?js$/u.test(token)) {
      return token;
    }
    return null;
  }

  return null;
}

function extractNodeCommandsFromMarkdown(content) {
  const commands = [];
  for (const line of content.split(/\r?\n/u)) {
    for (const match of line.matchAll(/`(node\s+[^`]+)`/gu)) {
      commands.push(match[1]);
    }

    const directCommand = line.trim().match(/^(?:[-*]\s+)?(node\s+.+)$/u);
    if (directCommand) {
      commands.push(directCommand[1]);
    }
  }
  return commands;
}

function assertNodeScriptTargetExists(relativeSourcePath, componentRoot, command) {
  const scriptTarget = findNodeScriptTarget(command);
  if (!scriptTarget) {
    return;
  }

  const repoRelativeTarget = path.resolve(repoRoot, scriptTarget);
  const componentRelativeTarget = path.resolve(componentRoot, scriptTarget);
  assert.ok(
    existsSync(repoRelativeTarget) || existsSync(componentRelativeTarget),
    `${relativeSourcePath} verification command must reference an existing node script: ${command}`,
  );
}

test("child component specs declare repository-local physical roots", () => {
  const specPaths = collectComponentSpecs();

  assert.ok(specPaths.length > 0, "component spec inventory must discover local component specs");

  for (const specPath of specPaths) {
    const relativeSpecPath = toPosixPath(path.relative(repoRoot, specPath));
    if (relativeSpecPath === "specs/component.spec.json") {
      continue;
    }

    const expectedRoot = toPosixPath(path.relative(repoRoot, path.dirname(path.dirname(specPath))));
    const spec = readJson(specPath);
    const actualRoot = spec.component?.root;

    assert.equal(
      actualRoot,
      expectedRoot,
      `${relativeSpecPath} component.root must match its repository-local physical component root`,
    );
    assert.ok(
      !path.isAbsolute(actualRoot),
      `${relativeSpecPath} component.root must be repository-relative, not absolute`,
    );
    assert.ok(
      !actualRoot.split("/").includes(".."),
      `${relativeSpecPath} component.root must not escape the repository root`,
    );
    assert.ok(
      !actualRoot.startsWith("sdkwork-kernel/"),
      `${relativeSpecPath} component.root must not point to sdkwork-kernel`,
    );
    assert.ok(
      existsSync(path.join(repoRoot, actualRoot)),
      `${relativeSpecPath} component.root must resolve under sdkwork-agents`,
    );
  }
});

test("child component specs link canonical standards and verification commands", () => {
  const specPaths = collectComponentSpecs();

  assert.ok(specPaths.length > 0, "component spec inventory must discover local component specs");

  for (const specPath of specPaths) {
    const relativeSpecPath = toPosixPath(path.relative(repoRoot, specPath));
    if (relativeSpecPath === "specs/component.spec.json") {
      continue;
    }

    const spec = readJson(specPath);
    const componentRoot = path.join(repoRoot, spec.component.root);
    const canonicalSpecs = spec.canonicalSpecs ?? [];
    const verificationCommands = spec.verification?.commands ?? [];

    assert.ok(
      canonicalSpecs.length > 0,
      `${relativeSpecPath} must declare canonicalSpecs instead of relying on implicit standards`,
    );
    for (const canonicalSpec of canonicalSpecs) {
      assert.ok(
        canonicalSpec.file,
        `${relativeSpecPath} canonicalSpecs entries must name the spec file`,
      );
      assert.ok(
        canonicalSpec.path,
        `${relativeSpecPath} canonicalSpecs entries must include a component-root-relative path`,
      );
      assert.ok(
        existsSync(path.resolve(componentRoot, canonicalSpec.path)),
        `${relativeSpecPath} canonical spec path must resolve from component.root: ${canonicalSpec.path}`,
      );
    }

    assert.ok(
      verificationCommands.length > 0,
      `${relativeSpecPath} must declare at least one verification command`,
    );
  }
});

test("component specs declare manifests that exist under their component roots", () => {
  const specPaths = collectComponentSpecs();

  assert.ok(specPaths.length > 0, "component spec inventory must discover local component specs");

  for (const specPath of specPaths) {
    const relativeSpecPath = toPosixPath(path.relative(repoRoot, specPath));
    const spec = readJson(specPath);
    const componentRoot = path.join(repoRoot, spec.component?.root ?? "");
    const manifests = spec.component?.manifests ?? [];

    assert.ok(
      Array.isArray(manifests),
      `${relativeSpecPath} component.manifests must be an array when declared`,
    );

    for (const manifest of manifests) {
      assert.equal(
        typeof manifest,
        "string",
        `${relativeSpecPath} component.manifests entries must be relative file paths`,
      );
      assert.ok(
        !path.isAbsolute(manifest),
        `${relativeSpecPath} manifest ${manifest} must be component-root-relative`,
      );
      assert.ok(
        !toPosixPath(manifest).split("/").includes(".."),
        `${relativeSpecPath} manifest ${manifest} must not escape the component root`,
      );
      assert.ok(
        existsSync(path.join(componentRoot, manifest)),
        `${relativeSpecPath} manifest ${manifest} must exist under ${spec.component.root}`,
      );
    }
  }
});

test("component spec node verification commands reference existing scripts", () => {
  const specPaths = collectComponentSpecs();

  assert.ok(specPaths.length > 0, "component spec inventory must discover local component specs");

  for (const specPath of specPaths) {
    const relativeSpecPath = toPosixPath(path.relative(repoRoot, specPath));
    const spec = readJson(specPath);
    const componentRoot = path.join(repoRoot, spec.component?.root ?? "");
    const verificationCommands = spec.verification?.commands ?? [];

    for (const command of verificationCommands) {
      assert.equal(
        typeof command,
        "string",
        `${relativeSpecPath} verification commands must be strings`,
      );

      assertNodeScriptTargetExists(relativeSpecPath, componentRoot, command);
    }
  }
});

test("component documentation node verification commands reference existing scripts", () => {
  const specPaths = collectComponentSpecs();

  assert.ok(specPaths.length > 0, "component spec inventory must discover local component specs");

  for (const specPath of specPaths) {
    const spec = readJson(specPath);
    const componentRoot = path.join(repoRoot, spec.component?.root ?? "");
    const markdownFiles = [
      path.join(componentRoot, "README.md"),
      ...collectMarkdownFiles(path.join(componentRoot, "specs")),
    ].filter((filePath, index, files) => {
      return existsSync(filePath) && files.indexOf(filePath) === index;
    });

    for (const markdownFile of markdownFiles) {
      const relativeMarkdownPath = toPosixPath(path.relative(repoRoot, markdownFile));
      const commands = extractNodeCommandsFromMarkdown(readFileSync(markdownFile, "utf8"));
      for (const command of commands) {
        assertNodeScriptTargetExists(relativeMarkdownPath, componentRoot, command);
      }
    }
  }
});
