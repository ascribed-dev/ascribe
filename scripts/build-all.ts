// Builds every JS package from a fresh checkout: `pnpm build:all`.
//
// `pnpm -r build` alone fails in examples/astro-site, whose build runs the
// `ascribe` binary through @ascribed/cli, which finds it in the native package
// for this machine. So this does what CI's js.yml does, in its order: build the
// binary, stage it into that package, then build every package in the
// workspace, which pnpm orders by their dependencies on each other.
import { spawnSync } from "node:child_process";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("..", import.meta.url));
const target = `${process.platform}-${process.arch}`;
const exe = process.platform === "win32" ? "ascribe.exe" : "ascribe";

function run(command: string, args: string[], env: NodeJS.ProcessEnv = process.env): void {
  process.stdout.write(`\n$ ${command} ${args.join(" ")}\n`);
  // pnpm is a .cmd script on Windows, which only a shell can start.
  const result = spawnSync(command, args, {
    cwd: root,
    env,
    stdio: "inherit",
    shell: process.platform === "win32",
  });
  if (result.status !== 0) process.exit(result.status ?? 1);
}

run("cargo", ["build", "-p", "tessera-cli", "--locked"]);
run("pnpm", ["--filter", "@ascribed/cli", "stage-native", target], {
  ...process.env,
  [`ASCRIBE_BIN_${target.replace("-", "_").toUpperCase()}`]: path.join(
    root,
    "target",
    "debug",
    exe,
  ),
});
run("pnpm", ["-r", "build"]);
