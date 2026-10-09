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
//   review      a copy of examples/quill made into a git repository with one
//               commit and a change in the working tree, review in the
//               preview, against the real `ascribe lsp`. Needs ASCRIBE_BIN
//               and `git`.
//   threads     a copy of examples/quill made into a git repository with a
//               `feature` branch pushed to github.com/acme/quill, review
//               threads in the preview and the source editor from a fake
//               GitHub, against the real `ascribe lsp`. Opened through a
//               symlink, except on Windows. Needs ASCRIBE_BIN and `git`.
//   actions     a copy of examples/quill, the editor's actions run through
//               their commands with scripted answers, against the real
//               `ascribe lsp`. Needs ASCRIBE_BIN as well.
//   site        a copy of examples/quill, Open Site Preview and the preview
//               panel's Site view, against a fake dev server, with the real
//               `ascribe lsp`. Needs ASCRIBE_BIN as well.
//   monorepo    test/fixtures/monorepo, several projects (one nested in
//               another) with servers started on demand, against the real
//               `ascribe lsp`. Needs ASCRIBE_BIN as well.
import { execFileSync } from "node:child_process";
import {
  cpSync,
  mkdirSync,
  mkdtempSync,
  readdirSync,
  readFileSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { Writable } from "node:stream";
import { stripVTControlCharacters } from "node:util";
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
  /** More arguments for VS Code. */
  launchArgs?: string[];
  /**
   * VS Code runs with trace logs: its output is kept off the console unless
   * the suite fails, and then printed with the file watcher's logs.
   */
  traceLogs?: boolean;
  /** Open the workspace through a symlink to it (not on Windows), as macOS's /var is. */
  throughLink?: boolean;
}

const stubServer = path.join(packageRoot, "test/stub-server/ascribe");
const realServer = process.env["ASCRIBE_BIN"];

/** Runs `git` in a workspace, as a fixed author, with no signing. */
function gitIn(workspace: string): (...args: string[]) => void {
  return (...args) => {
    execFileSync(
      "git",
      [
        "-c",
        "user.name=Test",
        "-c",
        "user.email=test@example.com",
        "-c",
        "commit.gpgsign=false",
        "-c",
        "core.autocrlf=false",
        ...args,
      ],
      { cwd: workspace },
    );
  };
}

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
    name: "review",
    fixture: path.join(repositoryRoot, "examples/quill"),
    prepare: (workspace) => {
      const git = (...args: string[]): void => {
        execFileSync(
          "git",
          [
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.com",
            "-c",
            "commit.gpgsign=false",
            "-c",
            "core.autocrlf=false",
            ...args,
          ],
          { cwd: workspace },
        );
      };
      git("init", "-q", "-b", "main");
      git("add", "-A");
      git("commit", "-q", "-m", "The Quill example");
      const edit = (file: string, from: string, to: string): void => {
        const at = path.join(workspace, "docs", file);
        const text = readFileSync(at, "utf8");
        if (!text.includes(from)) throw new Error(`${file} has no ${from}`);
        writeFileSync(at, text.replace(from, to));
      };
      edit("install-agent.md", "syncs changes to", "syncs every change to");
      edit("_fragments/prerequisites.md", "- Node.js 20 or later.", "- Node.js 22 or later.");
      return { "ascribe.path": realServer, ...startAll };
    },
  },
  {
    name: "threads",
    // git gives real paths; the editors keep the link (#74).
    throughLink: true,
    fixture: path.join(repositoryRoot, "examples/quill"),
    prepare: (workspace) => {
      const git = gitIn(workspace);
      git("init", "-q", "-b", "main");
      git("add", "-A");
      git("commit", "-q", "-m", "The Quill example");
      git("checkout", "-q", "-b", "feature");
      const at = path.join(workspace, "docs", "install-agent.md");
      const text = readFileSync(at, "utf8");
      if (!text.includes("syncs changes to")) throw new Error("install-agent.md has changed");
      writeFileSync(at, text.replace("syncs changes to", "syncs every change to"));
      git("commit", "-q", "-am", "Reword the agent's introduction");
      // Pushed to GitHub: the branch's remote, and the base's remote branch.
      git("remote", "add", "origin", "https://github.com/acme/quill.git");
      git("config", "branch.feature.remote", "origin");
      git("config", "branch.feature.merge", "refs/heads/feature");
      git("update-ref", "refs/remotes/origin/main", "main");
      git("update-ref", "refs/remotes/origin/feature", "feature");
      return { "ascribe.path": realServer, ...startAll };
    },
  },
  {
    name: "actions",
    fixture: path.join(repositoryRoot, "examples/quill"),
    prepare: () => ({ "ascribe.path": realServer, ...startAll }),
  },
  {
    name: "site",
    fixture: path.join(repositoryRoot, "examples/quill"),
    prepare: () => ({ "ascribe.path": realServer, ...startAll }),
  },
  {
    name: "monorepo",
    fixture: path.join(packageRoot, "test/fixtures/monorepo"),
    // The file watcher's trace, to see why it missed a file if it does (#56).
    launchArgs: ["--log=trace"],
    traceLogs: true,
    // The default `ascribe.startServers`: the suite checks what starts when.
    prepare: (workspace) => {
      // The folders the suite turns into projects. On Linux, VS Code's file
      // watcher can miss a directory made while it runs, and then never
      // reports the files in it; folders that exist when it starts are watched.
      for (const folder of ["guides", "idle"]) {
        mkdirSync(path.join(workspace, folder, "docs"), { recursive: true });
      }
      return {
        "ascribe.path": realServer,
        // A project the watcher never reports, found when one of its files opens.
        "files.watcherExclude": { "**/unwatched/**": true },
      };
    },
  },
];

