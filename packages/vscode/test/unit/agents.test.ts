// What agents in VS Code get from the extension, apart from VS Code: the
// MCP server it offers, the diagnostics it keeps with their versions, and the
// tools' answers in `ascribe check`'s and review's shapes.
import { readFileSync } from "node:fs";
import { describe, expect, it, vi } from "vitest";
import { changesReport, MAX_PAGES } from "../../src/agents/changes.js";
import { mcpServerSpec } from "../../src/agents/mcpServer.js";
import {
  CHECK_SCHEMA_VERSION,
  MAX_PROBLEMS,
  problemsReport,
  relativePath,
  type ProblemsInput,
} from "../../src/agents/problems.js";
import { PublishedDiagnostics, type ProtocolDiagnostic } from "../../src/agents/published.js";
import type { ResolvedBinary } from "../../src/binary.js";
import type { ChangedPage, ChangesResult } from "../../src/shapes.js";

const binary = (path: string): ResolvedBinary => ({
  path,
  source: "project",
  version: { parts: [0, 3, 0], prerelease: undefined },
});

describe("the MCP server", () => {
  it("runs `ascribe mcp` with the first project's binary, in the workspace folder", () => {
    const spec = mcpServerSpec(
      [
        { name: "docs", binary: binary("/w/node_modules/.bin/ascribe") },
        { name: "api", binary: binary("/w/node_modules/.bin/ascribe") },
      ],
      "/w",
      "linux",
    );
    expect(spec).toEqual({
      label: "Ascribe",
      command: "/w/node_modules/.bin/ascribe",
      args: ["mcp"],
      cwd: "/w",
      version: "0.3.0",
      note: undefined,
    });
  });

  it("says which binary runs when the projects resolve different ones", () => {
    const spec = mcpServerSpec(
      [
        { name: "docs", binary: undefined },
        { name: "guides", binary: binary("/w/guides/node_modules/.bin/ascribe") },
        { name: "api", binary: binary("/opt/ascribe") },
      ],
      "/w",
      "linux",
    );
    expect(spec?.command).toBe("/w/guides/node_modules/.bin/ascribe");
    expect(spec?.note).toBe(
      "The workspace's projects use different ascribe binaries. Its MCP server runs /w/guides/node_modules/.bin/ascribe, guides's, for every project; api uses /opt/ascribe.",
    );
  });

  it("runs a Windows shim through cmd.exe, since VS Code starts it without a shell", () => {
    const spec = mcpServerSpec(
      [{ name: "docs", binary: binary("C:\\w\\node_modules\\.bin\\ascribe.cmd") }],
      "C:\\w",
      "win32",
    );
    expect(spec?.command).toBe("cmd.exe");
    expect(spec?.args).toEqual(["/d", "/c", "C:\\w\\node_modules\\.bin\\ascribe.cmd", "mcp"]);
  });

  it("is none without a binary", () => {
    expect(mcpServerSpec([{ name: "docs", binary: undefined }], "/w")).toBeUndefined();
    expect(mcpServerSpec([], "/w")).toBeUndefined();
  });
});

const toPath = (uri: string) =>
  uri.startsWith("file://") ? uri.slice("file://".length) : undefined;

describe("the published diagnostics", () => {
  it("keep each file's last publication, with its version", () => {
    const store = new PublishedDiagnostics(
      toPath,
      () => false,
      () => 42,
    );
    store.record({ uri: "file:///p/docs/a.md", version: 3, diagnostics: [] });
    store.record({ uri: "file:///p/docs/b.md", diagnostics: [{ message: "x" }] });
    store.record({ uri: "untitled:1", diagnostics: [] });
    store.record({ nonsense: true });
    expect(store.get("/p/docs/a.md")).toEqual({
      file: "/p/docs/a.md",
      version: 3,
      diagnostics: [],
      at: 42,
    });
    expect(store.get("/p/docs/b.md")?.version).toBeNull();
    expect(store.all()).toHaveLength(2);
    store.clear();
    expect(store.all()).toEqual([]);
  });

  it("leave out another project's files", () => {
    const store = new PublishedDiagnostics(toPath, (file) => file.startsWith("/p/nested/"));
    store.record({ uri: "file:///p/nested/docs/a.md", diagnostics: [] });
    expect(store.all()).toEqual([]);
  });

  it("wait for the publication that's current, and no longer than the timeout", async () => {
    vi.useFakeTimers();
    try {
      const store = new PublishedDiagnostics(toPath);
      store.record({ uri: "file:///p/a.md", version: 1, diagnostics: [] });
      const waiting = store.waitFor("/p/a.md", (p) => p?.version === 2, 1_000);
      store.record({ uri: "file:///p/b.md", version: 2, diagnostics: [] });
      store.record({ uri: "file:///p/a.md", version: 2, diagnostics: [{ message: "late" }] });
      await expect(waiting).resolves.toMatchObject({
        current: true,
        publication: { version: 2, diagnostics: [{ message: "late" }] },
      });

      const late = store.waitFor("/p/a.md", (p) => p?.version === 3, 1_000);
      vi.advanceTimersByTime(1_000);
      await expect(late).resolves.toMatchObject({ current: false, publication: { version: 2 } });

      // Already current: no wait.
      await expect(store.waitFor("/p/a.md", () => true, 1_000)).resolves.toMatchObject({
        current: true,
      });
    } finally {
      vi.useRealTimers();
    }
  });
});

