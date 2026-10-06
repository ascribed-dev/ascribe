// Readers are sent to the docs site, not to the docs' source files: the
// READMEs and the extension's manifest link to the site's pages, at the
// address in docs/ascribe.toml's `[consumer] site`, and each link lands on a
// page and a heading the docs have. (The binary's help is checked by
// crates/ascribe-cli/src/docs.rs, and the site's own links by site/test/.)
import { execFileSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { expect, test } from "vitest";

const root = fileURLToPath(new URL("../..", import.meta.url));
const content = path.join(root, "docs", "content");

/** `[consumer] site` in docs/ascribe.toml, without a trailing slash. */
function siteAddress(): string {
  const config = readFileSync(path.join(root, "docs", "ascribe.toml"), "utf8");
  const table = /^\[consumer\]$([\s\S]*?)(?=^\[|(?![\s\S]))/m.exec(config)?.[1] ?? "";
  const site = /^site\s*=\s*"([^"]+)"/m.exec(table)?.[1];
  if (site === undefined) throw new Error("docs/ascribe.toml's [consumer] has no site");
  return site.replace(/\/$/, "");
}

/** Every README in the repository, outside the docs themselves. */
function readmes(): string[] {
  return execFileSync("git", ["ls-files", "README.md", "**/README.md"], {
    cwd: root,
    encoding: "utf8",
  })
    .split("\n")
    .filter((file) => file !== "" && !file.startsWith("docs/"));
}

/** A Markdown link's destinations: `[text](destination)`. */
function destinations(markdown: string): string[] {
  return Array.from(markdown.matchAll(/\]\(([^)\s]+)/g), (match) => match[1] ?? "");
}

/** A page's source file for its route on the site (`/guides/astro/` is `guides/astro.md`). */
function pageFor(route: string): string | undefined {
  const name = route.replace(/^\/|\/$/g, "");
  const candidates = name === "" ? ["index.md"] : [`${name}.md`, `${name}/index.md`];
  return candidates.map((file) => path.join(content, file)).find((file) => existsSync(file));
}

/** The anchors of a page's headings, as the site's slugger writes them. */
function anchors(file: string): Set<string> {
  const headings = readFileSync(file, "utf8").matchAll(/^#{1,6}\s+(.+?)\s*$/gm);
  return new Set(
    Array.from(headings, ([, heading = ""]) =>
      heading
        .toLowerCase()
        .replace(/[^\p{L}\p{N}\s_-]/gu, "")
        .replace(/ /g, "-"),
    ),
  );
}

test("READMEs link to the docs site, not to the docs' source files", () => {
  const source = /(^|\/)docs\/content\/(?![^#]*\/_)[^#]*\.md(#|$)/;
  const found = readmes().flatMap((file) =>
    destinations(readFileSync(path.join(root, file), "utf8"))
      .filter((link) => source.test(link))
      .map((link) => `${file}: ${link}`),
  );
  expect(found, "link to the page on the docs site instead").toEqual([]);
});

test("each link to the docs site lands on a page and a heading", () => {
  const site = siteAddress();
  const problems: string[] = [];
  let checked = 0;
  for (const file of readmes()) {
    for (const link of destinations(readFileSync(path.join(root, file), "utf8"))) {
      if (!link.startsWith(`${site}/`)) continue;
      checked++;
      const url = new URL(link);
      const page = pageFor(url.pathname);
      if (page === undefined) problems.push(`${file}: ${link} (no page)`);
      else if (url.hash !== "" && !anchors(page).has(decodeURIComponent(url.hash.slice(1)))) {
        problems.push(`${file}: ${link} (no heading)`);
      }
    }
  }
  expect(problems).toEqual([]);
  expect(checked).toBeGreaterThan(10);
});

test("the extension's homepage is a page of the docs site", () => {
  const manifest = JSON.parse(
    readFileSync(path.join(root, "packages", "vscode", "package.json"), "utf8"),
  ) as { homepage: string };
  expect(manifest.homepage.startsWith(`${siteAddress()}/`)).toBe(true);
  expect(pageFor(new URL(manifest.homepage).pathname)).toBeDefined();
});
