// A sidebar webview view's HTML: a shell that loads its script and stylesheet
// from the extension's own files, under a strict content security policy.
// Everything it shows arrives later in messages (`viewProtocol.ts`), and is
// set as text.

/**
 * The content security policy of a webview view.
 *
 * - `default-src 'none'`: everything not listed is refused, images, frames,
 *   and the network among them.
 * - `script-src`: the view's script, by its nonce, and nothing else: no
 *   inline script, no event-handler attribute, no `eval`.
 * - `style-src`, `font-src`: the extension's own files (`cspSource`), the
 *   stylesheet and the codicon font it loads. No inline style.
 * - `form-action 'none'`, `base-uri 'none'`: neither falls to `default-src`.
 */
function viewPolicy(cspSource: string, nonce: string): string {
  return [
    "default-src 'none'",
    `script-src 'nonce-${nonce}'`,
    `style-src ${cspSource}`,
    `font-src ${cspSource}`,
    "form-action 'none'",
    "base-uri 'none'",
  ].join("; ");
}

/** What the shell loads, as the webview reaches it. */
export interface ViewResources {
  cspSource: string;
  /** A new one for each document, which the script's tag carries. */
  nonce: string;
  script: string;
  style: string;
  /** The view's name, for the document's title. */
  title: string;
}

const escapeAttribute = (text: string): string =>
  text.replace(/&/g, "&amp;").replace(/"/g, "&quot;").replace(/</g, "&lt;").replace(/>/g, "&gt;");

/** The view's document: an empty `<main>` the script draws into. */
export function viewHtml(resources: ViewResources): string {
  const policy = escapeAttribute(viewPolicy(resources.cspSource, resources.nonce));
  return `<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta http-equiv="Content-Security-Policy" content="${policy}" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>${escapeAttribute(resources.title)}</title>
    <link rel="stylesheet" href="${escapeAttribute(resources.style)}" />
  </head>
  <body>
    <main data-role="view"></main>
    <script nonce="${escapeAttribute(resources.nonce)}" src="${escapeAttribute(resources.script)}"></script>
  </body>
</html>
`;
}
