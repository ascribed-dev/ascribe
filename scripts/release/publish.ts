// Publishes a packed release (scripts/release/pack.ts's output).
//
//   node scripts/release/publish.ts npm [--from <dir>] [--dry-run]
//   node scripts/release/publish.ts marketplace [--from <dir>] [--dry-run]
//
// Every target is required, unless --targets names the ones packed (a local
// dry run on one machine, like pack.ts's).
//
// npm: publishes every tarball in dependency order (the platform packages,
// then @ascribed/cli, @ascribed/elements, @ascribed/review, and
// @ascribed/astro), with provenance when it runs in GitHub Actions. In
// Actions it authenticates with npm trusted publishing, so there's no token.
// Run elsewhere, it needs NODE_AUTH_TOKEN.
//
// marketplace: publishes every extension package, authenticating with Microsoft
// Entra (`vsce publish --azure-credential`). Sign in first, as the workflow
// does with azure/login; there is no token.
//
// Both skip what's already published, so a release that failed partway can be
// run again. With --dry-run, nothing is published: npm checks each tarball
// with `npm publish --dry-run`, and each extension package's manifest is read
// and checked.
import { execFileSync, spawnSync } from "node:child_process";
import { existsSync, readdirSync } from "node:fs";
import { join } from "node:path";
import process from "node:process";
import { parseArgs } from "node:util";
import {
  checkVersion,
  extension,
  npmPackages,
  readJson,
  root,
  targets,
  type PackageManifest,
} from "./manifests.ts";

const { values: options, positionals } = parseArgs({
  allowPositionals: true,
  options: {
    from: { type: "string", default: join(root, "dist", "release") },
    "dry-run": { type: "boolean", default: false },
    targets: { type: "string" },
  },
});
const dryRun = options["dry-run"];
const [registry] = positionals;
const selected = options.targets?.split(",") ?? targets.map((t) => t.target);
const expected = npmPackages.filter(
  (pkg) => pkg.target === undefined || selected.includes(pkg.target),
);

const { version, problems } = checkVersion();
if (problems.length > 0) fail(problems.join("\n"));

if (registry === "npm") publishNpm();
else if (registry === "marketplace") publishMarketplace();
else fail("usage: publish.ts npm|marketplace [--from <dir>] [--dry-run]");

function publishNpm(): void {
  checkRepository();
  const dir = join(options.from, "npm");
  const tarballs = new Map<string, string>(
    readdirSync(dir).map((file) => {
      const manifest = JSON.parse(
        execFileSync("tar", ["-xzOf", join(dir, file), "package/package.json"], {
          encoding: "utf8",
        }),
      ) as PackageManifest;
      return [manifest.name, join(dir, file)];
    }),
  );
  const missing = expected.filter((pkg) => !tarballs.has(pkg.name)).map((pkg) => pkg.name);
  if (missing.length > 0) fail(`no tarball for ${missing.join(", ")} in ${dir}`);

  // A pre-release goes to the `next` tag, so `npm install` keeps the last release.
  const tag = version.includes("-") ? "next" : "latest";
  for (const { name } of expected) {
    const tarball = tarballs.get(name);
    if (tarball === undefined) fail(`no tarball for ${name} in ${dir}`);
    if (isPublished(name)) {
      log(`${name}@${version} is already on npm; skipping`);
      continue;
    }
    const args = ["publish", tarball, "--access", "public", "--tag", tag];
    // Provenance needs the workflow's OIDC token, which only a real publish has.
    if (dryRun) args.push("--dry-run");
    else if (process.env.GITHUB_ACTIONS === "true") args.push("--provenance");
    log(`npm ${args.join(" ")}`);
    run("npm", args);
  }
}

/** Whether npm already has this version of a package. */
function isPublished(name: string): boolean {
  const result = spawnSync("npm", ["view", `${name}@${version}`, "version"], {
    encoding: "utf8",
    shell: process.platform === "win32",
  });
  if (result.status === 0) return result.stdout.trim() === version;
  if (/E404|404 Not Found/.test(result.stderr)) return false;
  fail(`couldn't ask npm about ${name}: ${result.stderr.trim()}`);
}

/**
 * npm checks that a package with provenance names the repository that built
 * it. Checking first stops a mismatch before anything is published.
 */
function checkRepository(): void {
  const repository = process.env.GITHUB_REPOSITORY;
  if (repository === undefined) return;
  for (const { dir } of [...npmPackages, extension]) {
    const url = readJson(`${dir}/package.json`).repository?.url ?? "";
    if (!url.toLowerCase().includes(`github.com/${repository.toLowerCase()}.git`)) {
      fail(
        `${dir}/package.json's repository is ${url || "missing"}, but this runs in ` +
          `${repository}; npm rejects provenance from another repository`,
      );
    }
  }
}

function publishMarketplace(): void {
  const dir = join(options.from, "vsix");
  const packages = selected.map((target) => ({
    target,
    file: join(dir, `ascribe-vscode-${target}-${version}.vsix`),
  }));
  const missing = packages.filter((p) => !existsSync(p.file)).map((p) => p.target);
  if (missing.length > 0) fail(`no extension package for ${missing.join(", ")} in ${dir}`);

  for (const { target, file } of packages) {
    const manifest = JSON.parse(
      execFileSync("unzip", ["-p", file, "extension/package.json"], { encoding: "utf8" }),
    ) as PackageManifest;
    const id = `${manifest.publisher}.${manifest.name}`;
    if (id !== extension.id || manifest.version !== version) {
      fail(`${file} is ${id} ${manifest.version}, not ${extension.id} ${version}`);
    }
    if (dryRun) {
      log(`would publish ${id} ${version} for ${target}`);
      continue;
    }
    log(`publishing ${id} ${version} for ${target}`);
    // --skip-duplicate makes a rerun skip a target that's already published.
    const vsce = join(root, extension.dir, "node_modules", ".bin", "vsce");
    run(vsce, ["publish", "--packagePath", file, "--skip-duplicate", "--azure-credential"]);
  }
}

function run(command: string, args: string[]): void {
  const result = spawnSync(command, args, {
    cwd: root,
    stdio: "inherit",
    shell: process.platform === "win32",
  });
  if (result.status !== 0) fail(`${command} ${args.join(" ")} failed`);
}

function log(message: string): void {
  process.stdout.write(`${message}\n`);
}

function fail(message: string): never {
  process.stderr.write(`error: ${message}\n`);
  process.exit(1);
}
