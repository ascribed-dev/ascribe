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
    const binary = script('echo "args: $*"; echo "built site: 1 page, 0 assets (site)" >&2');
    const result = await runBuild({
      binary,
      configPath: "/p/ascribe.toml",
      build: "site",
      cwd: "/",
    });
    expect(result.diagnostics).toBe(
      "args: build --emit site --build site --config /p/ascribe.toml --color never",
    );
    expect(result.summary).toBe("built site: 1 page, 0 assets (site)");
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
    expect(hasWarnings("checked 14 files in build `site`: no problems")).toBe(false);
    expect(hasWarnings("checked 14 files in build `site`: no problems, 2 acknowledged")).toBe(
      false,
    );
  });

  it("is false for a summary with only advice", () => {
    expect(
      hasWarnings(
        "checked 4 files in build `site`: 0 errors, 0 warnings, 1 advice: ascribe check --config /p/ascribe.toml --build site for each",
      ),
    ).toBe(false);
  });

  it("is true for a summary with warnings, or anything else", () => {
    expect(
      hasWarnings(
        "checked 4 files in build `site`: 0 errors, 1 warning, 0 advice: ascribe check --build site for each",
      ),
    ).toBe(true);
    expect(
      hasWarnings(
        "checked 4 files in builds `site` and `site@3.4`: 0 errors, 2 warnings, 2 advice, 1 acknowledged: ascribe check --build site --build site@3.4 for each",
      ),
    ).toBe(true);
    expect(hasWarnings("checked 4 files in build `site`: 1 error, 0 warnings, 0 advice")).toBe(
      true,
    );
    expect(hasWarnings("something unexpected")).toBe(true);
  });
});
