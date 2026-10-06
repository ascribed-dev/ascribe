// Builds the site with the Ascribe in this checkout, not the one from npm: for
// working on Ascribe and its docs together, and for a pull request's preview,
// which has no canary yet. This isn't how production builds; production is
// `npm ci`, `npm run follow-next`, and `npm run build`, from npm alone.
//
// It's the only place the site touches packages/. The binary is
// `ASCRIBE_BIN`, or else target/debug/ascribe (`cargo build -p ascribe-cli`).
// The workspace's packages are built and packed, and the packs installed
// over the registry's without saving, so the lockfile keeps the canary.
//
// Needs the workspace installed (`pnpm install` at the repository's root).
// Afterwards, `npm ci` puts the registry's packages back.
import { execFileSync } from "node:child_process";
import { existsSync, mkdtempSync, readdirSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const site = fileURLToPath(new URL("..", import.meta.url));
const root = path.dirname(site);
const windows = process.platform === "win32";
const PACKAGES = ["cli", "elements", "review", "astro"];

function run(command, args, options = {}) {
  console.log(`> ${command} ${args.join(" ")}`);
  execFileSync(command, args, { stdio: "inherit", shell: windows, ...options });
}

const binary = path.resolve(
  process.env["ASCRIBE_BIN"] ??
    path.join(root, "target", "debug", windows ? "ascribe.exe" : "ascribe"),
);
if (!existsSync(binary)) {
  throw new Error(
    `no ascribe binary at ${binary}: run \`cargo build -p ascribe-cli\`, or set ASCRIBE_BIN`,
  );
}
if (!existsSync(path.join(root, "node_modules"))) {
  throw new Error("the workspace isn't installed: run `pnpm install` at the repository's root");
}

// pnpm, as the workspace's `packageManager` names it, through corepack if it isn't on the path.
let pnpm = ["pnpm"];
try {
  execFileSync("pnpm", ["--version"], { stdio: "ignore", shell: windows, cwd: root });
} catch {
  pnpm = ["corepack", "pnpm"];
}
const [pnpmCommand = "pnpm", ...pnpmArgs] = pnpm;

run(
  pnpmCommand,
  [...pnpmArgs, ...PACKAGES.flatMap((name) => ["--filter", `@ascribed/${name}`]), "build"],
  {
    cwd: root,
  },
);

const packs = mkdtempSync(path.join(tmpdir(), "ascribe-packs-"));
try {
  for (const name of PACKAGES) {
    // pnpm pack writes the workspace's `workspace:*` dependencies as versions.
    run(pnpmCommand, [...pnpmArgs, "pack", "--pack-destination", packs], {
      cwd: path.join(root, "packages", name),
    });
  }
  const tarballs = readdirSync(packs).map((file) => path.join(packs, file));
  run("npm", ["install", "--no-save", "--no-audit", "--no-fund", ...tarballs], { cwd: site });
} finally {
  rmSync(packs, { recursive: true, force: true });
}

run("npm", ["run", "build"], { cwd: site, env: { ...process.env, ASCRIBE_BIN: binary } });