const ROOT = "/p";

function diagnostic(over: Partial<ProtocolDiagnostic> = {}): ProtocolDiagnostic {
  return {
    range: { start: { line: 2, character: 5 }, end: { line: 2, character: 9 } },
    severity: 1,
    code: "ASC036",
    codeDescription: { href: "https://ascribed-dev.com/reference/diagnostics/#asc036" },
    message: "no page at `nowhere.md`",
    data: {
      slug: "link-target-missing",
      builds: ["site"],
      unpublished: false,
      help: "Fix the link.",
      fixes: [
        {
          title: "Link to `somewhere.md`",
          applicability: "unsafe",
          edits: [
            {
              range: { start: { line: 2, character: 5 }, end: { line: 2, character: 9 } },
              newText: "somewhere",
            },
          ],
        },
      ],
    },
    ...over,
  };
}

function input(over: Partial<ProblemsInput> = {}): ProblemsInput {
  return {
    root: ROOT,
    publications: [],
    text: () => undefined,
    toPath,
    ascribeVersion: "0.3.0",
    editorBuild: "site",
    filesChecked: 12,
    filesReported: 1,
    documentVersion: 7,
    current: true,
    unsaved: [],
    nextCommand: "ascribe check --editor-build --format json docs/a.md",
    ...over,
  };
}

