// The problems the language server reports, in `ascribe check --format
// json`'s shape, for the `ascribe_editor_problems` tool: what an agent in VS
// Code reads to see the problems of text that isn't saved, or that it wrote a
// moment ago. Apart from VS Code, so it can be tested by itself.

import * as path from "node:path";
import type { CheckReport, Entry, Fix, LspPosition, LspRange, Pos, Range } from "../shapes.js";
import type { ProtocolDiagnostic, Publication } from "./published.js";

/**
 * The version of `ascribe check`'s JSON this is written in: `SCHEMA_VERSION`
 * in `crates/ascribe-cli/src/report/json.rs`, which a test compares.
 */
export const CHECK_SCHEMA_VERSION = 1;

/**
 * The most diagnostics a report lists, as `ascribe check --format concise`
 * does: `LIMIT` in `crates/ascribe-cli/src/report/concise.rs`, which a test
 * compares.
 */
export const MAX_PROBLEMS = 50;

/** The report: `ascribe check`'s, and what the editor adds to it. */
export interface EditorProblemsReport extends CheckReport {
  /**
   * For a file open in an editor, the version of its document the
   * diagnostics are for; `null` for a project, or a file read from disk.
   */
  document_version: number | null;
  /**
   * Whether the server had checked the files as they are now. `false` when
   * it hadn't within about a second of the call, and some diagnostics may be
   * for an earlier version.
   */
  current: boolean;
  /** The files the report covers that have unsaved changes, as `file` names them. */
  unsaved: string[];
}

/** What the report is made from. */
export interface ProblemsInput {
  /** The project's folder, which paths are written relative to. */
  root: string;
  /** The publications the report covers. */
  publications: readonly Publication[];
  /** A file's text as the server checked it, to count columns and bytes in; `undefined` when unknown. */
  text: (file: string) => string | undefined;
  /** A related place's `file:` URI as a path; `undefined` for a URI that isn't a file's. */
  toPath: (uri: string) => string | undefined;
  /** The binary's version. */
  ascribeVersion: string;
  /** The build whose page-level checks the server runs. */
  editorBuild: string | undefined;
  /** How many source files the project has. */
  filesChecked: number;
  /** How many of them the report is about. */
  filesReported: number;
  documentVersion: number | null;
  current: boolean;
  /** The files with unsaved changes, as paths. */
  unsaved: readonly string[];
  /** The command that lists every diagnostic, when they're cut. */
  nextCommand: string;
}

/** The report of the publications' diagnostics. */
export function problemsReport(input: ProblemsInput): EditorProblemsReport {
  const positions = new Positions(input.text);
  const relative = (file: string) => relativePath(input.root, file);
  const all: { entry: Entry; order: [string, number] }[] = [];
  for (const publication of input.publications) {
    for (const diagnostic of publication.diagnostics) {
      const entry = toEntry(diagnostic, publication.file, relative, positions, input.toPath);
      all.push({ entry, order: [entry.file, entry.range.start.offset] });
    }
  }
  // Advice after errors and warnings, as `ascribe check` lists it.
  const advice = (e: Entry) => (e.severity === "advice" ? 1 : 0);
  all.sort(
    (a, b) =>
      advice(a.entry) - advice(b.entry) ||
      compare(a.order[0], b.order[0]) ||
      a.order[1] - b.order[1] ||
      compare(a.entry.code, b.entry.code),
  );
  const diagnostics = all.slice(0, MAX_PROBLEMS).map((d) => d.entry);
  const truncated = diagnostics.length < all.length;
  return {
    schema_version: CHECK_SCHEMA_VERSION,
    ascribe_version: input.ascribeVersion,
    error: null,
    files_checked: input.filesChecked,
    files_reported: input.filesReported,
    builds_checked: input.editorBuild === undefined ? [] : [input.editorBuild],
    diagnostics,
    truncated,
    shown: diagnostics.length,
    total: all.length,
    next_command: truncated ? input.nextCommand : null,
    summary: {
      errors: all.filter((d) => d.entry.severity === "error").length,
      warnings: all.filter((d) => d.entry.severity === "warning").length,
      advice: all.filter((d) => d.entry.severity === "advice").length,
    },
    document_version: input.documentVersion,
    current: input.current,
    unsaved: input.unsaved.map(relative).sort(compare),
  };
}

/** Ascribe's `data` on a published diagnostic. */
interface DiagnosticData {
  slug?: string;
  next?: string;
  builds?: string[];
  unpublished?: boolean;
  help?: string;
  fixes?: {
    title?: string;
    applicability?: string;
    edits?: { range: LspRange; newText?: string }[];
  }[];
}

