import * as path from "node:path";
import { compareVersions, formatVersion, parseVersion, type Version } from "./version.js";

/** Where a resolved binary came from, in the order they're tried. */
type BinarySource = "setting" | "project" | "bundled";

/** What binary resolution needs from the machine, so tests can fake it. */
export interface BinaryEnvironment {
  /** Whether a regular file exists at the path. */
  fileExists(file: string): Promise<boolean>;
  /** Runs `<file> --version` and returns its standard output. Rejects if it can't run. */
  runVersion(file: string): Promise<string>;
  platform: NodeJS.Platform;
  arch: string;
}

export interface ResolveOptions {
  /** The `ascribe.path` setting. Empty or whitespace means unset. */
  setting: string;
  /** Directories whose `node_modules/.bin` may hold the project's binary, nearest first. */
  projectRoots: readonly string[];
  /** The extension's install directory, which may hold `bin/<platform>-<arch>/`. */
  extensionPath: string;
  /** The oldest server the extension is written for. */
  minVersion: Version;
  env: BinaryEnvironment;
}

/** A binary that runs. */
export interface ResolvedBinary {
  readonly path: string;
  readonly source: BinarySource;
  /** The version it reported. */
  readonly version: Version;
  /** Set when a binary older than the extension expects was found. */
  readonly warning?: string;
}

/** No usable binary: what was tried, and what to do about it. */
interface BinaryNotFound {
  readonly message: string;
  /** Each place looked at, and why it didn't work. */
  readonly tried: readonly string[];
}

export type Resolution =
  | { readonly kind: "found"; readonly binary: ResolvedBinary }
  | { readonly kind: "missing"; readonly error: BinaryNotFound };

/** The binary's file name on a platform. */
export function executableName(platform: NodeJS.Platform): string {
  return platform === "win32" ? "ascribe.exe" : "ascribe";
}

/** The file npm and pnpm link into `node_modules/.bin` on a platform. */
export function projectBinaryName(platform: NodeJS.Platform): string {
  return platform === "win32" ? "ascribe.cmd" : "ascribe";
}

/** The directory of a bundled binary, `bin/<platform>-<arch>`, under the extension. */
export function bundledDirectory(extensionPath: string, platform: NodeJS.Platform, arch: string) {
  return path.join(extensionPath, "bin", `${platform}-${arch}`);
}

/**
 * Every directory from `dir` up to and including `boundary`, nearest first, so
 * a project nested in a monorepo finds the `node_modules` at the repository
 * root. A `dir` that isn't inside `boundary` gives only itself.
 */
export function ancestorsWithin(dir: string, boundary: string): string[] {
  const start = path.resolve(dir);
  const stop = path.resolve(boundary);
  const inside =
    start === stop || start.startsWith(stop.endsWith(path.sep) ? stop : stop + path.sep);
  if (!inside) return [start];
  const result: string[] = [];
  for (let current = start; ; current = path.dirname(current)) {
    result.push(current);
    if (current === stop || path.dirname(current) === current) return result;
  }
}

/**
 * Finds the `ascribe` binary, in order: the `ascribe.path` setting, the
 * project's `node_modules/.bin/ascribe`, and the binary bundled in the
 * extension. Each candidate must run `--version`.
 *
 * An `ascribe.path` that doesn't work is an error, and doesn't fall through:
 * the author asked for that binary, and silently using another would hide the
 * mistake. A project or bundled candidate that doesn't run is skipped.
 *
 * A binary older than `minVersion` is still returned, with a `warning`.
 */
export async function resolveBinary(options: ResolveOptions): Promise<Resolution> {
  const { env } = options;
  const tried: string[] = [];

  const check = async (file: string, source: BinarySource): Promise<ResolvedBinary | undefined> => {
    if (!(await env.fileExists(file))) {
      tried.push(`${file}: not found`);
      return undefined;
    }
    let output: string;
    try {
      output = await env.runVersion(file);
    } catch (error) {
      tried.push(`${file}: couldn't run \`--version\` (${describe(error)})`);
      return undefined;
    }
    const version = parseVersion(output);
    if (!version) {
      tried.push(`${file}: \`--version\` printed "${output.trim()}", which has no version number`);
      return undefined;
    }
    const outdated = compareVersions(version, options.minVersion) < 0;
    return {
      path: file,
      source,
      version,
      ...(outdated && {
        warning:
          `The Ascribe binary at ${file} is version ${formatVersion(version)}, older than the ` +
          `${formatVersion(options.minVersion)} this extension expects. ` +
          `Update it, or some features may not work.`,
      }),
    };
  };

  const setting = options.setting.trim();
  if (setting !== "") {
    const found = await check(setting, "setting");
    if (found) return { kind: "found", binary: found };
    return {
      kind: "missing",
      error: {
        message:
          `The \`ascribe.path\` setting is "${setting}", but that isn't a working Ascribe binary. ` +
          `Fix the path, or clear the setting to use the project's or the bundled binary.`,
        tried,
      },
    };
  }

  for (const root of options.projectRoots) {
    const file = path.join(root, "node_modules", ".bin", projectBinaryName(env.platform));
    const found = await check(file, "project");
    if (found) return { kind: "found", binary: found };
  }

  const bundled = path.join(
    bundledDirectory(options.extensionPath, env.platform, env.arch),
    executableName(env.platform),
  );
  const found = await check(bundled, "bundled");
  if (found) return { kind: "found", binary: found };

  return {
    kind: "missing",
    error: {
      message:
        "Couldn't find the Ascribe binary, so the language server can't start. " +
        "Install it in the project (for example, `npm install --save-dev @ascribed/cli`), " +
        "or set `ascribe.path` to an `ascribe` binary.",
      tried,
    },
  };
}

function describe(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
