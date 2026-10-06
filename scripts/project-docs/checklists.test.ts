// The change checklists must name every request the language server handles.
// (Whether the paths they name exist is checked with the other guides', in
// scripts/repo-docs/paths.test.ts.)
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { expect, test } from "vitest";

const root = fileURLToPath(new URL("../..", import.meta.url));

test("the checklists name every request the language server handles", () => {
  const server = readFileSync(path.join(root, "crates/ascribe-lsp/src/server.rs"), "utf8");
  const handler = server.slice(server.indexOf("fn handle_request("));
  const arms = handler.slice(0, handler.indexOf("\n}\n"));
  const requests = Array.from(
    arms.matchAll(/(?<![:\w])(\w+)::METHOD =>/g),
    (match) => match[1] ?? "",
  );
  // Ascribe's own requests are constants in their modules, such as `crate::preview::METHOD`.
  for (const [, module = "", constant = ""] of arms.matchAll(/crate::(\w+)::(\w+) =>/g)) {
    const source = readFileSync(path.join(root, `crates/ascribe-lsp/src/${module}.rs`), "utf8");
    const value = new RegExp(`const ${constant}: &str = "([^"]+)"`).exec(source)?.[1];
    requests.push(value ?? `crate::${module}::${constant}`);
  }
  expect(requests.length).toBeGreaterThan(10);

  const checklists = readFileSync(path.join(root, "project-docs/checklists.md"), "utf8");
  const missing = requests.filter((request) => !checklists.includes(`\`${request}\``));
  expect(missing, "add each to the language server's list in checklists.md").toEqual([]);
});
