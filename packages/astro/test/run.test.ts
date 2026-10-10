import { chmodSync, mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { describe, expect, it } from "vitest";
import { anchorsFor, hasWarnings, reviewFor, runBuild } from "../src/run.js";

// A stand-in for the compiler: a shell script, so these run where /bin/sh does.
function script(body: string): string {
  const file = path.join(mkdtempSync(path.join(tmpdir(), "ascribe-astro-")), "ascribe");
  writeFileSync(file, `#!/bin/sh\n${body}\n`);
  chmodSync(file, 0o755);
  return file;
}

describe.skipIf(process.platform === "win32")("runBuild", () => {
  it("passes the build, emitter, and config, and returns what was printed", async () => {
    const binary = script('echo "args: $*"; echo "built site/site: 1 page" >&2');
    const result = await runBuild({
      binary,
      configPath: "/p/ascribe.toml",
      build: "site",
      cwd: "/",
    });
    expect(result.diagnostics).toBe(
      "args: build --emit site --build site --config /p/ascribe.toml --color never",
    );
    expect(result.summary).toBe("built site/site: 1 page");
  });

  it("passes --anchors when asked", async () => {
    const binary = script('echo "args: $*"');
    const result = await runBuild({
      binary,
      configPath: "/p/ascribe.toml",
      build: "site",
      cwd: "/",
      anchors: true,
    });
    expect(result.diagnostics).toBe(
      "args: build --emit site --build site --config /p/ascribe.toml --color never --anchors",
    );
  });

  it("writes the outputs asked for", async () => {
    const binary = script('echo "args: $*"');
    const result = await runBuild({
      binary,
      configPath: "/p/ascribe.toml",
      build: "site",
      cwd: "/",
      outputs: ["site", "json"],
    });
    expect(result.diagnostics).toBe(
      "args: build --emit site,json --build site --config /p/ascribe.toml --color never",
    );
  });

  it("rejects with the compiler's report when the build fails", async () => {
    const binary = script(
      'echo "[ASC036] Error: link-target-missing"; echo "error: the build failed" >&2; exit 1',
    );
    const failure = await runBuild({
      binary,
      configPath: "/p/ascribe.toml",
      build: "site",
      cwd: "/",
    }).then(
      () => undefined,
      (error: unknown) => error,
    );
    expect(String(failure)).toContain("link-target-missing");
    expect(String(failure)).toContain("the build failed");
  });
});

describe("anchorsFor", () => {
  it('is off by default, on in dev for "dev", and on everywhere for true', () => {
    expect(anchorsFor(undefined, "dev")).toBe(false);
    expect(anchorsFor(false, "dev")).toBe(false);
    expect(anchorsFor("dev", "dev")).toBe(true);
    expect(anchorsFor("dev", "build")).toBe(false);
    expect(anchorsFor(true, "build")).toBe(true);
  });

  it("is on in dev for review, and never in a build for it", () => {
    expect(anchorsFor(undefined, "dev", true)).toBe(true);
    expect(anchorsFor(false, "dev", true)).toBe(true);
    expect(anchorsFor(undefined, "build", true)).toBe(false);
  });
});

describe("reviewFor", () => {
  it("is on in dev unless turned off, and never in a build", () => {
    expect(reviewFor(undefined, "dev")).toBe(true);
    expect(reviewFor(true, "dev")).toBe(true);
    expect(reviewFor(false, "dev")).toBe(false);
    expect(reviewFor(undefined, "build")).toBe(false);
    expect(reviewFor(true, "build")).toBe(false);
    expect(reviewFor(true, "preview")).toBe(false);
  });
});

describe("hasWarnings", () => {
  it("is false for nothing and for a clean check's summary, which is logged as info", () => {
    expect(hasWarnings("")).toBe(false);
    expect(hasWarnings("checked 14 files: 0 errors, 0 warnings")).toBe(false);
    expect(hasWarnings("checked 1 file: 0 errors, 0 warnings")).toBe(false);
  });

  it("is false for a report with only advice", () => {
    expect(
      hasWarnings(
        "[ASC999] Advice: this page is long\n   ╭─[ docs/keys.md:1:1 ]\n───╯\nchecked 4 files: 0 errors, 0 warnings, 1 advice",
      ),
    ).toBe(false);
  });

  it("is true for a report with warnings, or anything else", () => {
    expect(
      hasWarnings(
        "[ASC041] Warning: this looks like a route\n   ╭─[ docs/keys.md:7:15 ]\n───╯\nchecked 4 files: 0 errors, 1 warning",
      ),
    ).toBe(true);
    expect(hasWarnings("checked 4 files: 0 errors, 2 warnings")).toBe(true);
    expect(hasWarnings("checked 4 files: 0 errors, 1 warning, 2 advice")).toBe(true);
    expect(hasWarnings("something unexpected")).toBe(true);
  });
});
