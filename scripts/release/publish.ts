// Publishes a packed release (scripts/release/pack.ts's output).
//
//   node scripts/release/publish.ts npm [--from <dir>] [--dry-run] [--tag <tag>]
//   node scripts/release/publish.ts marketplace [--from <dir>] [--dry-run]
//   node scripts/release/publish.ts tag <tag> [--dry-run]
//   node scripts/release/publish.ts catch-up <from> <to> [--dry-run]
//   node scripts/release/publish.ts missing
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
// --tag publishes under that dist-tag instead of `latest` (or `next` for a
// pre-release). `tag` then points a dist-tag at this version of every package,
// so a canary can publish everything under a holding tag and move `next` only
// once all of it is on npm. It waits a while for npm to show every package,
// and moves no tag until all of them show; when one is still missing at the
// deadline, it moves none, warns, and passes. `catch-up` moves `<to>` to the
// version `<from>` names on @ascribed/cli, if every package shows that version
// now and it isn't deprecated: a later run's catch-up moves the `next` an
// earlier run's `tag` left. `missing` lists the packages npm has never had.
//
// Both skip what's already published, so a release that failed partway can be
// run again. With --dry-run, nothing is published: npm checks each tarball
// with `npm publish --dry-run`, and each extension package's manifest is read
// and checked.
import { execFileSync, spawnSync } from "node:child_process";
import { appendFileSync, existsSync, readdirSync } from "node:fs";
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
    tag: { type: "string" },
  },
  // Only the command line's own run reads it; a test imports the functions.
  args: import.meta.main ? process.argv.slice(2) : [],
});
const dryRun = options["dry-run"];
const [registry] = positionals;
const selected = options.targets?.split(",") ?? targets.map((t) => t.target);
const expected = npmPackages.filter(
  (pkg) => pkg.target === undefined || selected.includes(pkg.target),
);

if (import.meta.main) {
  if (registry === "npm") publishNpm();
  else if (registry === "marketplace") publishMarketplace();
  else if (registry === "tag" && positionals[1] !== undefined) tag(positionals[1]);
  else if (
    registry === "catch-up" &&
    positionals[1] !== undefined &&
    positionals[2] !== undefined
  ) {
    const caughtUp = catchUp(packageNames(), positionals[1], positionals[2], registryNpm(dryRun));
    setOutput("moved", String(caughtUp !== undefined));
    if (caughtUp !== undefined) setOutput("version", caughtUp);
  } else if (registry === "missing") listMissing();
  else {
    fail(
      "usage: publish.ts npm|marketplace [--from <dir>] [--dry-run] | tag <tag> | catch-up <from> <to> | missing",
    );
  }
}

/** The checkout's version, which `npm`, `marketplace`, and `tag` publish or tag. */
function checkoutVersion(): string {
  const { version, problems } = checkVersion();
  if (problems.length > 0) fail(problems.join("\n"));
  return version;
}

function packageNames(): string[] {
  return expected.map((pkg) => pkg.name);
}

