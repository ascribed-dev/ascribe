// What an edit says when it changes another project's `ascribe.toml`: a key
// `[from]` takes is edited in the file that holds it (SPEC §7.9), so an
// action or a setting in one project can change another, and says which
// file.

/** The model file's name, which `[from]` names the other file by. */
const MODEL_FILE = "ascribe.toml";

/**
 * The files of `changed`, each a path as the editor shows it, that are an
 * `ascribe.toml` other than `config`, the project's own, in order.
 */
export function otherModels(changed: readonly string[], config: string): string[] {
  const named = (path: string) => path === MODEL_FILE || /[\\/]ascribe\.toml$/.test(path);
  return changed.filter((path) => path !== config && named(path)).sort();
}

/**
 * The sentence that says an edit changed `files`, other projects'
 * `ascribe.toml`s; `undefined` when there are none.
 */
export function changedElsewhere(files: readonly string[]): string | undefined {
  if (files.length === 0) return undefined;
  const list = files.join(", ");
  return `Changed ${list}, which [from] takes ${files.length === 1 ? "it" : "them"} from: the edit is made where the key is written.`;
}
