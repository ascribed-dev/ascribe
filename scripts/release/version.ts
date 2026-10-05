// Sets or checks Ascribe's one version.
//
//   node scripts/release/version.ts 0.2.0     set it everywhere
//   node scripts/release/version.ts --check   check every file agrees, and
//                                              the changelog has the version
//   node scripts/release/version.ts --check --tag v0.2.0
//                                              also check the tag matches
//   node scripts/release/version.ts --notes   print the changelog section,
//                                              for the GitHub release
//   node scripts/release/version.ts --canary <n>
//                                              stamp a nightly canary's
//                                              version, and print it
//
// Setting it also sets the extension's `ascribe.minServerVersion`: the
// extension warns about a project binary older than the release it ships
// with. Lower it by hand if a release keeps working with older binaries.
//
// A canary is `<next>-next.<n>`: the version after the latest release (the
// one the changelog's unreleased section names, or a patch bump), and the
// workflow's run number. It's stamped in the working tree for a build and
// never committed, so it rewrites Cargo.lock itself, with no Cargo or network,
// and writes the commit into each npm package as its `gitHead`. A canary's
// version needs the changelog's unreleased section instead of its own.
import { execFileSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import process from "node:process";
import {
  VERSION,
  canaryVersion,
  cargoVersion,
  changelogSection,
  checkVersion,
  extension,
  npmPackages,
  readJson,
  relockWorkspace,
  root,
  writeJson,
} from "./manifests.ts";

const args = process.argv.slice(2);
const [first] = args;

if (first === "--check") {
  const tagAt = args.indexOf("--tag");
  const { version, problems } = checkVersion(tagAt === -1 ? undefined : args[tagAt + 1]);
  if (problems.length > 0) {
    for (const problem of problems) process.stderr.write(`error: ${problem}\n`);
    process.exit(1);
  }
  process.stdout.write(`${version}\n`);
} else if (first === "--notes") {
  const version = cargoVersion();
  const notes = changelogSection(version);
  if (notes === undefined) {
    process.stderr.write(`error: CHANGELOG.md has no "## ${version}" section\n`);
    process.exit(1);
  }
  process.stdout.write(`${notes}\n`);
} else if (first === "--canary" && args.length === 2) {
  const run = Number(args[1]);
  const released = cargoVersion();
  let version: string;
  try {
    version = canaryVersion(released, readFileSync(join(root, "CHANGELOG.md"), "utf8"), run);
  } catch (error) {
    process.stderr.write(`error: ${(error as Error).message}\n`);
    process.exit(1);
  }
  const commit = execFileSync("git", ["rev-parse", "HEAD"], { cwd: root, encoding: "utf8" }).trim();
  const lock = join(root, "Cargo.lock");
  writeFileSync(lock, relockWorkspace(readFileSync(lock, "utf8"), released, version));
  setVersion(version, commit);
  process.stdout.write(`${version}\n`);
} else if (args.length === 1 && first !== undefined && VERSION.test(first)) {
  const version = first;
  setVersion(version);
  // Rewrites the workspace crates' entries in Cargo.lock, and nothing else.
  execFileSync("cargo", ["update", "--workspace", "--offline"], { cwd: root, stdio: "inherit" });
  process.stdout.write(`Set the version to ${version}.\n`);
  if (changelogSection(version) === undefined) {
    process.stdout.write(`Add a "## ${version}" section to CHANGELOG.md before releasing.\n`);
  }
} else {
  process.stderr.write(
    "usage: node scripts/release/version.ts <version> | --check [--tag v<version>] | --notes | --canary <run>\n",
  );
  process.exit(2);
}

/**
 * Sets the version in Cargo.toml and every package.json, and, given the commit
 * a canary is built from, each npm package's `gitHead`.
 */
function setVersion(version: string, commit?: string): void {
  const cargo = join(root, "Cargo.toml");
  const toml = readFileSync(cargo, "utf8");
  const updated = toml.replace(
    /(\[workspace\.package\][^[]*?\nversion = ")[^"]+(")/,
    `$1${version}$2`,
  );
  writeFileSync(cargo, updated);
  for (const { dir } of [...npmPackages, extension]) {
    const file = `${dir}/package.json`;
    const manifest = readJson(file);
    manifest.version = version;
    if (dir === extension.dir) (manifest.ascribe ??= {}).minServerVersion = version;
    else if (commit !== undefined) manifest.gitHead = commit;
    writeJson(file, manifest);
  }
}
