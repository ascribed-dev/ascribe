// The webview tests load the preview the way VS Code does: the shell HTML
// and the bundled files under `dist/webview/`. Build them first, elements
// package included.
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

export default function setup(): void {
  const elements = fileURLToPath(new URL("../../../elements", import.meta.url));
  const here = fileURLToPath(new URL("../..", import.meta.url));
  const tsc = fileURLToPath(
    new URL("../../../../node_modules/typescript/bin/tsc", import.meta.url),
  );
  execFileSync(process.execPath, [tsc, "-p", "tsconfig.build.json"], {
    cwd: elements,
    stdio: "inherit",
  });
  execFileSync(process.execPath, ["esbuild.mjs"], { cwd: here, stdio: "inherit" });
}
