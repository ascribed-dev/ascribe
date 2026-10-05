// Packs a release from already-built binaries. Publishes nothing.
//
//   node scripts/release/pack.ts --binaries <dir> [--targets a,b] [--out <dir>]
//
// <dir> holds one directory per target with its binary: darwin-arm64/ascribe,
// win32-x64/ascribe.exe, and so on. By default every target is required;
// --targets packs only those (for a local dry run on one machine). The output,
// by default dist/release/, gets:
//
//   npm/     one tarball per npm package, versions and dependencies resolved
//   vsix/    one VS Code extension package per target, with its binary
//   github/  one archive of the binary per target, for the GitHub release
//   SHA256SUMS
//
// Each package is checked after it's packed: the version, no `workspace:`
// dependencies, and an executable binary where one belongs.
import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  chmodSync,
  copyFileSync,
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { basename, join, relative } from "node:path";
import process from "node:process";
import { parseArgs } from "node:util";
import {
  checkVersion,
  extension,
  npmPackages,
  root,
  targets,
  type PackageManifest,
  type Target,
} from "./manifests.ts";
import { thirdPartyNotices } from "./notices.ts";

class ReleaseError extends Error {}
process.on("uncaughtException", (error: Error) => {
  process.stderr.write(`error: ${error instanceof ReleaseError ? error.message : error.stack}\n`);
  process.exit(1);
});

const { values: options } = parseArgs({
  options: {
    binaries: { type: "string" },
    targets: { type: "string" },
    out: { type: "string", default: join(root, "dist", "release") },
    help: { type: "boolean", default: false },
  },
});
if (options.help) {
  process.stdout.write(
    "usage: node scripts/release/pack.ts --binaries <dir> [--targets a,b] [--out <dir>]\n",
  );
  process.exit(0);
}
const binaries = options.binaries;
if (binaries === undefined) fail("--binaries <dir> is required");

const { version, problems } = checkVersion();
if (problems.length > 0) fail(problems.join("\n"));

const wanted = options.targets?.split(",") ?? targets.map((t) => t.target);
const selected = wanted.map((name) => {
  const target = targets.find((t) => t.target === name);
  if (!target)
    fail(`unknown target ${name}; the targets are ${targets.map((t) => t.target).join(", ")}`);
  const binary = join(binaries, name, target.exe);
  if (!existsSync(binary)) fail(`no binary for ${name} at ${binary}`);
  return { ...target, binary };
});

const out = options.out;
rmSync(out, { recursive: true, force: true });
for (const dir of ["npm", "vsix", "github"]) mkdirSync(join(out, dir), { recursive: true });

// Files copied into packages for packing, removed afterwards.
const temporary: string[] = [];
const copyTemporary = (from: string, to: string): void => {
  if (existsSync(to)) return;
  copyFileSync(from, to);
  temporary.push(to);
};
const vscodeBin = join(root, extension.dir, "bin");

try {
  for (const { dir } of [...npmPackages, extension]) {
    copyTemporary(join(root, "LICENSE"), join(root, dir, "LICENSE"));
  }
  copyTemporary(join(root, "CHANGELOG.md"), join(root, extension.dir, "CHANGELOG.md"));
  // The packages that hold the binary carry the license text of the crates in it.
  const notices = join(out, "THIRD-PARTY-NOTICES");
  writeFileSync(notices, thirdPartyNotices());
  const withBinary = [...npmPackages.filter((pkg) => pkg.target !== undefined), extension];
  for (const { dir } of withBinary) {
    copyTemporary(notices, join(root, dir, "THIRD-PARTY-NOTICES"));
  }

  step("Building the JavaScript packages");
  pnpm([
    "--filter",
    "@ascribed/cli",
    "--filter",
    "@ascribed/elements",
    "--filter",
    "@ascribed/astro",
    "--filter",
    "@ascribed/review",
    "run",
    "build",
  ]);
  run("node", ["esbuild.mjs", "--minify"], join(root, extension.dir));

  step("Packing the npm packages");
  for (const pkg of npmPackages) {
    if (pkg.target !== undefined) {
      const target = selected.find((t) => t.target === pkg.target);
      if (!target) continue;
      const staged = join(root, pkg.dir, "bin", target.exe);
      copyFileSync(target.binary, staged);
      if (!target.exe.endsWith(".exe")) chmodSync(staged, 0o755);
      // npm pack keeps the binary's executable bit.
      run("npm", ["pack", join(root, pkg.dir), "--pack-destination", join(out, "npm")]);
    } else {
      // pnpm pack replaces `workspace:*` with the version.
      pnpm(["pack", "--pack-destination", join(out, "npm")], join(root, pkg.dir));
    }
  }
  for (const file of readdirSync(join(out, "npm"))) checkTarball(join(out, "npm", file));

  step("Packing the VS Code extension");
  for (const target of selected) {
    rmSync(vscodeBin, { recursive: true, force: true });
    const dir = join(vscodeBin, target.target);
    mkdirSync(dir, { recursive: true });
    copyFileSync(target.binary, join(dir, target.exe));
    if (!target.exe.endsWith(".exe")) chmodSync(join(dir, target.exe), 0o755);
    const vsix = join(out, "vsix", `ascribe-vscode-${target.target}-${version}.vsix`);
    pnpm(
      ["exec", "vsce", "package", "--no-dependencies", "--target", target.target, "--out", vsix],
      join(root, extension.dir),
    );
    checkVsix(vsix, target);
  }

  step("Archiving the binaries");
  for (const target of selected) {
    const stage = join(out, "github", `ascribe-${version}-${target.target}`);
    mkdirSync(stage);
    copyFileSync(target.binary, join(stage, target.exe));
    if (!target.exe.endsWith(".exe")) chmodSync(join(stage, target.exe), 0o755);
    copyFileSync(join(root, "LICENSE"), join(stage, "LICENSE"));
    copyFileSync(notices, join(stage, "THIRD-PARTY-NOTICES"));
    const name = basename(stage);
    if (target.target.startsWith("win32")) {
      run("zip", ["-q", "-r", `${name}.zip`, name], join(out, "github"));
    } else {
      run("tar", ["-czf", `${name}.tar.gz`, name], join(out, "github"));
    }
    rmSync(stage, { recursive: true });
  }

  const sums = listFiles(out)
    .map((file) => `${sha256(file)}  ${relative(out, file).split("\\").join("/")}`)
    .join("\n");
  writeFileSync(join(out, "SHA256SUMS"), `${sums}\n`);
} finally {
  for (const file of temporary) rmSync(file, { force: true });
  rmSync(vscodeBin, { recursive: true, force: true });
}

