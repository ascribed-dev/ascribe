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
 * - `style-src`: the element library's stylesheet, review's marks, and the
 *   preview's, all files. The elements are styled by that stylesheet and custom properties
 *   and set no inline styles, so nothing needs `'unsafe-inline'`; a `style`
 *   attribute in a page's raw HTML is ignored.
 * - `img-src`, `font-src`: the same origin, so the project's images show and
 *   remote images don't.
 * - `frame-src`: the site preview's dev server, once it's shown, and nothing
 *   before. The frame is that server's own page, from its own origin, so the
 *   preview's policy doesn't apply inside it.
 * - `connect-src`, `form-action` and the rest fall to `default-src`, so the
 *   page can't reach the network.
 */
// Stricter than the site, which passes raw HTML through.
export function contentSecurityPolicy(cspSource: string, frameOrigin?: string): string {
  const directives = [
    "default-src 'none'",
    `script-src ${cspSource}`,
    `style-src ${cspSource}`,
    `img-src ${cspSource}`,
    `font-src ${cspSource}`,
  ];
  if (frameOrigin !== undefined) directives.push(`frame-src ${frameOrigin}`);
  return directives.join("; ");
}

/** The URLs the shell loads, as the webview reaches them. */
export interface ShellResources {
  cspSource: string;
  elementsScript: string;
  elementsStyle: string;
  /** Review's marks (`@ascribed/review/marks.css`). */
  marksStyle: string;
  previewScript: string;
  previewStyle: string;
  /** The site preview's origin, which the shell may frame (`http://localhost:4321`). */
  frameOrigin?: string;
}

const escapeAttribute = (text: string): string =>
  text.replace(/&/g, "&amp;").replace(/"/g, "&quot;").replace(/</g, "&lt;").replace(/>/g, "&gt;");

/** The webview's document. Everything it shows arrives later in messages. */
export function shellHtml(resources: ShellResources): string {
  const csp = escapeAttribute(contentSecurityPolicy(resources.cspSource, resources.frameOrigin));
  return `<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta http-equiv="Content-Security-Policy" content="${csp}" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>Ascribe preview</title>
    <link rel="stylesheet" href="${escapeAttribute(resources.elementsStyle)}" />
    <link rel="stylesheet" href="${escapeAttribute(resources.marksStyle)}" />
    <link rel="stylesheet" href="${escapeAttribute(resources.previewStyle)}" />
  </head>
  <body>
    <!-- The shell's elements are found by data-role, never by id: a page's
         headings have ids of their own (content, page) that a link's
         fragment must reach. -->
    <header class="chrome">
      <div class="toolbar">
        <label>Build <select data-role="build" disabled></select></label>
        <span data-role="build-description"></span>
        <div class="surface" role="group" aria-label="Preview" data-role="surface">
          <button type="button" data-surface="page" aria-pressed="true">Page</button>
          <button type="button" data-surface="site" aria-pressed="false">Site</button>
        </div>
      </div>
      <div class="review" data-role="review" hidden></div>
    </header>
    <ul class="problems" data-role="problems" aria-live="polite"></ul>
    <main data-role="page" hidden>
      <h1 data-role="title"></h1>
      <div data-role="availability"></div>
      <article data-role="content"></article>
    </main>
    <!-- The site preview: the dev server's page for the same file. -->
    <section class="site" data-role="site" hidden>
      <p class="site-problem" data-role="site-problem" hidden></p>
      <iframe
        data-role="site-frame"
        title="Site preview"
        sandbox="allow-scripts allow-same-origin allow-forms allow-popups allow-downloads"
        hidden
      ></iframe>
    </section>
    <script src="${escapeAttribute(resources.elementsScript)}"></script>
    <script src="${escapeAttribute(resources.previewScript)}"></script>
  </body>
</html>
`;
}
