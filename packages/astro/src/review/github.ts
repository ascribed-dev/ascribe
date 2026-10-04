// The pull request's review, through the GitHub CLI: `@ascribed/review`'s
// session with `ghTransport`, found for the project's checkout. Loaded only
// when review starts, so `astro build` never reads it.
import { execFile } from "node:child_process";
import type { ReviewSession } from "@ascribed/review/github";
import type { LocalState, ThreadsState } from "./protocol.js";

/** A project's connection to its pull request: a session, or why there's none. */
export type Connection =
  | {
      state: "on";
      session: ReviewSession;
      /** The git revision for the pull request's base: `origin/main`. */
      base: string;
      local: { state: LocalState; behind: number; ahead: number };
    }
  | Exclude<ThreadsState, { state: "on" }>;

const CHANGES_ONLY = "Showing changes only.";

/** Finds the checkout's pull request and opens a review of it with the GitHub CLI's sign-in. */
export async function connect(projectDir: string): Promise<Connection> {
  const github = await import("@ascribed/review/github");
  let checkout;
  try {
    checkout = await github.readCheckout(projectDir);
  } catch {
    return {
      state: "no-pull-request",
      message: `${CHANGES_ONLY} Comments need a git checkout with a pull request on GitHub.`,
    };
  }
  const host = checkout.bases[0]?.host;
  if (host === undefined) {
    return {
      state: "no-pull-request",
      message: `${CHANGES_ONLY} Comments need the branch pushed to GitHub, with a pull request.`,
    };
  }
  let session: ReviewSession | undefined;
  try {
    session = await github.openReview({ projectDir });
  } catch (error) {
    const code = error instanceof github.ReviewError ? error.code : undefined;
    if (code === "gh-missing" || code === "gh-too-old") {
      return {
        state: "gh-missing",
        message: `${CHANGES_ONLY} Comments need the GitHub CLI: install gh, run \`gh auth login\`, then Refresh.`,
      };
    }
    if (code === "not-signed-in") {
      return {
        state: "signed-out",
        message: `${CHANGES_ONLY} Comments need GitHub: run \`gh auth login --hostname ${host}\`, then Refresh.`,
      };
    }
    return { state: "error", message: messageOf(error) };
  }
  if (session === undefined) {
    return {
      state: "no-pull-request",
      message: `${CHANGES_ONLY} This branch has no open pull request: open one to comment here, then Refresh.`,
    };
  }
  const pr = session.pullRequest;
  return {
    state: "on",
    session,
    base: await github.baseRevision(checkout.root, pr),
    local: { state: pr.local, ...(await aheadBehind(checkout.root, pr.headOid)) },
  };
}

/** How many commits `HEAD` is ahead of and behind the pull request's head. */
function aheadBehind(root: string, head: string): Promise<{ ahead: number; behind: number }> {
  return new Promise((resolve) => {
    execFile(
      "git",
      ["rev-list", "--left-right", "--count", `HEAD...${head}`],
      { cwd: root, timeout: 15_000, windowsHide: true, encoding: "utf8" },
      (error, stdout) => {
        const [ahead, behind] = (error ? "" : stdout).trim().split(/\s+/).map(Number);
        resolve({ ahead: ahead || 0, behind: behind || 0 });
      },
    );
  });
}

export function messageOf(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
