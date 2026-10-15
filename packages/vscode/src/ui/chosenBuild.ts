// The build you're looking at, one per project for the session: the preview
// renders it and the status bar names it, and the preview's picker and the
// status bar's Switch Build both set it. No VS Code in this module.

import { Emitter, type Event } from "../emitter.js";
import { comparable } from "../projects.js";

/**
 * The build chosen in each project, by project folder. A project with no
 * choice is looking at its editor's build (`[editor] build`), and follows it
 * when it changes.
 */
export class ChosenBuilds {
  private readonly chosen = new Map<string, string>();
  private readonly changed = new Emitter<string>();

  /** Fires with a project's folder when its choice changes. */
  get onDidChange(): Event<string> {
    return this.changed.event;
  }

  get(folder: string): string | undefined {
    return this.chosen.get(comparable(folder));
  }

  /** Records a project's choice; `undefined` goes back to its editor's build. */
  set(folder: string, build: string | undefined): void {
    const key = comparable(folder);
    if (this.chosen.get(key) === build) return;
    if (build === undefined) this.chosen.delete(key);
    else this.chosen.set(key, build);
    this.changed.fire(folder);
  }

  /**
   * Chooses a build by name. Choosing the editor's build is choosing the
   * default, so a later change of `[editor] build` is followed.
   */
  choose(folder: string, build: string, editorBuild: string | undefined): void {
    this.set(folder, build === editorBuild ? undefined : build);
  }

  /**
   * The build a project is looking at, given its builds: the chosen one, or
   * the editor's when there's no choice or the content model no longer has it.
   * With no builds known, the chosen one.
   */
  shown(folder: string, builds: readonly ListedBuild[]): string | undefined {
    const chosen = this.get(folder);
    if (builds.length === 0) return chosen;
    if (chosen !== undefined && choices(builds).some((choice) => choice.name === chosen)) {
      return chosen;
    }
    return builds.find((build) => build.editor)?.name;
  }

  dispose(): void {
    this.changed.dispose();
  }
}

/** A build as the server lists it: a versioned build with its lines. */
export interface ListedBuild {
  name: string;
  editor: boolean;
  lines?: readonly { name: string; line: string; current: boolean }[];
}

/** A build a picker offers: a build, or a line of a versioned build. */
export interface BuildChoice {
  /** The build's name: `site`, or a line's, `site@3.4`. */
  name: string;
  /** Whether it's the editor's build. */
  editor: boolean;
  /** For a line, the line, and whether it's the current one. */
  line?: { line: string; current: boolean; build: string };
}

/**
 * The builds a picker offers, in the server's order, a versioned build's
 * lines under it, the current one first: the current line is the build
 * itself.
 */
export function choices(builds: readonly ListedBuild[]): BuildChoice[] {
  const out: BuildChoice[] = [];
  for (const build of builds) {
    const lines = build.lines ?? [];
    if (lines.length === 0) {
      out.push({ name: build.name, editor: build.editor });
      continue;
    }
    for (const line of lines) {
      out.push({
        name: line.name,
        editor: line.name === build.name && build.editor,
        line: { line: line.line, current: line.current, build: build.name },
      });
    }
  }
  return out;
}

/** What a picker says about a line: `line 3.5, the current line`. */
export function lineDescription(choice: BuildChoice): string {
  if (!choice.line) return "";
  return choice.line.current
    ? `line ${choice.line.line}, the current line`
    : `line ${choice.line.line} of ${choice.line.build}`;
}
