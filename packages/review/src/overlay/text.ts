// What the overlay says about threads: where each one is, when a comment was
// made, and counts in words.
import type { LocatedThread } from "../place/place.js";
import type { Author } from "../shared/types.js";

/** `"1 comment"`, `"2 comments"`. */
export function plural(count: number, one: string, many = `${one}s`): string {
  return `${count} ${count === 1 ? one : many}`;
}

/** The last segment of a content path. */
function fileName(path: string): string {
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

/** Elements whose text starts on a line of its own. */
const BLOCK_TAGS = new Set([
  "ADDRESS",
  "ARTICLE",
  "ASIDE",
  "BLOCKQUOTE",
  "DD",
  "DETAILS",
  "DIV",
  "DL",
  "DT",
  "FIGCAPTION",
  "FIGURE",
  "FOOTER",
  "H1",
  "H2",
  "H3",
  "H4",
  "H5",
  "H6",
  "HEADER",
  "HR",
  "LI",
  "OL",
  "P",
  "PRE",
  "SECTION",
  "SUMMARY",
  "TABLE",
  "TR",
  "UL",
]);

/** Classes that hide text from sight but not from screen readers. */
const SCREEN_READER_ONLY = ["sr-only", "visually-hidden", "sl-sr-only"];

/** Whether a reader can't see `el`. */
function unseen(el: Element): boolean {
  if (
    el.hasAttribute("hidden") ||
    el.getAttribute("aria-hidden") === "true" ||
    SCREEN_READER_ONLY.some((name) => el.classList.contains(name))
  ) {
    return true;
  }
  const style = el.ownerDocument.defaultView?.getComputedStyle(el);
  return style?.display === "none" || style?.visibility === "hidden";
}

/**
 * A block's text as a reader sees it, for quoting: links as their text, a
 * line for each paragraph, list item, or row, numbered items with their
 * numbers, images as their alt text, and code as written. What the marks and
 * the overlay add is left out, but for the words a change inserted, and so
 * is anything a reader can't see.
 */
export function blockText(block: Element): string {
  // Each line, and whether it's code, which keeps its spaces.
  const lines: { text: string; pre: boolean }[] = [];
  let line = "";
  let linePre = false;
  const breakLine = (): void => {
    if (line.trim() !== "") lines.push({ text: line, pre: linePre });
    line = "";
    linePre = false;
  };
  const walk = (node: Node, pre: boolean): void => {
    if (node.nodeType === 3) {
      const text = node.nodeValue ?? "";
      if (!pre) {
        line += text.replace(/\s+/g, " ");
        return;
      }
      const [first = "", ...rest] = text.split("\n");
      line += first;
      linePre = true;
      for (const next of rest) {
        lines.push({ text: line, pre: true });
        line = next;
      }
      return;
    }
    if (!(node instanceof Element)) return;
    const tag = node.tagName.toUpperCase();
    if (
      node.hasAttribute("data-ascribe-overlay") ||
      (node.hasAttribute("data-ascribe-ui") && tag !== "INS") ||
      ["SCRIPT", "STYLE", "TEMPLATE", "NOSCRIPT"].includes(tag) ||
      unseen(node)
    ) {
      return;
    }
    if (tag === "BR") {
      breakLine();
      return;
    }
    if (tag === "IMG") {
      line += node.getAttribute("alt") ?? "";
      return;
    }
    const block = BLOCK_TAGS.has(tag);
    if (block) breakLine();
    if ((tag === "TD" || tag === "TH") && node.previousElementSibling !== null) line += " | ";
    if (tag === "LI" && node.parentElement?.tagName.toUpperCase() === "OL") {
      const list = node.parentElement;
      const start = Number(list.getAttribute("start") ?? "1");
      const items = Array.from(list.children).filter((c) => c.tagName.toUpperCase() === "LI");
      line += `${(Number.isFinite(start) ? start : 1) + items.indexOf(node)}. `;
    }
    for (const child of Array.from(node.childNodes)) walk(child, pre || tag === "PRE");
    if (block) breakLine();
  };
  walk(block, false);
  breakLine();
  return lines.map(({ text, pre }) => (pre ? text.trimEnd() : text.trim())).join("\n");
}
