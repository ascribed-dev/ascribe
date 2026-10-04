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
