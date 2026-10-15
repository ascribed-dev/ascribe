// What the preview's page may ask of the extension (`src/preview/messages.ts`):
// each message it sends passes, and each with a hostile value is dropped.
import * as path from "node:path";
import { describe, expect, it } from "vitest";
import { checkMessage, GIT_COMMANDS, MAX_TEXT, mayOpen } from "../../src/preview/messages.js";
import type { FromWebview } from "../../src/preview/protocol.js";

const REPORT = {
  headings: ["install"],
  anchored: 3,
  elements: { "ascribe-tabs": 1 },
  elementsDefined: true,
  violations: [],
  marks: { added: 1 },
  reviewHeader: null,
};

/** One message of each type, as the page sends it. */
const VALID: { [T in FromWebview["type"]]: Extract<FromWebview, { type: T }> } = {
  ready: { type: "ready" },
  build: { type: "build", name: "cloud" },
  open: { type: "open", href: "./install/#rotate" },
  showOutput: { type: "showOutput" },
  showProblems: { type: "showProblems" },
  rendered: { type: "rendered", seq: 4, report: REPORT },
  revealedLine: { type: "revealedLine", line: 12, source: "index.md:12-14" },
  scrolled: { type: "scrolled", line: 0 },
  openLine: { type: "openLine", line: 7 },
  images: { type: "images", seq: 4, images: [{ src: "a.png", loaded: true, width: 10 }] },
  openSource: { type: "openSource", source: "index.md:1-3" },
  openFile: { type: "openFile", path: "/work/docs/content/_fragments/x.md" },
  atEnd: { type: "atEnd" },
  openPage: { type: "openPage", path: "install.md" },
  threads: { type: "threads", id: 1, method: "reply", params: { threadId: "T", body: "Yes" } },
  openThread: { type: "openThread", threadId: "T", path: "install.md" },
  notify: { type: "notify", message: "Reply sent to GitHub." },
  signIn: { type: "signIn" },
  useGh: { type: "useGh" },
  git: { type: "git", command: "pull" },
  refreshThreads: { type: "refreshThreads" },
  threadsDrawn: {
    type: "threadsDrawn",
    report: { blocks: { "index.md:1-2": ["T"] }, detached: [], unsent: 0 },
  },
  surface: { type: "surface", surface: "site" },
  siteShown: { type: "siteShown", url: "http://localhost:4321/" },
};

const LONG = "x".repeat(MAX_TEXT + 1);

/** For each type, values that a page that isn't the preview's could send. */
const HOSTILE: Record<FromWebview["type"], Record<string, unknown>[]> = {
  ready: [],
  build: [{ name: 3 }, { name: LONG }, {}],
  open: [{ href: { toString: "x" } }, { href: LONG }],
  showOutput: [],
  showProblems: [],
  rendered: [
    { seq: "4", report: REPORT },
    { seq: 4, report: null },
    { seq: 4, report: { ...REPORT, headings: "install" } },
    { seq: 4, report: { ...REPORT, violations: Array(20_000).fill("v") } },
  ],
  revealedLine: [{ line: -1, source: "a" }, { line: 1.5, source: "a" }, { line: 1 }],
  scrolled: [{ line: Number.NaN }, { line: "1" }, { line: 2 ** 60 }],
  openLine: [{ line: Infinity }, { line: null }],
  images: [
    { seq: 1, images: "a.png" },
    { seq: 1, images: [{ src: 1 }] },
  ],
  openSource: [{ source: ["index.md:1-3"] }, { source: LONG }],
  openFile: [{ path: null }, { path: LONG }, { path: ["/etc/passwd"] }],
  atEnd: [],
  openPage: [{ path: 1 }, { path: LONG }],
  threads: [
    { id: 1, method: "deleteRepository", params: {} },
    { id: -1, method: "load", params: {} },
    { id: 1, method: "load", params: "anchors" },
    { id: 1, method: "load", params: [] },
    { id: 1, method: "promptAgent", params: { request: { kind: "shell", command: "rm" } } },
    { id: 1, method: "promptAgent", params: { request: { kind: "thread" } } },
    { id: 1, method: "promptAgent", params: {} },
  ],
  openThread: [{ threadId: 1, path: "a.md" }, { threadId: "T" }],
  notify: [{ message: LONG }, { message: 1 }],
  signIn: [],
  useGh: [],
  git: [
    { command: "checkout" },
    { command: "clone" },
    { command: "pull; rm -rf /" },
    { command: "Pull" },
    { command: ["pull"] },
    {},
  ],
  refreshThreads: [],
  threadsDrawn: [{ report: { blocks: [], detached: [], unsent: 0 } }, { report: "drawn" }],
  surface: [{ surface: "elsewhere" }, {}],
  siteShown: [{ url: 7 }, { url: LONG }],
};

describe("the preview's messages", () => {
  it("takes every message the page sends", () => {
    for (const message of Object.values(VALID)) expect(checkMessage(message)).toBe(message);
  });

  it("drops a message with a hostile value, for every type", () => {
    for (const [type, values] of Object.entries(HOSTILE)) {
      for (const value of values) {
        expect(checkMessage({ type, ...value }), `${type} ${JSON.stringify(value)}`).toBe(
          undefined,
        );
      }
    }
  });

  it("drops what isn't a message, or is of a type it doesn't take", () => {
    for (const value of [
      null,
      undefined,
      "ready",
      ["ready"],
      {},
      { type: 1 },
      { type: "toString" },
      { type: "__proto__" },
      { type: "constructor" },
      { type: "runCommand", command: "workbench.action.terminal.new" },
    ]) {
      expect(checkMessage(value), JSON.stringify(value)).toBeUndefined();
    }
  });

  it("runs only the git commands the preview has buttons for", () => {
    expect(GIT_COMMANDS).toEqual({ pull: "git.pull", push: "git.push", fetch: "git.fetch" });
  });
});

describe("the files the page may open", () => {
  const folder = path.resolve("/work/docs");

  it("opens a file inside the project", () => {
    expect(mayOpen(path.join(folder, "content", "a.md"), { folder })).toBe(true);
    expect(mayOpen(path.join(folder, "ascribe.toml"), { folder })).toBe(true);
  });

  it("opens a file the extension gave the page", () => {
    const given = path.resolve("/work/shared/x.md");
    expect(mayOpen(given, { given: [given], folder })).toBe(true);
  });

  it("doesn't open a file outside the project", () => {
    for (const file of [
      path.resolve("/etc/passwd"),
      path.join(folder, "..", "other", "a.md"),
      `${folder}-other${path.sep}a.md`,
      path.resolve("/work/shared/x.md"),
      "a.md",
    ]) {
      expect(mayOpen(file, { given: [], folder }), file).toBe(false);
    }
    expect(mayOpen(path.join(folder, "a.md"), { folder: undefined })).toBe(false);
  });
});
