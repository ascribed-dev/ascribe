# Phase 4: The editor and the web packages

Part of [Security review](README.md). Needs phase 0.

## Goal

Find where content, a review comment, or a message from a webview can run script, or reach something the user didn't ask for, in the editor or on a published site.

## Questions

### The extension

- Webviews (`packages/vscode/src/webview/`, `ui/webviewView.ts`, `ui/projectSettings.ts`): is the content security policy on every one, with no `unsafe-inline` or `unsafe-eval`? Is `localResourceRoots` as narrow as it can be? Does every message from a webview get checked before it acts (the This page view drops a key it didn't draw; check the others)?
- The preview renders a page's HTML, raw HTML included, in a webview: what can a page's script do there, given the policy?
- Workspace Trust: every process the extension starts (`processes.ts`, `diskChanges.ts`, `gitIgnored.ts`, `ui/checkEveryBuild.ts`, tasks), and every setting a workspace can set that changes what runs.
- Commands a link or a webview can trigger (`command:` URIs in hovers and Markdown).

### `@ascribed/review`

- Review comments come from GitHub, written by anyone who can comment: are their bodies rendered as text or as sanitized Markdown, never as raw HTML, in the site's overlay and in `ascribe diff`'s HTML report?
- Its GitHub transport (`packages/review/src/github/transport.ts`): tokens, where they're stored, and what they're sent to.

### `@ascribed/astro` and the elements

- The four places that set HTML directly (`set:html` or `innerHTML`) in `packages/astro/src`, `packages/elements/src`, and the webviews: what reaches each, and from whom.
- `<ascribe-tabs>`, the one element that runs script (`packages/elements/CONTRACT.md` §0): what it reads from the page and the address (`?pm=pnpm`), and whether an address can inject into the page.
- Help addresses (`go/<name>.json`) served with `Access-Control-Allow-Origin: *`: only public text, as decided; verify nothing else gets that header.

## Method

Build `examples/astro-site` with a page and a review comment holding each candidate payload (a `<script>`, an `onerror` attribute, a `javascript:` link, a `command:` link), open it in a browser and in the editor's preview, and record what runs.

## Output

Draft advisories for exploitable findings; `notes/phase-4.md` for hardening.
