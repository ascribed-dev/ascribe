// What a check of every build adds to the Problems panel: the problems
// `ascribe check` finds that the language server, which checks the editor's
// build only, doesn't show. No VS Code in this module.

import * as path from "node:path";
import { problemsText } from "./problems.js";
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

/** A problem's fix, as a quick fix makes it in the file it edits. */
export interface QuickFix {
  title: string;
  /** Whether it's marked safe: applying it leaves nothing to decide. */
  safe: boolean;
  edits: { range: EditorRange; newText: string }[];
}

/**
 * The fixes of `entry`, a problem of the project in `root`, that edit
 * `file`, an absolute path, whose text is `text`, as the check read it: a
 * check's problems in a file go when it changes.
 */
export function quickFixes(entry: Entry, root: string, file: string, text: string): QuickFix[] {
  return entry.fixes
    .filter((fix) => path.resolve(root, fix.file) === path.resolve(file))
    .map((fix) => ({
      title: fix.title,
      safe: fix.applicability === "safe",
      edits: fix.edits.map((edit) => ({
        range: editorRange(text, edit.range),
        newText: edit.new_text,
      })),
    }));
}

/**
 * The problems the Project view counts on their own, by their diagnostics'
 * names: tested examples changed since they were tested or never tested,
 * rechecks overdue, and a permalinks list that's behind.
 */
const COUNTED = {
  examples: ["example-stale", "example-unobserved"],
  rechecks: ["recheck-overdue"],
  permalinks: ["permalinks-list-behind"],
} as const satisfies Record<string, readonly string[]>;

export type Counted = keyof typeof COUNTED;

/** How many problems of one kind a check found, and where the first of them is. */
export interface Found {
  count: number;
  /** The first, in the report's order: its file, as an absolute path, and its place. */
  first: { file: string; range: EditorRange } | null;
}

/** What a report found of each kind the Project view counts, placed as `panelProblems` places them. */
export function countedIn(
  report: CheckReport,
  root: string,
  text: (file: string) => string | undefined,
): Record<Counted, Found> {
  const found = (slugs: readonly string[]): Found => {
    const entries = report.diagnostics.filter((entry) => slugs.includes(entry.slug));
    const first = entries[0];
    if (!first) return { count: 0, first: null };
    const file = path.resolve(root, first.file);
    return {
      count: entries.length,
      first: { file, range: editorRange(text(file) ?? "", first.range) },
    };
  };
  return {
    examples: found(COUNTED.examples),
    rechecks: found(COUNTED.rechecks),
    permalinks: found(COUNTED.permalinks),
  };
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
  const counts = problemsText(report.summary);
  let text =
    counts === ""
      ? `Ascribe checked every build of ${project} (${builds}): no problems.`
      : `Ascribe checked every build of ${project} (${builds}): ${counts}. ${
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

/**
 * A diagnostic as the language server writes it, in the protocol's shape:
 * what `ascribe/addresses` sends, each with the data a published diagnostic
 * carries.
 */
export interface ServerDiagnostic {
  range: EditorRange;
  /** 1 for an error, 2 a warning, 3 information, 4 a hint. */
  severity?: number;
  code?: string | number;
  codeDescription?: { href: string };
  message: string;
  relatedInformation?: { location: { uri: string; range: EditorRange }; message: string }[];
  data?: {
    slug?: string;
    fixes?: {
      title: string;
      applicability: string;
      edits: { range: EditorRange; newText: string }[];
    }[];
  };
}

/** The fixes the server gives a diagnostic of `ascribe/addresses`, which edit its own file. */
export function serverFixes(diagnostic: ServerDiagnostic): QuickFix[] {
  return (diagnostic.data?.fixes ?? []).map((fix) => ({
    title: fix.title,
    safe: fix.applicability === "safe",
    edits: fix.edits.map((edit) => ({ range: edit.range, newText: edit.newText })),
  }));
}
