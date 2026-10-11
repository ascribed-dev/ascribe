// Whether an agent in the editor's window runs the binary the editor runs
// when it runs `ascribe`. An agent another extension starts (Claude Code's)
// gets the window's own path; one that works in a terminal (VS Code's chat,
// Cursor's) gets the terminals'. When `ascribe` on the window's path is
// another binary or none, the extension puts the binary's folder first on the
// terminals' path, and **Prompt agent**'s prompts name the binary by its
// path. No VS Code in this module, so it's unit-tested as is.

import * as path from "node:path";
import { samePath } from "../projects.js";

/** What finding `ascribe` on the path needs, so tests can fake it. */
export interface PathEnvironment {
  /** The window's `PATH`. */
  readonly path: string | undefined;
  /** The window's `PATHEXT`: the extensions Windows runs a name with. */
  readonly pathext: string | undefined;
  readonly platform: NodeJS.Platform;
  /** The file's real path, through links; `undefined` when there's no such file. */
  realpath(file: string): Promise<string | undefined>;
}

/** Where `ascribe` doesn't run the editor's binary: what an agent runs instead. */
export interface Elsewhere {
  /** The binary's path, for a prompt's commands, with `/` on Windows too. */
  readonly program: string;
  /** The binary's folder, for the terminals' path. */
  readonly directory: string;
}

/** Windows's `PATHEXT` when the window has none. */
const DEFAULT_PATHEXT = ".COM;.EXE;.BAT;.CMD";

/** The file `ascribe` runs on the window's path; `undefined` when it runs none. */
export async function ascribeOnPath(env: PathEnvironment): Promise<string | undefined> {
  const paths = env.platform === "win32" ? path.win32 : path.posix;
  const names =
    env.platform === "win32"
      ? (env.pathext ?? DEFAULT_PATHEXT)
          .split(";")
          .filter((ext) => ext !== "")
          .map((ext) => `ascribe${ext.toLowerCase()}`)
      : ["ascribe"];
  for (const dir of (env.path ?? "").split(paths.delimiter)) {
    if (dir === "") continue;
    for (const name of names) {
      const found = await env.realpath(paths.join(dir, name));
      if (found !== undefined) return found;
    }
  }
  return undefined;
}

/**
 * What an agent runs for `binary` when `ascribe` on the window's path is
 * another binary or none; `undefined` when `ascribe` is it.
 */
export async function ascribeElsewhere(
  binary: string,
  env: PathEnvironment,
): Promise<Elsewhere | undefined> {
  const onPath = await ascribeOnPath(env);
  const real = (await env.realpath(binary)) ?? binary;
  if (onPath !== undefined && samePath(onPath, real)) return undefined;
  const paths = env.platform === "win32" ? path.win32 : path.posix;
  return {
    program: env.platform === "win32" ? binary.replace(/\\/g, "/") : binary,
    directory: paths.dirname(binary),
  };
}
