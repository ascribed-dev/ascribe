// The build lens: which projects have it on, and what it shows for a page,
// worked out from `ascribe/buildView`'s answer. No VS Code in this module.

import { Emitter, type Event } from "../emitter.js";
import { comparable } from "../projects.js";
import type { BuildViewResult, LspRange } from "../shapes.js";

/**
 * The projects whose build lens is on, by project folder, for the session.
 * The lens has no build of its own: it dims by the project's chosen build
 * (`ChosenBuilds`), the one the preview renders.
 */
export class BuildLenses {
  private readonly on = new Set<string>();
  private readonly changed = new Emitter<string>();

  /** Fires with a project's folder when its lens is turned on or off. */
  get onDidChange(): Event<string> {
    return this.changed.event;
  }

  isOn(folder: string): boolean {
    return this.on.has(comparable(folder));
  }

  set(folder: string, on: boolean): void {
    const key = comparable(folder);
    if (this.on.has(key) === on) return;
    if (on) this.on.add(key);
    else this.on.delete(key);
    this.changed.fire(folder);
  }

  /** Turns the lens on if it's off, and off if it's on; returns whether it's on now. */
  toggle(folder: string): boolean {
    const on = !this.isOn(folder);
    this.set(folder, on);
    return on;
  }

  dispose(): void {
    this.changed.dispose();
  }
}

/** A range the lens dims, with what its hover says. */
interface Dimmed {
  range: LspRange;
  /** Markdown. */
  hover: string;
}

/** What the lens shows on a page. */
export interface LensView {
  /** The ranges to dim. */
  dimmed: Dimmed[];
  /** The line at the top of the page, when the build leaves out the whole page. */
  banner: string | undefined;
}

/**
 * Text from the server, written as Markdown text rather than read as
 * Markdown: inside a line that starts with bold text, only these characters
 * mean anything.
 */
function escape(text: string): string {
  return text.replace(/[\\`*_[\]<>]/g, "\\$&");
}

/**
 * What the lens shows for an answer: each excluded range dimmed, with a hover
 * that names the build and says why. When the build leaves out the whole
 * page, the line at the top says so, and the whole page is dimmed.
 */
export function lensView(result: BuildViewResult, lineCount: number): LensView {
  if (result.build === "") return { dimmed: [], banner: undefined };
  if (!result.pageIncluded) {
    const banner = result.pageDetail ?? `The build ${result.build} doesn't publish this page.`;
    const last = Math.max(lineCount - 1, 0);
    return {
      dimmed: [
        {
          range: { start: { line: 0, character: 0 }, end: { line: last + 1, character: 0 } },
          hover: `**Left out of ${escape(result.build)}.** ${escape(banner)}`,
        },
      ],
      banner,
    };
  }
  return {
    dimmed: result.excluded.map((excluded) => ({
      range: excluded.range,
      hover: `**Left out of ${escape(result.build)}.** ${escape(excluded.detail)}.`,
    })),
    banner: undefined,
  };
}
