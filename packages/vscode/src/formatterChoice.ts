// Which formatter VS Code runs on a project's pages. VS Code picks a document's
// formatter by `editor.defaultFormatter`, and a person's own choice for
// Markdown, such as Prettier, outranks a default an extension declares. A
// general Markdown formatter can change what a page builds to (`version:
// {version}` becomes `version: { version }`), so the extension offers to be
// the formatter for the folder (`ui/formatter.ts`).

/**
 * The extension `editor.defaultFormatter` names for a project's pages when
 * it's another extension's, or undefined when it's Ascribe's or unset.
 * Extension ids compare without case, as VS Code compares them.
 */
export function otherFormatter(configured: unknown, own: string): string | undefined {
  if (typeof configured !== "string") return undefined;
  const id = configured.trim();
  if (id === "" || id.toLowerCase() === own.toLowerCase()) return undefined;
  return id;
}

/** The offer to make Ascribe the formatter, naming the one that's set. */
export function formatterOffer(name: string): string {
  return `Ascribe: ${name} formats Markdown here, and can change what a page builds to. Make Ascribe the Markdown formatter for this folder?`;
}
