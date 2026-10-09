// What agents in VS Code get from the extension, apart from VS Code: the
// MCP server it offers.
import { describe, expect, it } from "vitest";
import { mcpServerSpec } from "../../src/agents/mcpServer.js";
import type { ResolvedBinary } from "../../src/binary.js";

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
