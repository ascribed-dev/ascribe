// The browser tests load the library the way a site does: as the compiled ES
// module plus the CSS file. Compile it once before they run.
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

export default function setup(): void {
  const root = fileURLToPath(new URL("..", import.meta.url));
  const tsc = fileURLToPath(new URL("../../../node_modules/typescript/bin/tsc", import.meta.url));
  execFileSync(process.execPath, [tsc, "-p", "tsconfig.build.json"], {
    cwd: root,
    stdio: "inherit",
  });
}
