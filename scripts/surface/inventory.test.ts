// The map of Ascribe's surface is what its sources say. A stale
// project-docs/surface.md fails here; run with ASCRIBE_BLESS=1 (or
// `node scripts/surface/inventory.ts`) to rewrite it, then read the diff: it's
// what the change added to the surface, or took from it.
import { readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { expect, test } from "vitest";
import { generate, SURFACE } from "./inventory.ts";

const root = fileURLToPath(new URL("../..", import.meta.url));

test("the surface map is what its sources say", () => {
  const file = path.join(root, SURFACE);
  const expected = generate();
  if (process.env["ASCRIBE_BLESS"] === "1") writeFileSync(file, expected);
  expect(
    readFileSync(file, "utf8") === expected,
    `${SURFACE} is stale: run \`node scripts/surface/inventory.ts\`, then read its diff`,
  ).toBe(true);
});

test("the map has a section for each surface", () => {
  const headings = Array.from(generate().matchAll(/^## (.+)$/gm), (found) => found[1]);
  expect(headings).toEqual([
    "The command line",
    "The language",
    "`ascribe.toml`",
    "Diagnostics",
    "The VS Code extension",
    "The language server",
    "The MCP server",
    "VS Code's agent tools",
    "The plugin",
    "`@ascribed/astro`",
    "`@ascribed/elements`, `@ascribed/review`, and `@ascribed/cli`",
    "The outputs",
    "The docs",
    "What nothing declares",
  ]);
});
