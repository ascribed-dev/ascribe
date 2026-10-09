// What the extension offers VS Code as an MCP server: `ascribe mcp`, run with
// the binary the workspace's projects resolve. Apart from VS Code, so it can be
// tested by itself; `mcp.ts` registers it.

import type { ResolvedBinary } from "../binary.js";
import { formatVersion } from "../version.js";
import { usesShell } from "../environment.js";

/** The server's name in VS Code's list of MCP servers. */
export const MCP_LABEL = "Ascribe";

/** A project, and the binary it resolves. */
export interface McpCandidate {
  /** The project's name for people. */
  name: string;
  /** Its binary; `undefined` when none was found. */
  binary: ResolvedBinary | undefined;
}

/** How to start the server. */
export interface McpServerSpec {
  label: string;
  command: string;
  args: string[];
  /** Where it starts: the workspace folder, which the tools' paths are relative to. */
  cwd: string;
  /** The binary's version: VS Code reads the tools again when it changes. */
  version: string;
  /** A line for the output channel when the projects resolve different binaries. */
  note: string | undefined;
}

/**
 * The server for the workspace, run with the first project's binary; when a
 * later project resolves another, `note` says which runs. `undefined` when
 * no project has a binary.
 */
export function mcpServerSpec(
  candidates: readonly McpCandidate[],
  cwd: string,
  platform: NodeJS.Platform = process.platform,
): McpServerSpec | undefined {
  const first = candidates.find((c) => c.binary !== undefined);
  if (!first?.binary) return undefined;
  const binary = first.binary;
  const others = candidates.filter((c) => c.binary !== undefined && c.binary.path !== binary.path);
  const note =
    others.length === 0
      ? undefined
      : `The workspace's projects use different ascribe binaries. Its MCP server runs ${binary.path}, ${first.name}'s, for every project; ${others
          .map((c) => `${c.name} uses ${c.binary?.path ?? ""}`)
          .join("; ")}.`;
  // VS Code starts the server without a shell, and a `.cmd` shim, which npm
  // links on Windows, runs only through one.
  const { command, args } = usesShell(binary.path, platform)
    ? { command: "cmd.exe", args: ["/d", "/c", binary.path, "mcp"] }
    : { command: binary.path, args: ["mcp"] };
  return { label: MCP_LABEL, command, args, cwd, version: formatVersion(binary.version), note };
}
