// Runs the integration suites in a real VS Code with @vscode/test-electron.
//
//   pnpm --filter ascribe-vscode test:integration
//
// It needs a display: on Linux without one, run it under `xvfb-run -a`.
// VS Code is downloaded on first use into out/vscode-test.
//
// Suites (each opens its own copy of a fixture workspace):
//   activation  a workspace without ascribe.toml: the extension stays inactive
//   stub        a workspace with ascribe.toml, against test/stub-server
//   quill       a copy of examples/quill with a broken page added, against the
//               real `ascribe lsp`. Needs ASCRIBE_BIN, the path to a built
//               `ascribe`; skipped without it.
//   preview     a copy of examples/quill, the preview panel, against
//               the real `ascribe lsp`. Needs ASCRIBE_BIN as well.
//   monorepo    test/fixtures/monorepo, several projects (one nested in
//               another) with servers started on demand, against the real
//               `ascribe lsp`. Needs ASCRIBE_BIN as well.
import { cpSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import * as path from "node:path";
import { runTests } from "@vscode/test-electron";

declare const __dirname: string;

const packageRoot = path.resolve(__dirname, "../..");
const repositoryRoot = path.resolve(packageRoot, "../..");

interface Suite {
  name: string;
  /** Prepares the workspace in a fresh directory and returns extra settings. */
  prepare(workspace: string): Record<string, unknown>;
  fixture: string;
}

const stubServer = path.join(packageRoot, "test/stub-server/ascribe");
const realServer = process.env["ASCRIBE_BIN"];

// These suites check a server that's running from the start, with no file open.
const startAll = { "ascribe.startServers": "all" };

const suites: Suite[] = [
  {
    name: "activation",
    fixture: path.join(packageRoot, "test/fixtures/no-model"),
    prepare: () => ({ "ascribe.path": stubServer }),
  },
  {
    name: "stub",
    fixture: path.join(packageRoot, "test/fixtures/stub-project"),
    prepare: () => ({ "ascribe.path": stubServer, "ascribe.maxCrashes": 2, ...startAll }),
  },
  {
    name: "quill",
    fixture: path.join(repositoryRoot, "examples/quill"),
    prepare: (workspace) => {
      cpSync(
        path.join(packageRoot, "test/fixtures/broken-quill-docs"),
        path.join(workspace, "docs"),
        { recursive: true },
      );
      return { "ascribe.path": realServer, ...startAll };
    },
  },
  {
    name: "preview",
    fixture: path.join(repositoryRoot, "examples/quill"),
    prepare: () => ({ "ascribe.path": realServer, ...startAll }),
  },
  {
    name: "monorepo",
    fixture: path.join(packageRoot, "test/fixtures/monorepo"),
    // The default `ascribe.startServers`: the suite checks what starts when.
    prepare: (workspace) => {
      // The folders the suite turns into projects. On Linux, VS Code's file
      // watcher can miss a directory made while it runs, and then never
      // reports the files in it; folders that exist when it starts are watched.
      for (const folder of ["guides", "idle"]) {
        mkdirSync(path.join(workspace, folder, "docs"), { recursive: true });
      }
      return { "ascribe.path": realServer };
    },
  },
];

/** The suites that run the real language server. */
const needsServer = new Set(["quill", "preview", "monorepo"]);

async function main(): Promise<void> {
  // A process an extension host started has this set, and VS Code would then
  // run as plain Node instead of launching.
  delete process.env["ELECTRON_RUN_AS_NODE"];
  const only = process.env["ASCRIBE_SUITE"];
  if (only && !suites.some((suite) => suite.name === only)) {
    console.error(`There is no suite named ${only}.`);
    process.exit(1);
  }
  let failed = false;
  let ran = 0;
  for (const suite of suites) {
    if (only && only !== suite.name) continue;
    if (needsServer.has(suite.name) && !realServer) {
      console.log(
        `Skipping suite ${suite.name}: set ASCRIBE_BIN to a built \`ascribe\` (the language server).`,
      );
      continue;
    }
    const scratch = // Short on purpose: VS Code's IPC socket lives under --user-data-dir, and a
      // Unix socket path can be at most 103 characters on macOS, where $TMPDIR is long.
      mkdtempSync(path.join(tmpdir(), "tv-"));
    const workspace = path.join(scratch, "workspace");
    try {
      cpSync(suite.fixture, workspace, { recursive: true });
      const settings = suite.prepare(workspace);
      mkdirSync(path.join(workspace, ".vscode"), { recursive: true });
      writeFileSync(
        path.join(workspace, ".vscode/settings.json"),
        JSON.stringify(settings, null, 2),
      );
      console.log(`\n== Suite ${suite.name}`);
      await runTests({
        cachePath: path.join(packageRoot, "out/vscode-test"),
        version: process.env["VSCODE_VERSION"] ?? "stable",
        extensionDevelopmentPath: packageRoot,
        extensionTestsPath: path.join(packageRoot, "out/integration/suite/index.cjs"),
        extensionTestsEnv: { ASCRIBE_SUITE: suite.name, ASCRIBE_WORKSPACE: workspace },
        launchArgs: [
          workspace,
          "--disable-extensions",
          "--disable-workspace-trust",
          "--disable-gpu",
          "--disable-updates",
          "--no-sandbox",
          "--skip-welcome",
          "--skip-release-notes",
          `--user-data-dir=${path.join(scratch, "user-data")}`,
          `--extensions-dir=${path.join(scratch, "extensions")}`,
        ],
      });
      ran += 1;
    } catch (error) {
      console.error(`Suite ${suite.name} failed:`, error);
      failed = true;
    } finally {
      rmSync(scratch, { recursive: true, force: true });
    }
  }
  if (failed) process.exit(1);
  console.log(`\n${ran} integration suite(s) passed.`);
}

void main();
