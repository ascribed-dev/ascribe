// The guides for implementors name files and folders by path, and a path that
// no longer exists sends a reader the wrong way. Each path in a code span, and
// each relative link, must exist in the checkout. The change checklists must
// also name every request the language server handles.
import { execFileSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { expect, test } from "vitest";

const root = fileURLToPath(new URL("../..", import.meta.url));

/** The files whose paths are checked, relative to the root. */
const FILES = ["project-docs/decisions.md", "project-docs/checklists.md"];

/** The names at the root of the repository, which a path in a code span starts with. */
function topLevel(): Set<string> {
  const files = execFileSync("git", ["ls-files"], { cwd: root, encoding: "utf8" });
  return new Set(files.split("\n").map((file) => file.split("/")[0] ?? ""));
}

/**
 * The code spans that are paths in the repository: no spaces or wildcards, a
 * slash, and a first segment that's at the root (so `ascribe/preview`, a
 * method name, isn't one).
 */
function codePaths(markdown: string, roots: Set<string>): string[] {
  return Array.from(markdown.matchAll(/`([^`\s]+)`/g), (match) => match[1] ?? "").filter(
    (span) =>
      /^[\w.-]+(\/[\w.-]+)*\/?$/.test(span) &&
      span.includes("/") &&
      roots.has(span.split("/")[0] ?? ""),
  );
}

/** A Markdown link's destinations that are files in the repository, without fragments. */
function relativeLinks(markdown: string): string[] {
  return Array.from(markdown.matchAll(/\]\(([^)\s]+)\)/g), (match) => match[1] ?? "")
    .filter((link) => !/^[a-z]+:/.test(link) && !link.startsWith("#"))
    .map((link) => link.replace(/#.*$/, ""));
}

test("every path the guides name exists", () => {
  const roots = topLevel();
  const missing: string[] = [];
  for (const file of FILES) {
    const markdown = readFileSync(path.join(root, file), "utf8");
    for (const span of codePaths(markdown, roots)) {
      if (!existsSync(path.join(root, span))) missing.push(`${file}: \`${span}\``);
    }
    for (const link of relativeLinks(markdown)) {
      if (!existsSync(path.join(root, path.dirname(file), link))) missing.push(`${file}: ${link}`);
    }
  }
  expect(missing).toEqual([]);
});

test("the checklists name every request the language server handles", () => {
  const server = readFileSync(path.join(root, "crates/tessera-lsp/src/server.rs"), "utf8");
  const handler = server.slice(server.indexOf("fn handle_request("));
  const arms = handler.slice(0, handler.indexOf("\n}\n"));
  const requests = Array.from(
    arms.matchAll(/(?<![:\w])(\w+)::METHOD =>/g),
    (match) => match[1] ?? "",
  );
  // Ascribe's own requests are constants in their modules, such as `crate::preview::METHOD`.
  for (const [, module = "", constant = ""] of arms.matchAll(/crate::(\w+)::(\w+) =>/g)) {
    const source = readFileSync(path.join(root, `crates/tessera-lsp/src/${module}.rs`), "utf8");
    const value = new RegExp(`const ${constant}: &str = "([^"]+)"`).exec(source)?.[1];
    requests.push(value ?? `crate::${module}::${constant}`);
  }
  expect(requests.length).toBeGreaterThan(10);

  const checklists = readFileSync(path.join(root, "project-docs/checklists.md"), "utf8");
  const missing = requests.filter((request) => !checklists.includes(`\`${request}\``));
  expect(missing, "add each to the language server's list in checklists.md").toEqual([]);
});
