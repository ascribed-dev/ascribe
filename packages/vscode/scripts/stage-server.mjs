// Puts an `ascribe` binary where the extension looks for its bundled one,
// `bin/<platform>-<arch>/`, so a development copy of the extension runs it
// with no `ascribe.path` setting. Release packaging empties `bin/` before it
// stages each target's binary, so a binary left here never ships.
//
//   node scripts/stage-server.mjs [binary]
//
// The binary defaults to the workspace's debug build, `target/debug/ascribe`.
import { chmodSync, copyFileSync, existsSync, mkdirSync, rmSync } from "node:fs";
import { join, resolve } from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const packageRoot = fileURLToPath(new URL("..", import.meta.url));
const exe = process.platform === "win32" ? "ascribe.exe" : "ascribe";
const source = resolve(process.argv[2] ?? join(packageRoot, "..", "..", "target", "debug", exe));
if (!existsSync(source)) {
  console.error(`No binary at ${source}. Build it with \`cargo build -p tessera-cli\`.`);
  process.exit(1);
}

const dir = join(packageRoot, "bin", `${process.platform}-${process.arch}`);
const destination = join(dir, exe);
mkdirSync(dir, { recursive: true });
// A new file, not one overwritten in place: macOS kills a signed binary
// whose file was rewritten under it.
rmSync(destination, { force: true });
copyFileSync(source, destination);
if (process.platform !== "win32") chmodSync(destination, 0o755);
console.log(`Staged ${source} as ${destination}`);
