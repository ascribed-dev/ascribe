// Runs before `pnpm typecheck`: stops it when a package other packages import
// hasn't been built yet.
//
// @ascribed/cli and @ascribed/elements export their types from `dist/`, which
// their builds write. Without it, tsc reports "Cannot find module" for an
// import that is right, so this says what's missing and how to build it.
import { existsSync } from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("..", import.meta.url));

const packages = [
  { name: "@ascribed/cli", dir: "packages/cli" },
  { name: "@ascribed/elements", dir: "packages/elements" },
];

const missing = packages.filter((p) => !existsSync(path.join(root, p.dir, "dist", "index.d.ts")));
if (missing.length > 0) {
  const lines = missing.map((p) => `  ${p.dir}/dist/ (${p.name})`);
  process.stderr.write(
    [
      "pnpm typecheck needs these packages built first, and their builds are missing:",
      ...lines,
      "",
      "Run `pnpm build:all`, then `pnpm typecheck` again. To build only these:",
      ...missing.map((p) => `  pnpm --filter ${p.name} build`),
      "",
    ].join("\n"),
  );
  process.exit(1);
}
