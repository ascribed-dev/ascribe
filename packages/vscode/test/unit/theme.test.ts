// The extension's native UI takes its colors from the user's theme and its
// icons from codicons. Colors it names are theme colors with a default for
// every kind of theme, and icons of its own are declared before they're used.
// DEVELOPMENT.md says which codicon stands for what.
import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const root = fileURLToPath(new URL("../..", import.meta.url));
const src = join(root, "src");

interface Manifest {
  contributes: {
    colors?: { id: string; description?: string; defaults?: Record<string, string> }[];
    icons?: Record<string, unknown>;
  };
}

const manifestText = readFileSync(join(root, "package.json"), "utf8");
const manifest = JSON.parse(manifestText) as Manifest;

const sources = (readdirSync(src, { recursive: true }) as string[])
  .filter((file) => file.endsWith(".ts"))
  .map((file) => ({
    file: file.split("\\").join("/"),
    text: readFileSync(join(src, file), "utf8"),
  }));

/** The colors `text` writes, outside comments, as `file:line: …`. */
function colorLiterals(file: string, text: string): string[] {
  // Blank comments, keeping lines, so "#128" in a comment isn't a color.
  const code = text
    .replace(/\/\*[\s\S]*?\*\//g, (comment) => comment.replace(/[^\n]/g, " "))
    .replace(/(^|[^:"'`\\])\/\/.*$/gm, "$1");
  const found: string[] = [];
  code.split("\n").forEach((line, index) => {
    const color = /#(?:[0-9a-f]{8}|[0-9a-f]{6}|[0-9a-f]{3,4})\b|\b(?:rgba?|hsla?|oklch)\(/i.exec(
      line,
    );
    if (color !== null) found.push(`${file}:${index + 1}: ${color[0]}`);
  });
  return found;
}

describe("colors", () => {
  it("are never written in the extension's code; the native UI takes theme colors", () => {
    // The webview's stylesheet, preview.css, is held to design/tokens.toml by
    // scripts/design/tokens.test.ts.
    const found = sources.flatMap(({ file, text }) => colorLiterals(file, text));
    expect(found, "Use a vscode.ThemeColor, or declare one in contributes.colors").toEqual([]);
  });

  it("finds a color in code, by file and line, and not in a comment", () => {
    const text = [
      "// Pull request #128.",
      "/* #fff */",
      'const a = "#0969da";',
      "const b = `rgb(0 0 0)`;",
      'const c = "https://example.com/"; // #fff',
    ].join("\n");
    expect(colorLiterals("x.ts", text)).toEqual(["x.ts:3: #0969da", "x.ts:4: rgb("]);
  });

  it("each declared color has a description and a default for every kind of theme", () => {
    for (const color of manifest.contributes.colors ?? []) {
      expect(color.id, color.id).toMatch(/^ascribe\.[a-zA-Z.]+$/);
      expect(color.description, color.id).toBeTruthy();
      expect(Object.keys(color.defaults ?? {}).sort(), color.id).toEqual([
        "dark",
        "highContrast",
        "highContrastLight",
        "light",
      ]);
    }
  });
});

it("every Ascribe icon used, $(ascribe-…), is declared in contributes.icons", () => {
  const declared = new Set(Object.keys(manifest.contributes.icons ?? {}));
  const used = [manifestText, ...sources.map(({ text }) => text)].flatMap((text) =>
    [...text.matchAll(/\$\((ascribe-[\w-]+)(?:~[\w-]+)?\)/g)].map((match) => match[1]),
  );
  expect(used.filter((name) => name !== undefined && !declared.has(name))).toEqual([]);
});
