// The webview tests load the preview the way VS Code does: the shell HTML
// and the bundled files under `dist/webview/`. Bundle them first.
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

export default function setup(): void {
  const here = fileURLToPath(new URL("../..", import.meta.url));
  execFileSync(process.execPath, ["esbuild.mjs"], { cwd: here, stdio: "inherit" });
}