/** The suites that run the real language server. */
const needsServer = new Set([
  "quill",
  "preview",
  "review",
  "threads",
  "actions",
  "site",
  "monorepo",
]);

/** VS Code's output, kept for later. */
class Captured extends Writable {
  private readonly chunks: Buffer[] = [];

  override _write(chunk: Buffer, _encoding: string, done: () => void): void {
    this.chunks.push(Buffer.from(chunk));
    done();
  }

  text(): string {
    return Buffer.concat(this.chunks).toString("utf8");
  }

  /** Mocha's lines: each test's result and the counts. */
  testResults(): string {
    const results = this.text()
      .split("\n")
      // Without mocha's colors.
      .map((line) => stripVTControlCharacters(line))
      .filter((line) => /^\s*(✔|✓|\d+\) |\d+ (passing|pending|failing))/.test(line));
    return `${results.join("\n")}\n(VS Code's trace output is printed when the suite fails.)`;
  }
}

/** The end of each file watcher log VS Code wrote, or the logs there are when there's none. */
function printWatcherLogs(logs: string): void {
  let files: string[];
  try {
    files = readdirSync(logs, { recursive: true, encoding: "utf8" }).map((f) => path.join(logs, f));
  } catch {
    console.error(`No VS Code logs in ${logs}.`);
    return;
  }
  const watcher = files.filter(
    (file) => /watcher/i.test(path.basename(file)) && file.endsWith(".log"),
  );
  if (watcher.length === 0) {
    console.error(`No file watcher log. VS Code's logs:\n${files.join("\n")}`);
    return;
  }
  for (const file of watcher) {
    const lines = readFileSync(file, "utf8").split("\n");
    console.error(`\n-- ${path.relative(logs, file)} (last 400 of ${lines.length} lines)`);
    console.error(lines.slice(-400).join("\n"));
  }
}

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
    const output = new Captured();
    try {
      cpSync(suite.fixture, workspace, { recursive: true });
      const settings = suite.prepare(workspace);
      mkdirSync(path.join(workspace, ".vscode"), { recursive: true });
      writeFileSync(
        path.join(workspace, ".vscode/settings.json"),
        JSON.stringify(settings, null, 2),
      );
      let opened = workspace;
      if (suite.throughLink && process.platform !== "win32") {
        opened = path.join(scratch, "link");
        symlinkSync(workspace, opened);
      }
      console.log(`\n== Suite ${suite.name}`);
      await runTests({
        ...(suite.traceLogs ? { stdout: output, stderr: output } : {}),
        cachePath: path.join(packageRoot, "out/vscode-test"),
        version: process.env["VSCODE_VERSION"] ?? "stable",
        extensionDevelopmentPath: packageRoot,
        extensionTestsPath: path.join(packageRoot, "out/integration/suite/index.cjs"),
        extensionTestsEnv: { ASCRIBE_SUITE: suite.name },
        launchArgs: [
          opened,
          "--disable-extensions",
          "--disable-workspace-trust",
          "--disable-gpu",
          "--disable-updates",
          "--no-sandbox",
          "--skip-welcome",
          "--skip-release-notes",
          `--user-data-dir=${path.join(scratch, "user-data")}`,
          `--extensions-dir=${path.join(scratch, "extensions")}`,
          ...(suite.launchArgs ?? []),
        ],
      });
      if (suite.traceLogs) console.log(output.testResults());
      ran += 1;
    } catch (error) {
      if (suite.traceLogs) {
        process.stdout.write(output.text());
        printWatcherLogs(path.join(scratch, "user-data", "logs"));
      }
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
