// The generated reference pages write the docs' settled terms (#1283, #1361),
// so an old one doesn't come back through the code's own text: the commands'
// help, diagnostics.toml, the content model's settings, the JSON Schemas'
// descriptions, and the editor's actions. The terms are the swaps of the docs'
// Vale rule, `docs/.vale/Docs/Terms.yml`, matched as Vale matches them: whole
// words, case kept, outside code, bold text, and link destinations. `.vale.ini`
// leaves `_generated/` out of that rule, since a word there is changed at its
// source, which this test names. Two kinds of text are left out: what's quoted
// from SPEC §8.2's table, since SPEC is a contract whose words change only by
// a decision, and a diagnostic's message, which is the program's output, left
// as it is on purpose (#1353, #1361).
import { readdirSync, readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { expect, test } from "vitest";

const root = fileURLToPath(new URL("../..", import.meta.url));
const generated = path.join(root, "docs/content/_generated");

/** The old terms, each with the term to write, from the rule's `swap:` map. */
function swaps(): { old: RegExp; term: string }[] {
  const rule = readFileSync(path.join(root, "docs/.vale/Docs/Terms.yml"), "utf8");
  const map = rule.split(/^swap:\n/m)[1] ?? "";
  const found: { old: RegExp; term: string }[] = [];
  for (const line of map.split("\n")) {
    const entry = /^ {2}("[^"]*"|'[^']*'|[^:]+): (.+)$/.exec(line);
    if (entry === null) continue;
    const [, key = "", term = ""] = entry;
    const pattern = /^["']/.test(key) ? key.slice(1, -1) : key;
    found.push({ old: new RegExp(`\\b(?:${pattern})\\b`), term });
  }
  return found;
}

/** The conditions of SPEC §8.2's table, which the diagnostics reference quotes. */
function specConditions(): string[] {
  const spec = readFileSync(path.join(root, "SPEC.md"), "utf8");
  const section = spec.split(/^### 8\.2 /m)[1]?.split(/^### /m)[0] ?? "";
  return section
    .split("\n")
    .filter((line) => line.startsWith("| ") && !line.startsWith("| Construct"))
    .map((line) => line.split(" | ")[1] ?? "")
    .filter((condition) => condition !== "");
}

/** A page's prose: its lines outside fences, without what Vale skips. */
function prose(text: string, quoted: string[]): string[] {
  let fenced = false;
  return text.split("\n").map((line) => {
    if (/^\s*(`{3,}|~{3,})/.test(line)) {
      fenced = !fenced;
      return "";
    }
    if (fenced || line.startsWith("**Message:** ")) return "";
    let kept = line
      .replace(/`[^`]*`/g, "")
      .replace(/\*\*[^*]*\*\*/g, "")
      .replace(/\]\([^)]*\)/g, "]");
    for (const condition of quoted) kept = kept.split(condition.replace(/`[^`]*`/g, "")).join("");
    return kept;
  });
}

test("the terms rule has swaps to look for", () => {
  expect(swaps().length).toBeGreaterThan(5);
  expect(specConditions().length).toBeGreaterThan(50);
});

test("generated pages write the settled terms", () => {
  const rules = swaps();
  const quoted = specConditions();
  const found: string[] = [];
  for (const file of readdirSync(generated)
    .filter((name) => name.endsWith(".md"))
    .sort()) {
    const lines = prose(readFileSync(path.join(generated, file), "utf8"), quoted);
    lines.forEach((line, i) => {
      for (const { old, term } of rules) {
        const match = old.exec(line);
        if (match !== null) found.push(`${file}:${i + 1}: write '${term}', not '${match[0]}'`);
      }
    });
  }
  expect(found, "change each at its source, named in the page's @memo, and re-bless").toEqual([]);
});
