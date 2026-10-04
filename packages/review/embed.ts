// Writes the static report's script and stylesheet into the Rust crate that
// embeds them (`crates/tessera-diff/src/html/`), which builds without
// Node:
//
//   pnpm --filter @ascribed/review embed
//
// The script is the report's (`src/report/`) bundled with the marks and the
// element library, from their sources; the stylesheet is the element
// library's, the marks', and the report's, in that order. The script's
// SHA-256, in base64, goes beside it: the report's content security policy
// allows only the script with that hash to run. Run it after
// changing any of them: `test/embedded.test.ts` fails while the crate's
// copies differ.
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { build } from "esbuild";

const here = (path: string): string => fileURLToPath(new URL(path, import.meta.url));

/** Where the crate keeps the files. */
export const target = {
  js: here("../../crates/tessera-diff/src/html/report.js"),
  css: here("../../crates/tessera-diff/src/html/report.css"),
  hash: here("../../crates/tessera-diff/src/html/report.js.sha256"),
};

const header = (what: string): string =>
  `/* ${what} of \`ascribe diff --format html\`, generated from packages/review and\n   packages/elements by \`pnpm --filter @ascribed/review embed\`. Don't edit it. */\n`;

/** The report's script, stylesheet, and script hash, as the crate embeds them. */
export async function bundle(): Promise<{ js: string; css: string; hash: string }> {
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
  const script = header("The script") + js;
  return {
    js: script,
    css: `${header("The stylesheet")}\n${css}\n`,
    hash: createHash("sha256").update(script, "utf8").digest("base64"),
  };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const { js, css, hash } = await bundle();
  writeFileSync(target.js, js);
  writeFileSync(target.css, css);
  writeFileSync(target.hash, hash);
  process.stdout.write(`wrote ${target.js}\nwrote ${target.css}\nwrote ${target.hash}\n`);
}
