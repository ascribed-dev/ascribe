import { chmodSync, mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { describe, expect, it } from "vitest";
import { runBuild } from "../src/run.js";

// A stand-in for the compiler: a shell script, so these run where /bin/sh does.
function script(body: string): string {
  const file = path.join(mkdtempSync(path.join(tmpdir(), "tessera-astro-")), "tessera");
  writeFileSync(file, `#!/bin/sh\n${body}\n`);
  chmodSync(file, 0o755);
  return file;
}

describe.skipIf(process.platform === "win32")("runBuild", () => {
  it("passes the build, emitter, and config, and returns what was printed", async () => {
    const binary = script('echo "args: $*"; echo "built site/site: 1 page" >&2');
    const result = await runBuild({
      binary,
      configPath: "/p/tessera.toml",
      build: "site",
      cwd: "/",
    });
    expect(result.diagnostics).toBe(
      "args: build --emit site --build site --config /p/tessera.toml --color never",
    );
    expect(result.summary).toBe("built site/site: 1 page");
  });

  it("rejects with the compiler's report when the build fails", async () => {
    const binary = script(
      'echo "[TSR036] Error: link-target-missing"; echo "error: the build failed" >&2; exit 1',
    );
    const failure = await runBuild({
      binary,
      configPath: "/p/tessera.toml",
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
