// Checks the published JavaScript packages as a consumer gets them: `pnpm check:packages`.
//
// Each package is packed the way a release packs it, then publint checks the
// tarball's package.json against its files, and Are the Types Wrong checks
// that every entry point resolves to JavaScript and to its types. Build the
// packages first (`pnpm build:all`, or each package's `build`).
//
// The packages are ESM only, so the check is for ESM consumers: Node 16's and
// later's ESM resolution, and bundlers. A `require` of them, and Node 10's
// resolution, which ignores `exports`, aren't supported.
import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, readdirSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import process from "node:process";
import { npmPackages, root } from "./manifests.ts";

/**
 * The entry points in a package's `exports` that aren't JavaScript, such as a
 * stylesheet or an Astro component. Are the Types Wrong resolves the way
 * TypeScript does, so it can't follow them; publint checks that their files
 * are in the package.
 */
export function nonCodeEntrypoints(exports: Record<string, unknown>): string[] {
  return Object.entries(exports)
    .filter(([, target]) => typeof target === "string" && !/\.[cm]?js$/.test(target))
    .map(([subpath]) => subpath);
}

function run(command: string, args: string[], cwd = root): boolean {
  process.stdout.write(`\n$ ${command} ${args.join(" ")}\n`);
  // pnpm is a .cmd script on Windows, which only a shell can start.
  const result = spawnSync(command, args, {
    cwd,
    stdio: "inherit",
    shell: process.platform === "win32",
  });
  return result.status === 0;
}

function main(): void {
  const out = mkdtempSync(join(tmpdir(), "ascribe-packages-"));
  const failed: string[] = [];
  try {
    // The platform packages hold only a binary; these are the JavaScript ones.
    for (const pkg of npmPackages.filter(({ target }) => target === undefined)) {
      const dir = join(out, pkg.name.replace("/", "-"));
      // pnpm pack replaces `workspace:*` with the version, as a release does.
      if (!run("pnpm", ["pack", "--pack-destination", dir], join(root, pkg.dir))) {
        failed.push(`${pkg.name}: pack`);
        continue;
      }
      const [tarball] = readdirSync(dir);
      if (tarball === undefined) {
        failed.push(`${pkg.name}: pack made no tarball`);
        continue;
      }
      const path = join(dir, tarball);
      const manifest = JSON.parse(readFileSync(join(root, pkg.dir, "package.json"), "utf8")) as {
        exports?: Record<string, unknown>;
      };
      const excluded = nonCodeEntrypoints(manifest.exports ?? {});
      if (!run("pnpm", ["exec", "publint", "run", path, "--strict"])) {
        failed.push(`${pkg.name}: publint`);
      }
      const attw = ["exec", "attw", path, "--profile", "esm-only"];
      if (excluded.length > 0) attw.push("--exclude-entrypoints", ...excluded);
      if (!run("pnpm", attw)) failed.push(`${pkg.name}: Are the Types Wrong`);
    }
  } finally {
    rmSync(out, { recursive: true, force: true });
  }
  if (failed.length > 0) {
    process.stderr.write(`\nThe packages failed these checks:\n${failed.join("\n")}\n`);
    process.exit(1);
  }
  process.stdout.write("\nEvery package passed publint and Are the Types Wrong.\n");
}

if (import.meta.main) main();
