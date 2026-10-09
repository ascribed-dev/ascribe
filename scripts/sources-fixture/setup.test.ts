// The update pull request's recipe is one file,
// examples/docs-repository/.github/workflows/update-sources.yml: the drift
// guide shows it, and the sources fixture runs it. These tests check that
// both use that file, and, when asked, run the fixture's pass on this machine:
// with ASCRIBE_BIN naming the binary, or ASCRIBE_SOURCES_FIXTURE=1 for this
// repository's debug build. A plain `pnpm test` skips the pass, whether or not
// that build exists: it takes ten seconds, and rust.yml runs it after its
// build (`node scripts/sources-fixture/setup.ts --local`).
import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { expect, test } from "vitest";

const root = fileURLToPath(new URL("../..", import.meta.url));
const workflow = path.join(
  "examples",
  "docs-repository",
  ".github",
  "workflows",
  "update-sources.yml",
);
const built = path.join(
  root,
  "target",
  "debug",
  process.platform === "win32" ? "ascribe.exe" : "ascribe",
);
const ascribe =
  process.env["ASCRIBE_BIN"] ??
  (process.env["ASCRIBE_SOURCES_FIXTURE"] === "1" ? built : undefined);

test("the drift guide shows the workflow the fixture runs", () => {
  const guide = readFileSync(path.join(root, "docs", "content", "guides", "drift.md"), "utf8");
  // docs/ascribe.toml's `code` source is the repository's root.
  expect(guide).toContain(
    "@snippet {lang=yaml}: code:examples/docs-repository/.github/workflows/update-sources.yml#workflow\n",
  );
  const text = readFileSync(path.join(root, workflow), "utf8");
  expect(text).toContain("# :snippet-start: workflow\n");
  expect(text).toContain("# :snippet-end:\n");
});

test.skipIf(ascribe === undefined || process.platform === "win32")(
  "the fixture runs the guide's workflow, and its pass is right",
  { timeout: 120_000 },
  () => {
    const dir = mkdtempSync(path.join(tmpdir(), "ascribe-sources-fixture-test-"));
    try {
      const result = spawnSync(
        process.execPath,
        [path.join(root, "scripts", "sources-fixture", "setup.ts"), "--local", "--dir", dir],
        { encoding: "utf8", env: { ...process.env, ASCRIBE_BIN: ascribe } },
      );
      expect(result.status, `${result.stdout}${result.stderr}`).toBe(0);
      expect(result.stdout).not.toContain("WRONG");
      const fixture = path.join(dir, "docs", ".github", "workflows", "update-sources.yml");
      expect(readFileSync(fixture, "utf8")).toBe(readFileSync(path.join(root, workflow), "utf8"));
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  },
);
