// Writes an archive's search script into the Rust crate that embeds it
// (`crates/ascribe-emit/src/archive/search.js`), which builds without Node:
//
//   pnpm --filter @ascribed/archive embed
//
// The script is `src/index.ts` bundled. Every archive an Ascribe version
// writes carries this copy. Run it after changing `src/`:
// `test/embedded.test.ts` fails while the crate's copy differs.
import { writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { build } from "esbuild";

const here = (path: string): string => fileURLToPath(new URL(path, import.meta.url));

/** Where the crate keeps the script. */
export const target = here("../../crates/ascribe-emit/src/archive/search.js");

/** The search script, as the crate embeds it. */
export async function bundle(): Promise<string> {
  const result = await build({
    entryPoints: [here("src/index.ts")],
    bundle: true,
    write: false,
    format: "iife",
    platform: "browser",
    target: "es2022",
    charset: "utf8",
    legalComments: "none",
    logLevel: "silent",
  });
  const js = result.outputFiles[0]?.text ?? "";
  return `/* The search script of an archive written by \`ascribe versions export\`, generated from\n   packages/archive by \`pnpm --filter @ascribed/archive embed\`. Don't edit it. */\n${js}`;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  writeFileSync(target, await bundle());
  process.stdout.write(`wrote ${target}\n`);
}
