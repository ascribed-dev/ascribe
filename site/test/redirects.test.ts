// The built site's `_redirects`, which `[consumer] host = "netlify"` in
// ../docs/ascribe.toml has @ascribed/astro write: each permalink's address
// leads to its section with a 301, forced past its forwarding page, which is
// served instead for an address with a dimension's parameter, so the page's
// script takes the reader's choice from it.
import { readFile } from "node:fs/promises";
import path from "node:path";
import { expect, test } from "vitest";
import { distDir, outputDir, requireBuild } from "./built.ts";

test("a permalink's address leads to its section, and to its page with a parameter", async () => {
  requireBuild();
  const lines = (await readFile(path.join(distDir, "_redirects"), "utf8")).split("\n");
  const list = JSON.parse(
    await readFile(path.join(outputDir, "_ascribe", "permalinks.json"), "utf8"),
  ) as { permalinks: Record<string, { route: string; fragment?: string }> };
  const entry = list.permalinks["ASC036"];
  expect(entry?.fragment).toBe("ASC036");
  const at = lines.indexOf(`/go/ASC036/ ${entry?.route ?? ""}#ASC036 301!`);
  expect(at).toBeGreaterThan(-1);
  // Before it, since Netlify uses the first rule that matches.
  expect(lines.indexOf("/go/ASC036/ pm=:pm /go/ASC036/ 200!")).toBeLessThan(at);
});