function toEntry(
  d: ProtocolDiagnostic,
  file: string,
  relative: (file: string) => string,
  positions: Positions,
  toPath: (uri: string) => string | undefined,
): Entry {
  const data = (isRecord(d.data) ? d.data : {}) as DiagnosticData;
  const fixes: Fix[] = (data.fixes ?? []).map((fix) => ({
    title: fix.title ?? "",
    file: relative(file),
    edits: (fix.edits ?? []).map((edit) => ({
      range: positions.range(file, edit.range),
      new_text: edit.newText ?? "",
    })),
    applicability: fix.applicability === "safe" ? "safe" : "unsafe",
  }));
  return {
    code: String(d.code ?? ""),
    slug: data.slug ?? "",
    severity: severityName(d.severity),
    next: data.next ?? "write",
    message: d.message,
    file: relative(file),
    range: positions.range(file, d.range),
    related: (d.relatedInformation ?? []).flatMap((r) => {
      const at = toPath(r.location.uri);
      if (at === undefined) return [];
      return [
        { file: relative(at), range: positions.range(at, r.location.range), message: r.message },
      ];
    }),
    fixes,
    builds: data.builds ?? [],
    unpublished: data.unpublished ?? false,
    help: data.help ?? "",
    docs: d.codeDescription?.href ?? "",
    repeats: 0,
  };
}

/**
 * A protocol severity as `ascribe check` names it: the server publishes an
 * error as 1, a warning as 2, and advice as 3 (information). A diagnostic
 * with none is an error, as the protocol leaves it to the client.
 */
function severityName(severity: number | undefined): string {
  switch (severity) {
    case 2:
      return "warning";
    case 3:
    case 4:
      return "advice";
    default:
      return "error";
  }
}

/** A path relative to the project's folder, with `/`, as `ascribe check` writes it. */
export function relativePath(root: string, file: string): string {
  const windows = /^[a-zA-Z]:|^\\\\/.test(root);
  const relative = windows ? path.win32.relative(root, file) : path.posix.relative(root, file);
  return relative.split(windows ? "\\" : "/").join("/");
}

/**
 * Positions in `ascribe check`'s terms: the line and the column from 1, the
 * column in Unicode characters, and the byte offset in UTF-8, from the
 * protocol's lines from 0 and columns in UTF-16 units. Lines end at `\n`,
 * `\r\n`, or `\r`, as both count them.
 */
class Positions {
  private readonly indexes = new Map<string, LineStarts | undefined>();

  constructor(private readonly text: (file: string) => string | undefined) {}

  range(file: string, range: LspRange): Range {
    return { start: this.pos(file, range.start), end: this.pos(file, range.end) };
  }

  private pos(file: string, at: LspPosition): Pos {
    if (!this.indexes.has(file)) {
      const text = this.text(file);
      this.indexes.set(file, text === undefined ? undefined : lineStarts(text));
    }
    const index = this.indexes.get(file);
    // Without the text, the protocol's columns are the best there is.
    if (!index) return { line: at.line + 1, column: at.character + 1, offset: 0 };
    const line = Math.min(at.line, index.starts.length - 1);
    const start = index.starts[line] ?? 0;
    const end = index.ends[line] ?? index.text.length;
    const unit = Math.min(start + at.character, end);
    const before = index.text.slice(start, unit);
    return {
      line: line + 1,
      column: Array.from(before).length + 1,
      offset: (index.bytes[line] ?? 0) + Buffer.byteLength(before, "utf8"),
    };
  }
}

interface LineStarts {
  text: string;
  /** Each line's first UTF-16 unit. */
  starts: number[];
  /** Each line's end, before its line break. */
  ends: number[];
  /** Each line's first byte in UTF-8. */
  bytes: number[];
}

function lineStarts(text: string): LineStarts {
  const starts = [0];
  const ends: number[] = [];
  for (let i = 0; i < text.length; i++) {
    const c = text.charCodeAt(i);
    if (c !== 10 && c !== 13) continue;
    ends.push(i);
    if (c === 13 && text.charCodeAt(i + 1) === 10) i++;
    starts.push(i + 1);
  }
  ends.push(text.length);
  const bytes = [0];
  for (let line = 1; line < starts.length; line++) {
    const from = starts[line - 1] ?? 0;
    bytes.push((bytes[line - 1] ?? 0) + Buffer.byteLength(text.slice(from, starts[line]), "utf8"));
  }
  return { text, starts, ends, bytes };
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function compare(a: string, b: string): number {
  return a < b ? -1 : a > b ? 1 : 0;
}