describe("the problems report", () => {
  it("is `ascribe check`'s JSON, positions counted as check counts them", () => {
    // Line 3 starts after "---\r\n" and "é😀\n": the column counts characters,
    // the offset bytes, and the protocol's character UTF-16 units.
    const text = "---\r\né😀\nab😀cdefgh\n";
    const report = problemsReport(
      input({
        publications: [
          {
            file: "/p/docs/a.md",
            version: 7,
            at: 0,
            diagnostics: [
              diagnostic({
                relatedInformation: [
                  {
                    location: {
                      uri: "file:///p/docs/b.md",
                      range: { start: { line: 0, character: 0 }, end: { line: 0, character: 1 } },
                    },
                    message: "included here",
                  },
                ],
              }),
            ],
          },
        ],
        text: (file) => (file === "/p/docs/a.md" ? text : undefined),
        unsaved: ["/p/docs/a.md"],
      }),
    );
    expect(report).toEqual({
      schema_version: CHECK_SCHEMA_VERSION,
      ascribe_version: "0.3.0",
      error: null,
      files_checked: 12,
      files_reported: 1,
      builds_checked: ["site"],
      diagnostics: [
        {
          code: "ASC036",
          slug: "link-target-missing",
          severity: "error",
          message: "no page at `nowhere.md`",
          file: "docs/a.md",
          range: {
            // "ab😀c" is 5 UTF-16 units, 4 characters, and 7 bytes.
            start: { line: 3, column: 5, offset: 5 + 7 + 7 },
            end: { line: 3, column: 9, offset: 5 + 7 + 11 },
          },
          related: [
            {
              file: "docs/b.md",
              // No text: the protocol's positions, from 1.
              range: {
                start: { line: 1, column: 1, offset: 0 },
                end: { line: 1, column: 2, offset: 0 },
              },
              message: "included here",
            },
          ],
          fixes: [
            {
              title: "Link to `somewhere.md`",
              file: "docs/a.md",
              edits: [
                {
                  range: {
                    start: { line: 3, column: 5, offset: 19 },
                    end: { line: 3, column: 9, offset: 23 },
                  },
                  new_text: "somewhere",
                },
              ],
              applicability: "unsafe",
            },
          ],
          builds: ["site"],
          unpublished: false,
          help: "Fix the link.",
          docs: "https://ascribed-dev.com/reference/diagnostics/#asc036",
          repeats: 0,
        },
      ],
      truncated: false,
      shown: 1,
      total: 1,
      next_command: null,
      summary: { errors: 1, warnings: 0 },
      document_version: 7,
      current: true,
      unsaved: ["docs/a.md"],
    });
  });

  it("lists files in order and caps the list, counting every one", () => {
    const many = Array.from({ length: MAX_PROBLEMS + 5 }, (_, i) =>
      diagnostic({
        severity: i % 2 === 0 ? 1 : 2,
        range: { start: { line: i, character: 0 }, end: { line: i, character: 1 } },
      }),
    );
    const report = problemsReport(
      input({
        publications: [
          { file: "/p/docs/z.md", version: null, at: 0, diagnostics: many },
          { file: "/p/docs/a.md", version: null, at: 0, diagnostics: [diagnostic()] },
        ],
      }),
    );
    expect(report.diagnostics).toHaveLength(MAX_PROBLEMS);
    expect(report.diagnostics[0]?.file).toBe("docs/a.md");
    expect(report.truncated).toBe(true);
    expect(report.total).toBe(MAX_PROBLEMS + 6);
    expect(report.next_command).toBe("ascribe check --editor-build --format json docs/a.md");
    expect(report.summary).toEqual({ errors: 29, warnings: 27 });
  });

  it("is written in the schema version `ascribe check` writes", () => {
    const rust = readFileSync(
      new URL("../../../../crates/ascribe-cli/src/report/json.rs", import.meta.url),
      "utf8",
    );
    expect(/pub const SCHEMA_VERSION: u32 = (\d+);/.exec(rust)?.[1]).toBe(
      String(CHECK_SCHEMA_VERSION),
    );
  });

  it("writes paths as check does, on either platform", () => {
    expect(relativePath("/p", "/p/docs/a.md")).toBe("docs/a.md");
    expect(relativePath("C:\\p", "c:\\p\\docs\\a.md")).toBe("docs/a.md");
  });
});

function page(path: string, over: Partial<ChangedPage> = {}): ChangedPage {
  return {
    path,
    route: `/${path.replace(/\.md$/, "/")}`,
    status: "changed",
    own_file_changed: true,
    because: [],
    page_changed: [],
    counts: { changed: 1, added: 0, removed: 0, moved: 0 },
    title: "A page",
    formatted_title: null,
    ...over,
  } as ChangedPage;
}

describe("the changes report", () => {
  const base = { requested: "origin/main", commit: "abc", merge_base: "def" };

  it("gives each changed page its file, counts, and causes", () => {
    const result: ChangesResult = {
      build: "site",
      base,
      contentRoot: "/p/docs",
      pages: [page("guides/install.md", { because: ["_fragments/prereqs.md"] })],
      problem: null,
    };
    expect(changesReport(result, ROOT, 128)).toEqual({
      build: "site",
      base,
      pull_request: 128,
      content_root: "docs",
      pages: [
        {
          file: "docs/guides/install.md",
          path: "guides/install.md",
          title: "A page",
          route: "/guides/install/",
          status: "changed",
          own_file_changed: true,
          because: ["_fragments/prereqs.md"],
          page_changed: [],
          counts: { changed: 1, added: 0, removed: 0, moved: 0 },
        },
      ],
      truncated: false,
      shown: 1,
      total: 1,
      next_command: null,
    });
  });

  it("caps the pages and names the command that lists them all", () => {
    const result: ChangesResult = {
      build: "site",
      base,
      contentRoot: "/p",
      pages: Array.from({ length: MAX_PAGES + 1 }, (_, i) => page(`p${i}.md`)),
      problem: null,
    };
    const report = changesReport(result, ROOT, undefined);
    expect(report.content_root).toBe(".");
    expect(report.shown).toBe(MAX_PAGES);
    expect(report.pull_request).toBeNull();
    expect(report.next_command).toBe(
      "ascribe diff --base origin/main --build site --pages-only --format json",
    );
  });
});
