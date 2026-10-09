// The webview tests load the preview the way VS Code does: the shell HTML
// and the bundled files under `dist/webview/`. Bundle them first, unless each
// is already newer than everything esbuild.mjs bundles them from (this
// package's source, the element library's source and stylesheet, and review's
// source) and the script itself, so a repeated local run doesn't wait.
import { execFileSync } from "node:child_process";
import { readdirSync, statSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const here = fileURLToPath(new URL("../..", import.meta.url));
const packages = join(here, "..");

/** The files `dist/webview/` holds when esbuild.mjs has run. */
const OUTPUTS = ["elements.js", "elements.css", "marks.css", "preview.js", "preview.css"];

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

/** Whether every bundled file exists and is newer than every input. */
function upToDate(): boolean {
  let outputs: number[];
  try {
    outputs = OUTPUTS.map((name) => statSync(join(here, "dist", "webview", name)).mtimeMs);
  } catch {
    return false;
  }
  const inputs = [
    statSync(join(here, "esbuild.mjs")).mtimeMs,
    ...mtimes(join(here, "src")),
    ...mtimes(join(packages, "elements", "src")),
    ...mtimes(join(packages, "elements", "css")),
    ...mtimes(join(packages, "review", "src")),
  ];
  return Math.min(...outputs) > Math.max(...inputs);
}

export default function setup(): void {
  if (upToDate()) return;
  execFileSync(process.execPath, ["esbuild.mjs"], { cwd: here, stdio: "inherit" });
}
