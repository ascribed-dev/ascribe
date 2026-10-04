// What the overlay says about threads: where each one is, when a comment was
// made, and counts in words.
import type { LocatedThread } from "../place/place.js";
import type { Author } from "../shared/types.js";

/** `"1 comment"`, `"2 comments"`. */
export function plural(count: number, one: string, many = `${one}s`): string {
  return `${count} ${count === 1 ? one : many}`;
}

/** The last segment of a content path. */
export function fileName(path: string): string {
  return path.slice(path.lastIndexOf("/") + 1);
}

/** `rollouts.md:28`, or `rollouts.md:28-30`. */
export function linesLabel(path: string, lines: { first: number; last: number }): string {
  const range = lines.first === lines.last ? `${lines.first}` : `${lines.first}-${lines.last}`;
  return `${fileName(path)}:${range}`;
}

/** Where a thread is, for its card's header. */
export function whereLabel(thread: LocatedThread): string {
  const file = fileName(thread.path);
  if (thread.detached === "file") return `On ${file} as a whole`;
  if (thread.detached === "line-gone") {
    const line = thread.originalLine ?? thread.line;
    const commit = thread.originalCommit?.slice(0, 7);
    if (line === null) return `Was on ${file}, since removed`;
    return `Was on ${file}, line ${line}${commit ? ` at ${commit}` : ""}, since removed`;
  }
  if (thread.lines === undefined) return file;
  if (thread.detached === "no-block")
    return `${linesLabel(thread.path, thread.lines)}, not on this page`;
  if (thread.kind === "review" && thread.side === "LEFT") return file;
  return linesLabel(thread.path, thread.lines);
}

/** Where a thread is, for a list of them: shorter, and naming removed text. */
export function listLabel(thread: LocatedThread): string {
  const file = fileName(thread.path);
  if (thread.detached !== undefined) return `no block, was ${file}`;
  if (thread.kind === "review" && thread.side === "LEFT") return `removed text in ${file}`;
  return thread.lines === undefined ? file : linesLabel(thread.path, thread.lines);
}

/** Whether the thread is on removed text. */
export function onRemovedText(thread: LocatedThread): boolean {
  return thread.kind === "review" && thread.side === "LEFT";
}

/** Whether any comment of the thread is unsent. */
export function hasUnsent(thread: LocatedThread): boolean {
  return thread.comments.some((comment) => comment.pending);
}

/** A comment's author's name, `"you"` for the viewer. */
export function authorName(author: Author | null, viewer: string): string {
  if (author === null) return "ghost";
  return author.login === viewer ? "you" : author.login;
}

/** Two letters standing for an author, for the avatar's place. */
export function initials(author: Author | null): string {
  return (
    (author?.login ?? "?")
      .replace(/[^A-Za-z0-9]/g, "")
      .slice(0, 2)
      .toUpperCase() || "?"
  );
}

/** `"just now"`, `"5 minutes ago"`, `"yesterday"`, or a date. */
export function relativeTime(iso: string, now: number): string {
  const then = Date.parse(iso);
  if (Number.isNaN(then)) return "";
  const seconds = Math.max(0, Math.round((now - then) / 1000));
  if (seconds < 60) return "just now";
  const minutes = Math.round(seconds / 60);
  if (minutes < 60) return `${plural(minutes, "minute")} ago`;
  const hours = Math.round(minutes / 60);
  if (hours < 24) return `${plural(hours, "hour")} ago`;
  const days = Math.round(hours / 24);
  if (days === 1) return "yesterday";
  if (days < 30) return `${days} days ago`;
  return new Date(then).toISOString().slice(0, 10);
}
