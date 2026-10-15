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
  shown(folder: string, builds: readonly { name: string; editor: boolean }[]): string | undefined {
    const chosen = this.get(folder);
    if (builds.length === 0) return chosen;
    if (chosen !== undefined && builds.some((build) => build.name === chosen)) return chosen;
    return builds.find((build) => build.editor)?.name;
  }

  dispose(): void {
    this.changed.dispose();
  }
}
