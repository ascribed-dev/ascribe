// Installs a packaged extension into a fresh VS Code and checks it works with
// examples/quill: once with the binary bundled in the package, and once with a
// project binary in node_modules/.bin, which the extension must prefer.
//
//   node scripts/release/smoke-vsix.mjs <file.vsix> [--vscode-platform <name>]
//
// --vscode-platform picks which VS Code build to download, by
// @vscode/test-electron's names: darwin-arm64, linux-x64, linux-arm64, and
// win32-x64-archive. By default, the machine's own.
//
// On Linux without a display, run it under `xvfb-run -a`.
import { spawnSync } from "node:child_process";
import {
  chmodSync,
  cpSync,
  mkdirSync,
  mkdtempSync,
  readdirSync,
  rmSync,
  unlinkSync,
  writeFileSync,
} from "node:fs";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import process from "node:process";
import { parseArgs } from "node:util";
import { extension, root } from "./manifests.mjs";

const require = createRequire(join(root, extension.dir, "package.json"));
const {
  downloadAndUnzipVSCode,
  resolveCliArgsFromVSCodeExecutablePath,
  runTests,
} = require("@vscode/test-electron");

const { values: options, positionals } = parseArgs({
  allowPositionals: true,
  options: { "vscode-platform": { type: "string" } },
});
const [vsixArgument] = positionals;
if (vsixArgument === undefined) {
  process.stderr.write("usage: smoke-vsix.mjs <file.vsix> [--vscode-platform <name>]\n");
  process.exit(2);
}
const vsix = resolve(vsixArgument);
const platform = options["vscode-platform"];

// A process an extension host started has this set, and VS Code would then run
// as plain Node instead of launching.
delete process.env.ELECTRON_RUN_AS_NODE;

const vscodeExecutablePath = await downloadAndUnzipVSCode({
  cachePath: join(root, extension.dir, "out", "vscode-test"),
  version: process.env.VSCODE_VERSION ?? "stable",
  ...(platform && { platform }),
});

// Short on purpose: VS Code's IPC socket lives under the user data directory,
// and a Unix socket path can be at most 103 characters on macOS.
const scratch = mkdtempSync(join(tmpdir(), "as-"));
let failed = false;
try {
  const extensions = join(scratch, "ext");
  // `reuseMachineInstall` keeps the helper from adding its own profile
  // directories under ./.vscode-test; these are the smoke test's.
  const [cli, ...cliArgs] = resolveCliArgsFromVSCodeExecutablePath(vscodeExecutablePath, {
    reuseMachineInstall: true,
    ...(platform && { platform }),
  });
  const install = spawnSync(
    cli,
    [
      ...cliArgs,
      `--extensions-dir=${extensions}`,
      `--user-data-dir=${join(scratch, "ud")}`,
      "--install-extension",
      vsix,
    ],
    { stdio: "inherit", shell: process.platform === "win32" },
  );
  if (install.status !== 0) throw new Error(`installing ${vsix} failed`);

  const workspace = join(scratch, "ws");
  cpSync(join(root, "examples", "quill"), workspace, { recursive: true });
  cpSync(
    join(root, extension.dir, "test", "fixtures", "broken-quill-docs"),
    join(workspace, "docs"),
    { recursive: true },
  );

  await smoke("bundled", workspace, extensions);

  // A project binary: a launcher in node_modules/.bin that runs the bundled
  // binary, as npm's link to @ascribed/cli's launcher would run the project's.
  const installed = readdirSync(extensions).find((name) =>
    name.toLowerCase().startsWith(`${extension.id.toLowerCase()}-`),
  );
  if (installed === undefined) throw new Error(`${extension.id} isn't in ${extensions}`);
  const bundled = join(
    extensions,
    installed,
    "bin",
    target(),
    process.platform === "win32" ? "ascribe.exe" : "ascribe",
  );
  const bin = join(workspace, "node_modules", ".bin");
  mkdirSync(bin, { recursive: true });
  if (process.platform === "win32") {
    writeFileSync(join(bin, "ascribe.cmd"), `@"${bundled}" %*\r\n`);
  } else {
    writeFileSync(join(bin, "ascribe"), `#!/bin/sh\nexec "${bundled}" "$@"\n`);
    chmodSync(join(bin, "ascribe"), 0o755);
  }
  await smoke("project", workspace, extensions);
  unlinkSync(join(bin, process.platform === "win32" ? "ascribe.cmd" : "ascribe"));
} catch (error) {
  process.stderr.write(
    `smoke test failed: ${error instanceof Error ? error.message : String(error)}\n`,
  );
  failed = true;
} finally {
  rmSync(scratch, { recursive: true, force: true });
}
process.exit(failed ? 1 : 0);

async function smoke(source, workspace, extensions) {
  process.stdout.write(`\n== ${source} binary\n`);
  await runTests({
    vscodeExecutablePath,
    extensionDevelopmentPath: join(root, "scripts", "release", "smoke"),
    extensionTestsPath: join(root, "scripts", "release", "smoke", "suite.cjs"),
    extensionTestsEnv: { SMOKE_EXPECT_SOURCE: source, SMOKE_WORKSPACE: workspace },
    launchArgs: [
      workspace,
      "--disable-workspace-trust",
      "--disable-gpu",
      "--disable-updates",
      "--no-sandbox",
      "--skip-welcome",
      "--skip-release-notes",
      `--extensions-dir=${extensions}`,
      `--user-data-dir=${join(workspace, "..", "ud")}`,
    ],
  });
}

/** The target the installed package was built for, from its file name. */
function target() {
  const match = /ascribe-vscode-([a-z0-9]+-[a-z0-9]+)-/.exec(vsix);
  if (!match) throw new Error(`can't tell the target from the file name ${vsix}`);
  return match[1];
}
