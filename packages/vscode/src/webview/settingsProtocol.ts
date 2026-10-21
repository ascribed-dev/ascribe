// The messages between the extension and Project Settings' webview, beside
// the ones every view has (`viewProtocol.ts`): its answers come the same
// way, numbered, after the webview says it's ready, and it says what it drew.
// What it adds is a change of value and a link to a setting's line, each
// naming a setting of the answer it drew, which the extension acts on only
// when that answer is still the newest.
//
// Apart from VS Code and the DOM: the extension and the webview both read it.

/** A piece of a description: text, or a code span. */
export interface SettingPieceView {
  text: string;
  code: boolean;
}

/** One setting as the tab draws it. */
export interface SettingView {
  /** `table.key`: the same for the same setting from one answer to the next. */
  id: string;
  /** Its name, `Trailing slash`. */
  name: string;
  /** Its key as the file writes it, `[consumer] trailing-slash`. */
  key: string;
  /** `switch`, `choice`, `number`, `text`, or `level`. */
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
  /** Whether the tab changes it: the file sets it, as a value the tab can show. */
  editable: boolean;
  description: SettingPieceView[];
  /** The server's problems on its lines, in its words. */
  problems: { message: string; severity: string }[];
}

/** What Project Settings draws: a message in place of the settings, or the settings. */
export interface SettingsAnswer {
  /** The server's state, or why the file can't be read; `null` when it has settings to draw. */
  message: string | null;
  settings: SettingView[];
}

/** What the webview drew, for tests. */
export interface SettingsReport {
  message: string | null;
  /** Each setting as drawn: its control's value as text, its place, and its problems. */
  settings: {
    id: string;
    value: string;
    where: string;
    problems: string[];
    editable: boolean;
  }[];
  /** Content security policy violations seen so far. */
  violations: string[];
}

/** Webview to extension, besides `viewProtocol.ts`'s `FromView`. */
export type FromSettings =
  /** Set the setting `id` of the answer `seq` to `value`. */
  | { type: "set"; seq: number; id: string; value: string | number | boolean }
  /** Open `ascribe.toml` at the line that sets `id`, of the answer `seq`. */
  | { type: "openLine"; seq: number; id: string }
  /** Open VS Code's Settings, filtered to Ascribe's: your editor's settings. */
  | { type: "editorSettings" };
