// Which document links from a server the editor may follow. The server gives
// links only for web and email addresses and for files it resolved
// (`LINK_SCHEMES` in crates/ascribe-lsp/src/links.rs), but the extension runs
// the project's own binary, which may be older: an older server gave a link
// for any scheme, `command:` included, which runs an editor command.

/** The schemes the server links outside the docs; a test compares them with the server's. */
export const LINK_SCHEMES: readonly string[] = ["http", "https", "mailto"];

/** Whether the editor may follow a link whose target has this scheme: a web or email address, or a file. */
export function followableScheme(scheme: string): boolean {
  const lower = scheme.toLowerCase();
  return lower === "file" || LINK_SCHEMES.includes(lower);
}
