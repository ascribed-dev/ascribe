// `pnpm bless` runs every rewrite ARCHITECTURE.md's tables name, and nothing
// else; a test's command is the one the test itself says rewrites it; and the
// order lets each rewrite see what the ones it reads wrote.
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { expect, test } from "vitest";
import { command, rewrites } from "./bless.ts";
import { writers } from "./repo-docs/generated.ts";

const root = fileURLToPath(new URL("..", import.meta.url));

test("the rewrites are the rows of ARCHITECTURE.md's tables, each once", () => {
  const named = rewrites.map((rewrite) => rewrite.writer);
  expect(new Set(named).size).toBe(named.length);
  expect([...named].sort()).toEqual(
    writers()
      .map((writer) => writer.name)
      .sort(),
  );
});

test("each runs, or says why not", () => {
  for (const rewrite of rewrites) {
    expect((rewrite.run === undefined) !== (rewrite.skip === undefined), rewrite.writer).toBe(true);
  }
});

test("a test's command is the one the test says rewrites it", () => {
  for (const rewrite of rewrites) {
    const file = path.join(root, rewrite.writer);
    if (!existsSync(file)) continue;
    const said = /\bconst BLESS(?:: &str)? = "([^"]+)"/.exec(readFileSync(file, "utf8"))?.[1];
    if (said !== undefined) expect(rewrite.run, rewrite.writer).toBe(said);
  }
});

test("a test runs with ASCRIBE_BLESS=1, and a command without it", () => {
  for (const rewrite of rewrites) {
    if (rewrite.run === undefined) continue;
    const isTest = existsSync(path.join(root, rewrite.writer));
    expect(command(rewrite.run).env["ASCRIBE_BLESS"], rewrite.writer).toBe(
      isTest ? "1" : undefined,
    );
  }
});

test("a rewrite comes after the ones whose files it reads", () => {
  const at = (writer: string): number => rewrites.findIndex((r) => r.writer === writer);
  const fragments = writers().filter((writer) =>
    writer.paths.some((p) => p.startsWith("docs/content/_generated/")),
  );
  expect(fragments.length).toBeGreaterThan(0);
  for (const writer of fragments) {
    expect(at(writer.name), writer.name).toBeLessThan(at("scripts/surface/inventory.test.ts"));
  }
  const tokens = at("scripts/design/tokens.test.ts");
  expect(tokens).toBeLessThan(at("pnpm --filter @ascribed/review embed"));
  expect(tokens).toBeLessThan(at("scripts/docs-site/elements.test.ts"));
  expect(tokens).toBeLessThan(at("scripts/design/specimen.test.ts"));
  for (const source of ["crates/ascribe-core/tests/names.rs", "crates/ascribe-cli/src/shapes.rs"]) {
    expect(at(source)).toBeLessThan(at("pnpm --filter @ascribed/review embed"));
  }
  expect(at("crates/ascribe-cli/tests/all/output.rs")).toBeLessThan(at("ascribe observe"));
});

test("a command's environment is its leading assignments", () => {
  expect(command("ASCRIBE_BLESS=1 cargo test -p x --test all output::")).toEqual({
    env: { ASCRIBE_BLESS: "1" },
    argv: ["cargo", "test", "-p", "x", "--test", "all", "output::"],
  });
  expect(command("node scripts/design/assets.ts")).toEqual({
    env: {},
    argv: ["node", "scripts/design/assets.ts"],
  });
});
