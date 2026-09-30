// The webview's HTML: a shell that loads the bundled element library and the
// preview script from the extension's own files, under a strict content
// security policy.

/**
 * The content security policy of the preview.
 *
 * `cspSource` is `webview.cspSource`: the origin VS Code serves the
 * extension's and the project's local resources from, and nothing else.
 *
 * - `default-src 'none'`: everything not listed is refused.
 * - `script-src`: the element library and the preview script, which are files
 *   of the extension. No `'unsafe-inline'`, no nonce, no `'unsafe-eval'`: an
 *   inline `<script>` or event-handler attribute in a page's raw HTML doesn't
 *   run.
 * - `style-src`: the element library's stylesheet and the preview's, both
 *   files. The elements are styled by that stylesheet and custom properties
 *   and set no inline styles, so nothing needs `'unsafe-inline'`; a `style`
 *   attribute in a page's raw HTML is ignored.
 * - `img-src`, `font-src`: the same origin, so the project's images show and
 *   remote images don't.
 * - `connect-src`, `frame-src`, `form-action` and the rest fall to
 *   `default-src`, so the page can't reach the network.
 */
// Stricter than the site, which passes raw HTML through.
export function contentSecurityPolicy(cspSource: string): string {
  return [
    "default-src 'none'",
    `script-src ${cspSource}`,
    `style-src ${cspSource}`,
    `img-src ${cspSource}`,
    `font-src ${cspSource}`,
  ].join("; ");
}

/** The URLs the shell loads, as the webview reaches them. */
export interface ShellResources {
  cspSource: string;
  elementsScript: string;
  elementsStyle: string;
  previewScript: string;
  previewStyle: string;
}

const escapeAttribute = (text: string): string =>
  text.replace(/&/g, "&amp;").replace(/"/g, "&quot;").replace(/</g, "&lt;").replace(/>/g, "&gt;");

/** The webview's document. Everything it shows arrives later in messages. */
export function shellHtml(resources: ShellResources): string {
  const csp = escapeAttribute(contentSecurityPolicy(resources.cspSource));
  return `<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta http-equiv="Content-Security-Policy" content="${csp}" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>Ascribe preview</title>
    <link rel="stylesheet" href="${escapeAttribute(resources.elementsStyle)}" />
    <link rel="stylesheet" href="${escapeAttribute(resources.previewStyle)}" />
  </head>
  <body>
    <!-- The shell's elements are found by data-role, never by id: a page's
         headings have ids of their own (content, page) that a link's
         fragment must reach. -->
    <div class="toolbar">
      <label>Build <select data-role="build" disabled></select></label>
      <span data-role="build-description"></span>
    </div>
    <ul class="problems" data-role="problems" aria-live="polite"></ul>
    <main data-role="page" hidden>
      <h1 data-role="title"></h1>
      <div data-role="availability"></div>
      <article data-role="content"></article>
    </main>
    <script src="${escapeAttribute(resources.elementsScript)}"></script>
    <script src="${escapeAttribute(resources.previewScript)}"></script>
  </body>
</html>
`;
}