function publishNpm(): void {
  const version = checkoutVersion();
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
  const tag = options.tag ?? (version.includes("-") ? "next" : "latest");
  for (const { name } of expected) {
    const tarball = tarballs.get(name);
    if (tarball === undefined) fail(`no tarball for ${name} in ${dir}`);
    if (isPublished(name, version)) {
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

function tag(name: string): void {
  const version = checkoutVersion();
  // A version published a moment ago can take a while to show: one canary's
  // linux-arm64 package showed 54 minutes after the others (RELEASING.md, the
  // nightly canary). This waits a while, well inside the job's timeout, and a
  // later run's catch-up moves the tag if it gives up.
  const moved = moveTag(packageNames(), version, name, registryNpm(dryRun), {
    wait: dryRun ? 0 : 15 * 60_000,
  });
  setOutput("moved", String(moved));
  if (!moved) {
    log(
      `::warning::Not every package shows ${version} on npm yet, so \`${name}\` hasn't moved on any of them. ` +
        "The next canary run moves it once they all show, or run the Canary workflow by hand.",
    );
  }
}

/** What the tag commands ask of npm, so a test can stand in for it. */
export interface Npm {
  /** Whether npm shows this version of a package. */
  shows(name: string, version: string): boolean;
  /** The version a dist-tag names on a package, if any. */
  tagged(name: string, tag: string): string | undefined;
  /** Whether this version of a package is deprecated. */
  deprecated(name: string, version: string): boolean;
  addTag(name: string, version: string, tag: string): void;
  /** Milliseconds since the epoch. */
  now(): number;
  sleep(ms: number): void;
}

/**
 * Points `tag` at `version` of every package, once npm shows all of them; one
 * still missing after `wait` milliseconds leaves every package's `tag` where
 * it was. Moving them only together is what keeps `next` from mixing two
 * canaries. Returns whether it moved them.
 */
export function moveTag(
  names: readonly string[],
  version: string,
  tag: string,
  npm: Npm,
  { wait, poll = 10_000 }: { wait: number; poll?: number },
): boolean {
  const deadline = npm.now() + wait;
  let missing = names.filter((name) => !npm.shows(name, version));
  while (missing.length > 0) {
    if (npm.now() >= deadline) {
      log(`${missing.join(", ")} still ${missing.length === 1 ? "isn't" : "aren't"} on npm`);
      return false;
    }
    log(`waiting for ${missing.join(", ")} @${version} to show on npm`);
    npm.sleep(poll);
    missing = missing.filter((name) => !npm.shows(name, version));
  }
  for (const name of names) {
    if (npm.tagged(name, tag) === version) continue;
    log(`npm dist-tag add ${name}@${version} ${tag}`);
    npm.addTag(name, version, tag);
  }
  return true;
}

/**
 * Moves `to` to the version `from` names on @ascribed/cli (the canary a run
 * published), when that's a different version, npm shows it for every
 * package now, and it isn't deprecated: a bad canary, deprecated and with
 * `next` pointed back, stays behind, and so does `to` where it names a newer
 * version. Returns the version it moved to, if it did.
 */
export function catchUp(
  names: readonly string[],
  from: string,
  to: string,
  npm: Npm,
): string | undefined {
  const version = npm.tagged("@ascribed/cli", from);
  if (version === undefined) {
    log(`@ascribed/cli has no \`${from}\`; nothing to catch up`);
    return undefined;
  }
  const current = names.map((name) => npm.tagged(name, to));
  if (current.every((tagged) => tagged === version)) {
    log(`\`${to}\` is already at ${version} on every package`);
    return undefined;
  }
  // `next` only moves forward (canary.ts counts on it): a pre-release from
  // release.yml takes `next` without touching `next-pending`.
  const ahead = current.find(
    (tagged) => tagged !== undefined && compareVersions(tagged, version) > 0,
  );
  if (ahead !== undefined) {
    log(`\`${to}\` is at ${ahead}, newer than ${version}; it stays where it is`);
    return undefined;
  }
  if (names.some((name) => npm.shows(name, version) && npm.deprecated(name, version))) {
    log(`${version} is deprecated; \`${to}\` stays where it is`);
    return undefined;
  }
  if (moveTag(names, version, to, npm, { wait: 0 })) return version;
  log(`\`${to}\` stays where it is until every package shows ${version}`);
  return undefined;
}

/**
 * Orders two versions by semver precedence: a pre-release before its release,
 * and pre-release identifiers one by one, numbers by value and before words.
 */
export function compareVersions(a: string, b: string): number {
  const [aCore, aPre] = splitVersion(a);
  const [bCore, bPre] = splitVersion(b);
  const aParts = aCore.split(".").map(Number);
  const bParts = bCore.split(".").map(Number);
  for (let i = 0; i < 3; i++) {
    const diff = (aParts[i] ?? 0) - (bParts[i] ?? 0);
    if (diff !== 0) return Math.sign(diff);
  }
  if (aPre === undefined || bPre === undefined) {
    return aPre === bPre ? 0 : aPre === undefined ? 1 : -1;
  }
  const aIds = aPre.split(".");
  const bIds = bPre.split(".");
  for (let i = 0; i < Math.max(aIds.length, bIds.length); i++) {
    const x = aIds[i];
    const y = bIds[i];
    if (x === undefined || y === undefined) return x === undefined ? -1 : 1;
    if (x === y) continue;
    const xNumber = /^\d+$/.test(x);
    const yNumber = /^\d+$/.test(y);
    if (xNumber && yNumber) return Math.sign(Number(x) - Number(y));
    if (xNumber !== yNumber) return xNumber ? -1 : 1;
    return x < y ? -1 : 1;
  }
  return 0;
}

function splitVersion(version: string): [string, string | undefined] {
  const dash = version.indexOf("-");
  return dash === -1 ? [version, undefined] : [version.slice(0, dash), version.slice(dash + 1)];
}

/** npm itself. With `dryRun`, it moves no tag. */
function registryNpm(dryRun: boolean): Npm {
  return {
    shows: isPublished,
    tagged(name, tag) {
      const result = spawnSync("npm", ["view", name, "dist-tags", "--json", "--prefer-online"], {
        encoding: "utf8",
        shell: process.platform === "win32",
      });
      if (result.status !== 0) fail(`couldn't ask npm about ${name}: ${result.stderr.trim()}`);
      return (JSON.parse(result.stdout) as Record<string, string>)[tag];
    },
    deprecated(name, version) {
      const result = spawnSync(
        "npm",
        ["view", `${name}@${version}`, "deprecated", "--prefer-online"],
        { encoding: "utf8", shell: process.platform === "win32" },
      );
      if (result.status !== 0) fail(`couldn't ask npm about ${name}: ${result.stderr.trim()}`);
      return result.stdout.trim() !== "";
    },
    addTag(name, version, tag) {
      if (!dryRun) run("npm", ["dist-tag", "add", `${name}@${version}`, tag]);
    },
    now: () => Date.now(),
    sleep(ms) {
      Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, ms);
    },
  };
}

/** Prints each package npm has no version of. */
function listMissing(): void {
  for (const { name } of expected) {
    const result = spawnSync("npm", ["view", name, "name"], {
      encoding: "utf8",
      shell: process.platform === "win32",
    });
    if (result.status === 0) continue;
    if (/E404|404 Not Found/.test(result.stderr)) log(name);
    else fail(`couldn't ask npm about ${name}: ${result.stderr.trim()}`);
  }
}

/** Whether npm already has this version of a package. */
function isPublished(name: string, version: string): boolean {
  // `--prefer-online`: npm otherwise answers from its cache for some minutes,
  // so a check made before a publish would hide the version just published.
  const result = spawnSync("npm", ["view", `${name}@${version}`, "version", "--prefer-online"], {
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
  const version = checkoutVersion();
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

/** Sets a step output, when this runs in GitHub Actions. */
function setOutput(name: string, value: string): void {
  const output = process.env.GITHUB_OUTPUT;
  if (output !== undefined && output !== "") appendFileSync(output, `${name}=${value}\n`);
}

function log(message: string): void {
  process.stdout.write(`${message}\n`);
}

function fail(message: string): never {
  process.stderr.write(`error: ${message}\n`);
  process.exit(1);
}
