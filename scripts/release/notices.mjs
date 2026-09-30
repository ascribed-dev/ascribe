// Writes THIRD-PARTY-NOTICES: the license text of every Rust crate compiled
// into the `ascribe` binary, which the binary's archives and packages must
// carry alongside Ascribe's own LICENSE.
//
//   node scripts/release/notices.mjs [--out <file>]
//
// The crate list is what Cargo resolves for `tessera-cli`'s normal
// dependencies, so a crate that's only a build tool or a test dependency isn't
// listed. The vendored comrak fork is. Crates that share a license text are
// listed together under one copy of it.
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";
import { root } from "./manifests.mjs";

const LICENSE_FILE = /^(licen[cs]e|copying|unlicense|notice)(?![a-z])/i;
/** In-repo crates that aren't Ascribe's own code. */
const VENDORED = new Set(["comrak-tessera"]);

/** The notices for the `ascribe` binary, as text. */
export function thirdPartyNotices() {
  const metadata = JSON.parse(
    execFileSync("cargo", ["metadata", "--format-version", "1", "--locked"], {
      cwd: root,
      encoding: "utf8",
      maxBuffer: 256 * 1024 * 1024,
    }),
  );
  const packages = new Map(metadata.packages.map((p) => [p.id, p]));
  const nodes = new Map(metadata.resolve.nodes.map((n) => [n.id, n]));
  const start = metadata.packages.find((p) => p.name === "tessera-cli");
  if (!start) throw new Error("the workspace has no tessera-cli package");

  const seen = new Set();
  const visit = (id) => {
    if (seen.has(id)) return;
    seen.add(id);
    for (const dep of nodes.get(id)?.deps ?? []) {
      if (dep.dep_kinds.some((k) => k.kind === null)) visit(dep.pkg);
    }
  };
  visit(start.id);

  const workspace = new Set(metadata.workspace_members);
  const groups = new Map();
  for (const id of seen) {
    const pkg = packages.get(id);
    if (workspace.has(id) && !VENDORED.has(pkg.name)) continue;
    const dir = dirname(pkg.manifest_path);
    const texts = readdirSync(dir)
      .filter((name) => LICENSE_FILE.test(name))
      .sort()
      .map((name) => readFileSync(join(dir, name), "utf8").trim())
      .filter((text) => text !== "");
    const text = texts.join("\n\n---\n\n");
    const key = createHash("sha256").update(text).digest("hex");
    if (!groups.has(key)) groups.set(key, { text, crates: [] });
    groups.get(key).crates.push(pkg);
  }

  const sections = [...groups.values()]
    .map((group) => ({
      ...group,
      crates: group.crates.sort((a, b) => a.name.localeCompare(b.name)),
    }))
    .sort((a, b) => a.crates[0].name.localeCompare(b.crates[0].name))
    .map(({ text, crates }) => {
      const list = crates
        .map((c) => `  ${c.name} ${c.version} (${c.license ?? "see its repository"})`)
        .join("\n");
      const body =
        text === "" ? "(These crates ship no license file; see their repositories.)" : text;
      return `${list}\n\n${body}`;
    });

  return [
    "Ascribe's binary includes the following third-party software.",
    "Ascribe itself is licensed under the MPL-2.0; see LICENSE.",
    "",
    sections.join(`\n\n${"=".repeat(72)}\n\n`),
    "",
  ].join("\n");
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const { values } = parseArgs({ options: { out: { type: "string" } } });
  const text = thirdPartyNotices();
  if (values.out) writeFileSync(values.out, text);
  else process.stdout.write(text);
}
