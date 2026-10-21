// The messages between the extension and Project Settings' webview, beside
// the ones every view has (`viewProtocol.ts`): its answers come the same
// way, numbered, after the webview says it's ready, and it says what it drew.
// What it adds is a change of value, a reset to the default, and a link to a
// setting's line, each naming a setting of the answer it drew, which the
// extension acts on only when that answer is still the newest. A check's
// level and its limit are settings too, with ids of their own. A part of the
// project with structure, a content type or a build, is a card whose link
// opens the file at its table; and a tab whose `ascribe.toml` is gone closes
// on the next click.
//
// Apart from VS Code and the DOM: the extension and the webview both read it.

/** A piece of a description: text, or a code span. */
interface SettingPieceView {
  text: string;
  code: boolean;
}

/** One setting as the tab draws it. */
export interface SettingView {
  /**
   * `table.key`: the same for the same setting from one answer to the next.
   * A check's level is `checks.<check>.level`, and its limit
   * `checks.<check>.limit`.
   */
  id: string;
  /** The heading it's drawn under: its table, `[consumer]`, or a check's area. */
  section: string;
  /** Its name, `Trailing slash`. */
  name: string;
  /** Its key as the file writes it, `[consumer] trailing-slash`. */
  key: string;
  /** `switch`, `choice`, `number`, `text`, `level`, or `size` (a number of bytes, or a string with a unit). */
  kind: string;
  choices: string[];
  /** Its value in the file, or its default when the file leaves it out; `null` when it has neither. */
  value: string | number | boolean | null;
  /** Its default, as text: `"always"`, or the reference's words. Empty when it has none. */
  defaultText: string;
  /** Where it's set, `ascribe.toml:12`, or that the file leaves it out. */
  where: string;
  /** Whether the file sets it: a bar marks it, and its place is a link. */
  set: boolean;
  /**
   * Whether the tab changes it: the file leaves it out, which the tab adds,
   * or sets it as a value the tab can show.
   */
  editable: boolean;
  /** Whether Reset to default is offered: the file sets it. */
  resettable: boolean;
  description: SettingPieceView[];
  /** The server's problems on its lines, in its words. */
  problems: { message: string; severity: string }[];
}

/** One configurable check as the tab draws it: its level, and its limit if it takes one. */
export interface CheckView {
  /** Its slug, `page-size`. */
  slug: string;
  /** Its code, `ASC149`. */
  code: string;
  /** Whether the file names it in `[checks]`: a bar marks it. */
  named: boolean;
  /** Its level, a setting whose description is what the check reports. */
  level: SettingView;
  limit: SettingView | null;
}

/** A part of the project with structure, as a card: the file is where it's edited. */
export interface PartView {
  /** `part:<table>.<name>`: the same for the same part from one answer to the next. */
  id: string;
  /** The heading it's drawn under: `Content types`. */
  section: string;
  /** Its name in its table: `guide`. */
  name: string;
  /** Its table as the file writes it: `[types.guide]`. */
  key: string;
  /** What the file sets in it: `Sets purpose, template`. */
  summary: string;
  /** Where the file names it, `ascribe.toml:12`. */
  where: string;
}

/** What Project Settings draws: a message in place of the settings, or the settings. */
export interface SettingsAnswer {
  /** The server's state, or why the file can't be read; `null` when it has settings to draw. */
  message: string | null;
  /** Whether `ascribe.toml` is gone, so the message's next click closes the tab. */
  gone: boolean;
  settings: SettingView[];
  /** Every configurable check, the ones the file names first. */
  checks: CheckView[];
  /** The parts with structure the file names, in the reference's order. */
  parts: PartView[];
}

/** What the webview drew, for tests. */
export interface SettingsReport {
  message: string | null;
  /**
   * Each setting as drawn, the checks' levels and limits after the settings:
   * its control's value as text, its place, and its problems.
   */
  settings: {
    id: string;
    value: string;
    where: string;
    problems: string[];
    editable: boolean;
    /** Whether it has a Reset to default. */
    resettable: boolean;
  }[];
  /** Each part's card, as drawn. */
  parts: { id: string; where: string }[];
  /** Content security policy violations seen so far. */
  violations: string[];
}

/** Webview to extension, besides `viewProtocol.ts`'s `FromView`. */
export type FromSettings =
  /** Set the setting `id` of the answer `seq` to `value`. */
  | { type: "set"; seq: number; id: string; value: string | number | boolean }
  /** Reset the setting `id` of the answer `seq` to its default: take it out of the file. */
  | { type: "reset"; seq: number; id: string }
  /** Open `ascribe.toml` at the line that sets `id`, a setting's or a part's, of the answer `seq`. */
  | { type: "openLine"; seq: number; id: string }
  /** Show the Content model view, where phrases, features, and terms are added and renamed. */
  | { type: "contentModel" }
  /** Close the tab, whose `ascribe.toml` is gone. */
  | { type: "close" }
  /** Open VS Code's Settings, filtered to Ascribe's: your editor's settings. */
  | { type: "editorSettings" };
