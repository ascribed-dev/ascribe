// What a check of every build adds to the Problems panel: the problems
// `ascribe check` finds that the language server, which checks the editor's
// build only, doesn't show. No VS Code in this module.

import * as path from "node:path";
import type { CheckReport, Entry, Pos, Range } from "./shapes.js";

/** A position as the editor has it: from 0, in UTF-16 units. */
export interface EditorPosition {
  line: number;
  character: number;
}

/** A span as the editor has it. */
export interface EditorRange {
  start: EditorPosition;
  end: EditorPosition;
}

/** A problem for the Problems panel, with its places as the editor has them. */
export interface PanelProblem {
  entry: Entry;
  /** The file, as an absolute path. */
  file: string;
  range: EditorRange;
  related: { file: string; range: EditorRange; message: string }[];
}

/**
 * A position of `ascribe check`'s (a line from 1, and a column from 1 in
 * Unicode characters) in `text`, as the editor has it. One past the text's
 * end is clamped to it.
 */
export function editorPosition(text: string, pos: Pos): EditorPosition {
  const lines = text.split("\n");
  const line = Math.min(Math.max(pos.line - 1, 0), Math.max(lines.length - 1, 0));
  let character = 0;
  let column = 1;
  for (const char of lines[line] ?? "") {
    if (column >= pos.column) break;
    character += char.length;
    column++;
  }
  return { line, character };
}

function editorRange(text: string, range: Range): EditorRange {
  return { start: editorPosition(text, range.start), end: editorPosition(text, range.end) };
}

/**
 * The problems of a report the Problems panel doesn't show yet: those
 * `inPanel` doesn't find at the same place with the same code. `text`
 * gives a file's text as the check read it, by absolute path; a problem in
 * a file it can't give is placed at the file's start.
 */
export function panelProblems(
  report: CheckReport,
  root: string,
  text: (file: string) => string | undefined,
  inPanel: (file: string, code: string, start: EditorPosition) => boolean,
): PanelProblem[] {
  const place = (file: string, range: Range) => {
    const absolute = path.resolve(root, file);
    const content = text(absolute) ?? "";
    return { file: absolute, range: editorRange(content, range) };
  };
  const problems: PanelProblem[] = [];
  for (const entry of report.diagnostics) {
    const { file, range } = place(entry.file, entry.range);
    if (inPanel(file, entry.code, range.start)) continue;
    const related = entry.related.map((r) => ({ ...place(r.file, r.range), message: r.message }));
    problems.push({ entry, file, range, related });
  }
  return problems;
}

/** What the check found, in a sentence: what the notification after it says. */
export function checkedText(
  report: CheckReport,
  project: string,
  added: number,
  unsaved: number,
): string {
  if (report.error !== null) return `Ascribe couldn't check ${project}: ${report.error}`;
  const builds = report.builds_checked.join(", ");
  const { errors, warnings, advice } = report.summary;
  const counts = [
    count(errors, "error"),
    count(warnings, "warning"),
    advice > 0 ? `${advice} advice` : "",
  ].filter(Boolean);
  let text =
    counts.length === 0
      ? `Ascribe checked every build of ${project} (${builds}): no problems.`
      : `Ascribe checked every build of ${project} (${builds}): ${counts.join(", ")}. ${
          added === 0
            ? "The Problems panel already showed them."
            : added === 1
              ? "1 wasn't in the Problems panel, and is now."
              : `${added} weren't in the Problems panel, and are now.`
        }`;
  if (unsaved > 0) {
    text += ` ${count(unsaved, "file")} with unsaved changes ${unsaved === 1 ? "was" : "were"} checked as saved.`;
  }
  return text;
}

function count(n: number, noun: string): string {
  return n === 0 ? "" : `${n} ${noun}${n === 1 ? "" : "s"}`;
}
