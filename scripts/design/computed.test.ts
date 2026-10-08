// Proof that generating the stylesheets' variables from design/tokens.toml
// changed nothing: every declaration of every stylesheet, by the rule it sits
// in, with `light-dark()` and `prefers-color-scheme: dark` resolved for a
// light and a dark reader, against `computed.json`, recorded from `main`
// before the change. Run with ASCRIBE_BLESS=1 to record it.

import { readFileSync, writeFileSync } from "node:fs";
import { expect, test } from "vitest";
import { declarations, sheet } from "./css.ts";

const root = new URL("../../", import.meta.url);
const fixture = new URL("computed.json", import.meta.url);

const stylesheets = [
  "packages/elements/css/style.css",
  "packages/review/src/marks/marks.css",
  "packages/review/src/overlay/style.ts",
  "packages/review/src/report/report.css",
  "packages/astro/src/toolbar/style.ts",
  "site/src/styles/site.css",
  "packages/vscode/src/webview/preview.css",
];

type Scheme = "light" | "dark";

/** `light-dark(a, b)`'s `a` or `b`, wherever it appears. */
function pick(value: string, scheme: Scheme): string {
  const at = value.indexOf("light-dark(");
  if (at === -1) return value;
  let depth = 0;
  let comma = -1;
  let end = -1;
  for (let i = at + "light-dark".length; i < value.length; i++) {
    const c = value[i];
    if (c === "(") depth++;
    else if (c === ")" && --depth === 0) {
      end = i;
      break;
    } else if (c === "," && depth === 1 && comma === -1) comma = i;
  }
  const inner =
    scheme === "light"
      ? value.slice(at + "light-dark(".length, comma)
      : value.slice(comma + 1, end);
  return pick(value.slice(0, at) + inner.trim() + value.slice(end + 1), scheme);
}

/** One spelling per color: `#rrggbb`, and `rgb(r g b / a)`. */
function spell(value: string): string {
  return value
    .replace(/#([0-9a-f]{3})\b/gi, (_, h: string) => `#${h.replace(/./g, "$&$&")}`.toLowerCase())
    .replace(/#[0-9a-f]{6}\b/gi, (h) => h.toLowerCase())
    .replace(
      /rgba?\(\s*([\d.]+)[\s,]+([\d.]+)[\s,]+([\d.]+)\s*[,/]\s*([\d.]+)\s*\)/g,
      "rgb($1 $2 $3 / $4)",
    );
}

function computed(): Record<string, Record<Scheme, Record<string, string>>> {
  const all: Record<string, Record<Scheme, Record<string, string>>> = {};
  for (const path of stylesheets) {
    const found = declarations(sheet(path, readFileSync(new URL(path, root), "utf8")));
    const schemes = { light: {}, dark: {} } as Record<Scheme, Record<string, string>>;
    for (const scheme of ["light", "dark"] as const) {
      for (const { property, value, context } of found) {
        const dark = context.some((c) => /prefers-color-scheme:\s*dark/.test(c));
        if (dark && scheme === "light") continue;
        schemes[scheme][`${context.join(" > ")} | ${property}`] = spell(pick(value, scheme));
      }
      schemes[scheme] = Object.fromEntries(
        Object.entries(schemes[scheme]).sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0)),
      );
    }
    all[path] = schemes;
  }
  return all;
}

test("every stylesheet's declarations are what they were", () => {
  const now = computed();
  if (process.env.ASCRIBE_BLESS === "1") {
    writeFileSync(fixture, `${JSON.stringify(now, null, 2)}\n`);
    return;
  }
  const before = JSON.parse(readFileSync(fixture, "utf8")) as ReturnType<typeof computed>;
  expect(now).toEqual(before);
});
