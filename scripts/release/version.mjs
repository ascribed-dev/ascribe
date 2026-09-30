// Sets or checks Ascribe's one version.
//
//   node scripts/release/version.mjs 0.2.0     set it everywhere
//   node scripts/release/version.mjs --check   check every file agrees, and
//                                              the changelog has the version
//   node scripts/release/version.mjs --check --tag v0.2.0
//                                              also check the tag matches
//   node scripts/release/version.mjs --notes   print the changelog section,
//                                              for the GitHub release
//
// Setting it also sets the extension's `ascribe.minServerVersion`: the
// extension warns about a project binary older than the release it ships
// with. Lower it by hand if a release keeps working with older binaries.
import { execFileSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import process from "node:process";
import {
  VERSION,
  cargoVersion,
  changelogSection,
  checkVersion,
  extension,
  npmPackages,
  readJson,
  root,
  writeJson,
} from "./manifests.mjs";

const args = process.argv.slice(2);

if (args[0] === "--check") {
  const tagAt = args.indexOf("--tag");
  const { version, problems } = checkVersion(tagAt === -1 ? undefined : args[tagAt + 1]);
  if (problems.length > 0) {
    for (const problem of problems) process.stderr.write(`error: ${problem}\n`);
    process.exit(1);
  }
  process.stdout.write(`${version}\n`);
} else if (args[0] === "--notes") {
  const version = cargoVersion();
  const notes = changelogSection(version);
  if (notes === undefined) {
    process.stderr.write(`error: CHANGELOG.md has no "## ${version}" section\n`);
    process.exit(1);
  }
  process.stdout.write(`${notes}\n`);
} else if (args.length === 1 && VERSION.test(args[0])) {
  const version = args[0];
  const cargo = join(root, "Cargo.toml");
  const toml = readFileSync(cargo, "utf8");
  const updated = toml.replace(
    /(\[workspace\.package\][^[]*?\nversion = ")[^"]+(")/,
    `$1${version}$2`,
  );
  writeFileSync(cargo, updated);
  // Rewrites the workspace crates' entries in Cargo.lock, and nothing else.
  execFileSync("cargo", ["update", "--workspace", "--offline"], { cwd: root, stdio: "inherit" });
  for (const { dir } of [...npmPackages, extension]) {
    const file = `${dir}/package.json`;
    const manifest = readJson(file);
    manifest.version = version;
    if (dir === extension.dir) manifest.ascribe.minServerVersion = version;
    writeJson(file, manifest);
  }
  process.stdout.write(`Set the version to ${version}.\n`);
  if (changelogSection(version) === undefined) {
    process.stdout.write(`Add a "## ${version}" section to CHANGELOG.md before releasing.\n`);
  }
} else {
  process.stderr.write(
    "usage: node scripts/release/version.mjs <version> | --check [--tag v<version>] | --notes\n",
  );
  process.exit(2);
}
