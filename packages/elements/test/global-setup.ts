// The browser tests load the library the way a site does: as the compiled ES
// module plus the CSS file. Compile it before they run, unless `dist/` is
// already newer than every source and the compiler's configuration, so a
// repeated local run doesn't wait for the compiler.
import { execFileSync } from "node:child_process";
import { readdirSync, statSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("..", import.meta.url));

/** The modification times of every file under `dir`, recursively; none when it's missing. */
function mtimes(dir: string): number[] {
  let entries;
  try {
    entries = readdirSync(dir, { withFileTypes: true });
  } catch {
    return [];
  }
  return entries.flatMap((entry) => {
    const file = join(dir, entry.name);
    return entry.isDirectory() ? mtimes(file) : [statSync(file).mtimeMs];
  });
}

/** Whether every file in `dist/` is newer than every source and both tsconfigs. */
function upToDate(): boolean {
  const outputs = mtimes(join(root, "dist"));
  if (outputs.length === 0) return false;
  const inputs = [
    ...mtimes(join(root, "src")),
    ...["tsconfig.json", "tsconfig.build.json"].map((f) => statSync(join(root, f)).mtimeMs),
  ];
  return Math.min(...outputs) > Math.max(...inputs);
}

export default function setup(): void {
  if (upToDate()) return;
  const tsc = fileURLToPath(new URL("../../../node_modules/typescript/bin/tsc", import.meta.url));
  execFileSync(process.execPath, [tsc, "-p", "tsconfig.build.json"], {
    cwd: root,
    stdio: "inherit",
  });
}
