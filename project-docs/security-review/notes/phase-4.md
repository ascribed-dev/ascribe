# Phase 4 notes: the editor and the web packages

Hardening notes from [phase 4](../phase-4-editor-web.md), reviewed on 27 October 2026 by one agent and checked by a second, adversarial one. Method: `examples/astro-site` built with a page and review comments holding each payload the phase names, the HTML report built from it, the published site and the report opened in Chromium, the comment renderer driven with hostile bodies through the built `@ascribed/review` module, and the preview webview's HTML shell rendered in Chromium under its exact policy string, since VS Code itself can't run here. What a real VS Code webview does with a resource URI is inferred from its documented forms, not observed.

**One private finding exists, medium** (README decision 1), in the extension's page preview; the second reviewer promoted it from the first's hardening note. It's with the maintainer with its input. The notes below are the rest.

## H1: the HTML report's policy has no `form-action` and no `base-uri`, and `<form>` survives

- **Where:** `crates/ascribe-diff/src/html/mod.rs`, the report's policy (`default-src 'none'; img-src data:; style-src 'unsafe-inline'; script-src 'sha256-…'`); neither directive falls back to `default-src`. `disarm` (`packages/review/src/marks/index.ts`) keeps a page's `<form>`.
- **What it costs:** a reviewer who clicks a submit button in a page's rendered content in `ascribe diff --format html`'s file is sent to the author's URL with the form's fields. An offline file with no secrets and no other reach, so a click-gated navigation, not disclosure. Phase 1's H3 is the same policy's `'unsafe-inline'`.
- **Recommendation:** `form-action 'none'; base-uri 'none'` in the report's policy, as the preview already has, and `disarm` neutralising `<form>`.
- **Size:** small.

## H2: the preview webview forwards every link's `href` to the extension, and one allow-list decides

- **Where:** `packages/vscode/src/webview/preview.ts` posts `{type: "open", href}` for any link that isn't a fragment; `packages/vscode/src/preview/controller.ts`'s `openLink` opens only `http`, `https`, and `mailto` through `openExternal` and drops the rest. Confirmed in the harness: `command:`, `vscode:`, `file:`, and `data:` links are posted and dropped; `javascript:` is removed by `disarm` first.
- **What it costs:** nothing today, and it matches decision 75. The guarantee rests on the controller's list alone; an `open` handler that passed the `href` on without the check would reopen it.
- **Recommendation:** drop the schemes the extension never follows in the webview too, before posting, so neither side alone can widen it; keep the controller's list as the backstop.
- **Size:** small.

## H3: the comment renderer nests one anchor inside another

- **Where:** `packages/review/src/overlay/markdown.ts`, `renderInline`'s `[` branch, which recurses into a link's label while already inside an anchor. A body with a link inside a link's label renders `<a>` inside `<a>`, which the browser re-parents.
- **What it costs:** nothing for security: both anchors go through `safeUrl`, so only `http`, `https`, and `mailto` survive, with `rel="noopener noreferrer"`. Invalid HTML.
- **Recommendation:** render a nested link's label as text when already inside an anchor.
- **Size:** small.

## Checked, nothing found

- **Every webview's policy.** The sidebar views and Project Settings (`packages/vscode/src/webview/viewShell.ts`): `default-src 'none'`, a nonce on `script-src`, `style-src` and `font-src` the resource origin, `form-action 'none'`, `base-uri 'none'`. The preview (`packages/vscode/src/preview/html.ts`): the same with the resource origin on `script-src` and `img-src`, and `frame-src` the dev server's origin only once the Site switch is used. No `unsafe-inline` or `unsafe-eval` anywhere. Under the preview's policy in Chromium, a page's inline `<script>`, `onerror` attribute, `javascript:` link, `<object>`, `<embed>`, a framed page, and a `<meta refresh>` were each blocked or stripped. The one way through is the private finding.
- **`localResourceRoots`.** The preview: `dist/webview`, the content root, and each asset root the server names, never the workspace or the project root; the other views: `dist/webview` only.
- **Every message from a webview** (decision 78). `packages/vscode/src/preview/messages.ts` has one typed check per `FromWebview` variant, which a type error enforces; an unknown type or a bad shape is dropped and logged once. `git` is one of `pull`, `push`, `fetch`; `openFile`, `openPage`, `openThread`, and `openSource` go through `within`, which normalises `..` before comparing, and refuses a traversal, an absolute path outside, and a prefix trap (`/proj-evil/` against `/proj/`), checked with inputs of the second reviewer's own. The sidebar views and Project Settings check each of their own types the same way.
- **Workspace Trust.** Activation of everything that starts a process waits on `onDidGrantWorkspaceTrust`; the four task entries recheck `isTrusted`; every process start is `execFile` or `spawn` with a list of arguments (phase 2 has the one shell case). No `restrictedConfigurations` entry, and no workspace-settable setting runs anything in Restricted Mode, since activation is blocked wholesale; `ascribe.path` would be the one to list if that gate ever moved.
- **`command:` URIs in hovers and Markdown.** The one `MarkdownString` that carries a review comment sets `isTrusted` and `supportHtml` false and turns images into links; the others carry the extension's own text. `linkSchemes.ts` follows only `file`, `http`, `https`, and `mailto`.
- **Review comments from GitHub.** `renderMarkdown` builds DOM nodes only: raw HTML stays text, links go through `safeUrl`. Eighteen hostile bodies from the first reviewer and nine from the second, through the built module in Chromium: none ran, each rendered as escaped text or dropped; `javascript:`, `command:`, `vscode:`, and `data:` links became plain text; a code-span breakout stayed text. The HTML report renders no comment threads at all.
- **The GitHub transport** (decision 80). `ghTransport` runs `gh api graphql --hostname <host>` with the query on standard input and never sees a token; `tokenTransport` sends with `redirect: "error"` and, given a `HostToken`, refuses another host before sending. The extension's `providerFor` maps only `github.com` and the `github-enterprise.uri` host to a sign-in, any other host to none. No token is stored by the package or printed in an error.
- **`set:html` and `innerHTML`.** `Head.astro`'s JSON-LD escapes `<`; the starter's title and label use the site output's `formatted` field, whose emitter escapes `< > & "` and emits only `<code>`, so a title holding markup built to escaped text with no script or image; the preview and the report assign into an inert `<template>` and then `disarm` it under the policy.
- **`<ascribe-tabs>` and the address.** `Permalink.astro` calls `choose` only for a declared dimension with one of its declared values; `tabs.ts` matches a value against each tab's own and toggles `aria-selected`, never writing the address into the DOM or `innerHTML`. `?pm=pnpm` selected pnpm; an injected value was ignored with nothing written to the DOM or storage; a repeated `pm` takes the first.
- **Help addresses.** `crates/ascribe-emit/src/site/headers.rs` gives `Access-Control-Allow-Origin: *` to `go/<name>.json` and `go/<name>.md` only; each Markdown page version gets a canonical `Link` and nothing else, confirmed in the built `_ascribe/headers.json`. `@ascribed/astro`'s help route serves that text and nothing else under the header.
