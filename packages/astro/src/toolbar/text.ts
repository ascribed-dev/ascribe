// What the toolbar app says, apart from the DOM so it can be tested by itself.
import type { Counts, DiffPage, PageView, ThreadsState } from "../review/protocol.js";

/** "10 changes on this page", or "3 of 10 on this page" while stepping through them. */
export function position(total: number, at: number): string {
  if (total === 0) return "No changes on this page";
  if (at >= 0 && at < total) return `${at + 1} of ${total} on this page`;
  return total === 1 ? "1 change on this page" : `${total} changes on this page`;
}

/** "5 changed · 3 added · 1 removed · 1 moved", leaving out kinds with none. */
export function countsText(counts: Counts): string {
  const parts: string[] = [];
  if (counts.changed > 0) parts.push(`${counts.changed} changed`);
  if (counts.added > 0) parts.push(`${counts.added} added`);
  if (counts.removed > 0) parts.push(`${counts.removed} removed`);
  if (counts.moved > 0) parts.push(`${counts.moved} moved`);
  return parts.join(" · ");
}

/** A changed page's line in the list: its counts, or what happened to it. */
export function pageDetail(page: Omit<DiffPage, "changes">): string {
  if (page.status === "added") return "new page";
  if (page.status === "removed") return "removed";
  const counts = countsText(page.counts);
  if (counts !== "") return counts;
  return page.page_changed.length > 0 ? `${page.page_changed.join(", ")} changed` : "changed";
}

/**
 * The notice about the working tree's errors, or `undefined` with none: a
 * page with an error may render oddly, which shouldn't read as the change.
 */
export function errorsNotice(errors: number): string | undefined {
  if (errors <= 0) return undefined;
  const one = errors === 1;
  return (
    `The working tree has ${one ? "1 error" : `${errors} errors`}, so a page may not show as ` +
    `it will once ${one ? "it's" : "they're"} fixed. \`ascribe check\` lists ${one ? "it" : "them"}.`
  );
}

/** "1 unsent comment", "2 unsent comments". */
export function unsentText(count: number): string {
  return count === 1 ? "1 unsent comment" : `${count} unsent comments`;
}

/** The toolbar button's label: "Ascribe review", with the unsent count when there is one. */
export function buttonLabel(unsent: number): string {
  return unsent > 0 ? `Ascribe review · ${unsentText(unsent)}` : "Ascribe review";
}

/** The base's name and the pull request, as the panel's first words: "#128 against main". */
export function againstText(view: PageView): { pullRequest: string | null; base: string | null } {
  const pullRequest = view.threads.state === "on" ? `#${view.threads.pullRequest.number}` : null;
  const requested = view.base?.requested ?? null;
  // The pull request's base by its name ("main"), not the remote branch compared with ("origin/main").
  const name = view.threads.state === "on" ? view.threads.pullRequest.baseRefName : undefined;
  const base =
    requested !== null && name !== undefined && requested.endsWith(`/${name}`) ? name : requested;
  return { pullRequest, base };
}

/** A sentence about the threads: why there are none, or how the checkout differs. */
export function threadsNotice(threads: ThreadsState): string | undefined {
  if (threads.state !== "on") return threads.message;
  const { local, pullRequest } = threads;
  const name = `#${pullRequest.number}`;
  const commits = (n: number) => (n === 1 ? "1 commit" : `${n} commits`);
  switch (local.state) {
    case "same":
      return undefined;
    case "behind":
      return `Your checkout is ${commits(local.behind)} behind ${name}, so some comments may be on lines you don't have. Pull to see the latest.`;
    case "ahead":
      return `You have ${commits(local.ahead)} that ${local.ahead === 1 ? "isn't" : "aren't"} pushed. You can comment only on lines that are on GitHub.`;
    case "diverged":
      return `Your checkout and ${name} have both moved on (${local.behind} behind, ${local.ahead} ahead). Some comments may be on lines you don't have, and you can comment only on lines that are on GitHub.`;
    case "missing":
      return `Your checkout doesn't have ${name}'s latest commit, so some comments may be on lines you don't have. Fetch to see the latest.`;
  }
}

/**
 * The changed page to offer after `current` (a content path): the next in
 * path order, or past the last, the first (`first`). Removed pages are
 * skipped. `null` when no other page changed.
 */
export function nextChangedPage(
  pages: readonly Omit<DiffPage, "changes">[],
  current: string | null,
): { page: Omit<DiffPage, "changes">; first: boolean } | null {
  const open = pages.filter((p) => p.status !== "removed" && p.path !== current);
  const first = open[0];
  if (first === undefined) return null;
  const after = current === null ? undefined : open.find((p) => p.path > current);
  return after ? { page: after, first: false } : { page: first, first: true };
}

/** The hash that tells a page to go to a thread once it's drawn. */
export const THREAD_HASH = "#ascribe-thread=";
