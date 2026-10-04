// What review says about a pull request's threads, apart from VS Code so it
// can be tested by itself, and shared with the webview: the header's notices,
// which pages show a thread, and whether the source editor shows threads.

import type { ChangedPage, ThreadsView } from "./protocol.js";

/** "1 commit", "3 commits". */
function commits(count: number): string {
  return count === 1 ? "1 commit" : `${count} commits`;
}

/** A notice under the review header: what it says, and the button that helps, if any. */
export interface ThreadsNotice {
  text: string;
  actions: { label: string; message: "signIn" | "useGh" | "pull" | "push" | "fetch" | "refresh" }[];
}

/** The header's notice about the threads, or `undefined` when there's nothing to say. */
export function threadsNotice(view: ThreadsView): ThreadsNotice | undefined {
  if (view.state === "signed-out") {
    return {
      text: "Showing changes only. Comments need GitHub.",
      actions: [
        { label: "Sign in to see comments", message: "signIn" },
        ...(view.gh ? [{ label: "Use GitHub CLI", message: "useGh" as const }] : []),
      ],
    };
  }
  if (view.state === "error") {
    return {
      text: `Couldn't read the comments. ${view.message ?? ""}`.trim(),
      actions: [{ label: "Try again", message: "refresh" }],
    };
  }
  const local = view.local;
  const pr = view.pullRequest;
  if (!local || !pr) return undefined;
  const name = `#${pr.number}`;
  switch (local.state) {
    case "same":
      return undefined;
    case "behind":
      return {
        text: `Your checkout is ${commits(local.behind)} behind ${name}, so some comments may be on lines you don't have.`,
        actions: [{ label: "Pull", message: "pull" }],
      };
    case "ahead":
      return {
        text: `You have ${commits(local.ahead)} that ${local.ahead === 1 ? "isn't" : "aren't"} pushed. You can comment only on lines that are on GitHub.`,
        actions: [{ label: "Push", message: "push" }],
      };
    case "diverged":
      return {
        text: `Your checkout and ${name} have both moved on (${local.behind} behind, ${local.ahead} ahead). Some comments may be on lines you don't have, and you can comment only on lines that are on GitHub.`,
        actions: [{ label: "Pull", message: "pull" }],
      };
    case "missing":
      return {
        text: `Your checkout doesn't have ${name}'s latest commit, so some comments may be on lines you don't have.`,
        actions: [{ label: "Fetch", message: "fetch" }],
      };
  }
}

/** The header's opening words: "#128 against main", or "Against main" without a pull request. */
export function againstText(view: ThreadsView | null): string {
  const pr = view?.pullRequest;
  return pr ? `#${pr.number} against ` : "Against ";
}

/**
 * The changed pages that show a file (a content path): the file's own page,
 * and the pages that changed through it (an include). `[]` when none does.
 */
export function pagesShowing(
  file: string,
  pages: readonly ChangedPage[],
): { path: string; title: string | null }[] {
  return pages
    .filter((page) => page.status !== "removed")
    .filter((page) => page.path === file || page.because.includes(file))
    .sort((a, b) => Number(b.path === file) - Number(a.path === file))
    .map((page) => ({ path: page.path, title: page.title }));
}

/** The `ascribe.review.sourceComments` setting. */
export type SourceCommentsSetting = "auto" | "on" | "off";

/**
 * Whether the source editor shows review threads: with `auto`, only when the
 * GitHub Pull Requests extension isn't active, since it shows them already.
 */
export function showSourceComments(setting: unknown, pullRequestsActive: boolean): boolean {
  if (setting === "on") return true;
  if (setting === "off") return false;
  return !pullRequestsActive;
}

/**
 * A comment's Markdown with each image made a link to it, as the overlay
 * shows them, so the editor loads nothing until the reader asks. Code keeps
 * its text.
 */
export function imagesAsLinks(body: string): string {
  let fence: string | undefined;
  return body
    .split("\n")
    .map((line) => {
      const marker = /^ {0,3}(`{3,}|~{3,})/.exec(line)?.[1];
      if (fence !== undefined) {
        if (marker?.startsWith(fence)) fence = undefined;
        return line;
      }
      if (marker !== undefined) {
        fence = marker;
        return line;
      }
      return inlineImagesAsLinks(line);
    })
    .join("\n");
}

/** One line's images made links, its code spans kept: in one pass, as a comment is anyone's text. */
function inlineImagesAsLinks(line: string): string {
  let out = "";
  /** By backtick run length: the first index from which no run of that length closes a span. */
  const none = new Map<number, number>();
  let i = 0;
  while (i < line.length) {
    const ch = line[i];
    if (ch === "\\") {
      out += line.slice(i, i + 2);
      i += 2;
    } else if (ch === "`") {
      let run = 1;
      while (line[i + run] === "`") run++;
      const close = (none.get(run) ?? Infinity) <= i + run ? -1 : closingRun(line, i + run, run);
      if (close < 0) {
        if (!none.has(run)) none.set(run, i + run);
        out += line.slice(i, i + run);
        i += run;
      } else {
        out += line.slice(i, close + run);
        i = close + run;
      }
    } else if (ch === "!" && line[i + 1] === "[") {
      const empty = line[i + 2] === "]";
      out += empty ? "[Image]" : "[Image: ";
      i += empty ? 3 : 2;
    } else {
      out += ch;
      i++;
    }
  }
  return out;
}

/** Where a run of exactly `run` backticks starts at or after `from`, or -1. */
function closingRun(line: string, from: number, run: number): number {
  const ticks = "`".repeat(run);
  let at = line.indexOf(ticks, from);
  while (at >= 0) {
    let end = at + run;
    if (line[end] !== "`") return at;
    while (line[end] === "`") end++;
    at = line.indexOf(ticks, end);
  }
  return -1;
}

/**
 * The git revision to compare with for a pull request's base: the remote
 * branch, from the remote whose URL is the pull request's repository, when
 * there is one; else the branch's name.
 */
export function pullRequestBase(
  remotes: readonly { name: string; repository: string | undefined }[],
  repository: string,
  baseRefName: string,
): string {
  const remote =
    remotes.find((r) => r.repository === repository && r.name === "upstream") ??
    remotes.find((r) => r.repository === repository && r.name === "origin") ??
    remotes.find((r) => r.repository === repository);
  return remote ? `${remote.name}/${baseRefName}` : baseRefName;
}
