// Writes the static report's script and stylesheet into the Rust crate that
// embeds them (`crates/tessera-diff/src/html/`), which builds without
// Node:
//
//   pnpm --filter @ascribed/review embed
//
// The script is the report's (`src/report/`) bundled with the marks and the
// element library, from their sources; the stylesheet is the element
// library's, the marks', and the report's, in that order. Run it after
// changing any of them: `test/embedded.test.ts` fails while the crate's
// copies differ.
import { readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { build } from "esbuild";

const here = (path: string): string => fileURLToPath(new URL(path, import.meta.url));

/** Where the crate keeps the files. */
export const target = {
  js: here("../../crates/tessera-diff/src/html/report.js"),
  css: here("../../crates/tessera-diff/src/html/report.css"),
};

const header = (what: string): string =>
  `/* ${what} of \`ascribe diff --format html\`, generated from packages/review and\n   packages/elements by \`pnpm --filter @ascribed/review embed\`. Don't edit it. */\n`;

/** The report's script and stylesheet, as the crate embeds them. */
export async function bundle(): Promise<{ js: string; css: string }> {
  const result = await build({
    entryPoints: [here("src/report/index.ts")],
    bundle: true,
    write: false,
    format: "iife",
    platform: "browser",
    target: "es2022",
    charset: "utf8",
    legalComments: "none",
    // The package marks only its `dist/index.js` as having side effects;
    // bundling its source, registering the elements is the point.
    ignoreAnnotations: true,
    alias: { "@ascribed/elements": here("../elements/src/index.ts") },
    logLevel: "silent",
  });
  const js = result.outputFiles[0]?.text ?? "";
  const css = [
    here("../elements/css/style.css"),
    here("src/marks/marks.css"),
    here("src/report/report.css"),
  ]
    .map((file) => readFileSync(file, "utf8").trimEnd())
    .join("\n\n");
  return {
    js: header("The script") + js,
    css: `${header("The stylesheet")}\n${css}\n`,
  };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const { js, css } = await bundle();
  writeFileSync(target.js, js);
  writeFileSync(target.css, css);
  process.stdout.write(`wrote ${target.js}\nwrote ${target.css}\n`);
}