step(`Packed ${version} in ${relative(process.cwd(), out) || "."}`);
for (const file of listFiles(out)) process.stdout.write(`  ${relative(out, file)}\n`);

/** A packed npm package: its manifest and the executable bit of a native binary. */
function checkTarball(file: string): void {
  const manifest = JSON.parse(tar(["-xzOf", file, "package/package.json"])) as PackageManifest & {
    private?: boolean;
    dependencies?: Record<string, string>;
    optionalDependencies?: Record<string, string>;
    peerDependencies?: Record<string, string>;
  };
  const where = `${basename(file)}:`;
  if (manifest.version !== version) fail(`${where} version ${manifest.version}, not ${version}`);
  if (manifest.private) fail(`${where} is private`);
  for (const field of ["dependencies", "optionalDependencies", "peerDependencies"] as const) {
    for (const [name, range] of Object.entries(manifest[field] ?? {})) {
      if (String(range).startsWith("workspace:")) fail(`${where} ${name} is ${range}`);
    }
  }
  const listing = tar(["-tzvf", file]);
  if (!listing.includes("package/LICENSE")) fail(`${where} has no LICENSE`);
  if (
    manifest.name.startsWith("@ascribed/cli-") &&
    !listing.includes("package/THIRD-PARTY-NOTICES")
  ) {
    fail(`${where} has no THIRD-PARTY-NOTICES`);
  }
  const binary = listing.split("\n").find((l) => l.endsWith("package/bin/ascribe"));
  if (binary !== undefined && !binary.startsWith("-rwx")) {
    fail(`${where} bin/ascribe isn't executable: ${binary}`);
  }
  process.stdout.write(`checked ${basename(file)}\n`);
}

/** A VS Code extension package: its manifest, and the binary for its target. */
function checkVsix(file: string, target: Target): void {
  const where = `${basename(file)}:`;
  const manifest = JSON.parse(unzip(["-p", file, "extension/package.json"])) as PackageManifest;
  if (`${manifest.publisher}.${manifest.name}` !== extension.id) {
    fail(`${where} is ${manifest.publisher}.${manifest.name}, not ${extension.id}`);
  }
  if (manifest.version !== version) fail(`${where} version ${manifest.version}, not ${version}`);
  const listing = unzip(["-Z", file]);
  const entry = `extension/bin/${target.target}/${target.exe}`;
  const line = listing.split("\n").find((l) => l.endsWith(entry));
  if (!line) fail(`${where} has no ${entry}`);
  if (!target.exe.endsWith(".exe") && !line.startsWith("-rwx")) {
    fail(`${where} ${entry} isn't executable: ${line}`);
  }
  for (const needed of [
    "extension/THIRD-PARTY-NOTICES",
    "extension/dist/extension.cjs",
    "extension/dist/webview/elements.js",
  ]) {
    if (!listing.includes(needed)) fail(`${where} has no ${needed}`);
  }
  const bins = listing.split("\n").filter((l) => l.includes("extension/bin/") && !l.endsWith("/"));
  if (bins.length !== 1) fail(`${where} has ${bins.length} binaries; it should have one`);
  process.stdout.write(`checked ${basename(file)}\n`);
}

function listFiles(dir: string): string[] {
  return readdirSync(dir, { recursive: true })
    .map((entry) => join(dir, String(entry)))
    .filter((path) => statSync(path).isFile() && basename(path) !== "SHA256SUMS")
    .sort();
}

function sha256(file: string): string {
  return createHash("sha256").update(readFileSync(file)).digest("hex");
}

function tar(args: string[]): string {
  return execFileSync("tar", args, { encoding: "utf8" });
}

function unzip(args: string[]): string {
  return execFileSync("unzip", args, { encoding: "utf8", maxBuffer: 64 * 1024 * 1024 });
}

/** Runs pnpm, through corepack when pnpm itself isn't on the path. */
function pnpm(args: string[], cwd = root): void {
  const direct = spawnSync("pnpm", ["--version"], { stdio: "ignore", shell: isWindows() });
  if (direct.status === 0) run("pnpm", args, cwd);
  else run("corepack", ["pnpm", ...args], cwd);
}

function run(command: string, args: string[], cwd = root): void {
  const result = spawnSync(command, args, { cwd, stdio: "inherit", shell: isWindows() });
  if (result.status !== 0) fail(`${command} ${args.join(" ")} failed`);
}

function isWindows(): boolean {
  return process.platform === "win32";
}

function step(message: string): void {
  process.stdout.write(`\n== ${message}\n`);
}

/** Stops the script. It throws, so the cleanup in `finally` still runs. */
function fail(message: string): never {
  throw new ReleaseError(message);
}
