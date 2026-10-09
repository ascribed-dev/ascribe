# Changelog

Every Ascribe release: the `ascribe` binary, the npm packages (`@ascribed/cli`, `@ascribed/astro`, `@ascribed/elements`, `@ascribed/review`), and the VS Code extension share one version. Versions follow [semantic versioning](https://semver.org/); while the major version is 0, a minor version may change behavior.

## Unreleased

- **Commands that answer questions.** Six new commands tell an author, or an agent, what it would otherwise guess, and change nothing. `ascribe explain ASC036` says what a diagnostic means and how to fix it, with a checked example for the diagnostics people meet most; `ascribe model` shows what the content model allows, in under 4,000 characters; `ascribe outline <page>` lists a page's headings with the ids links use, including those from its fragments; `ascribe link <target> --from <page>` says whether a link works and what to write; `ascribe refs <target>` lists where a page, heading, fragment, phrase, feature, or other content model entry is used, as **Find All References** does in the editor; and `ascribe render <page> --build <name>` shows a page as that build's readers see it. Each takes `--format json`, described by a JSON Schema. `ascribe --help` starts with examples. See the [command reference](docs/content/reference/cli.md#ascribe-explain).
- **Check one file, or text that isn't saved.** `ascribe check docs/guides/install.md` reports only what counts for that file: its own problems, and those its fragments cause on it; checking a fragment shows each of its problems once. `--stdin --path <file>` checks standard input as that file's text without writing it. `--editor-build` runs the editor's build's checks alone, and with a file named checks only it and the pages that include it, quick enough to run after every edit. `--format concise` writes one line per diagnostic, for an agent, and `--summary` counts the diagnostics by code and by file. In the JSON, each diagnostic has its fix advice (`help`) and a link to its entry in the diagnostics reference (`docs`), and each fix says whether it's safe to apply as it is (`applicability`); `schema_version` stays `1`. In the editor, a diagnostic's code links to its entry. See [Checking some files](docs/content/reference/cli.md#checking-some-files).
- **Problems for files changed on disk, in VS Code.** A project's language server now also starts when one of its files changes on disk while the window is open, not only when one is opened. An agent or a script that writes pages without opening them gets their problems in the Problems panel. A `git checkout` starts each touched project's server once, and opening the window still starts nothing. The Problems panel checks pages as the editor's build only; `ascribe check` checks every build. See [When servers start](docs/content/guides/editor.md#when-servers-start).
- **Actions in VS Code, and the actions bar.** Actions write Ascribe for you, such as **Wrap in a note**, **Make the list steps**, **Link the selected text**, **Insert content that varies**, and **Mark where it's available**. Each asks what it needs in short steps, choosing from the project's note types, pages, phrases, dimensions, and features, and writes canonical Ascribe that one Undo takes back. `Ctrl+K A` (`Cmd+K A` on macOS), or **Ascribe: Actions for the Cursor**, opens the actions bar: the fixes for problems at the cursor first, then the actions that apply to the cursor or the selection, grouped as **Write**, **Structure**, **Link**, **Media**, and **Content model**. Every action is also in the Command Palette, in an **Ascribe** submenu of the editor's context menu, and, for those that rewrite what's at the cursor, in the lightbulb. The **Content model** actions make the selected text a phrase (and, if you choose, its other occurrences), add it to the glossary, change a feature's availability, and rename a phrase or a dimension value everywhere, editing `ascribe.toml` in place with its comments and layout; a rename that changes other files shows them in the refactor preview first. See [Actions](docs/content/guides/editor.md#actions).
- **The Ascribe sidebar, the status bar, and the build lens in VS Code.** The Ascribe sidebar's **Projects** view lists every project in the workspace, started or not, with buttons to show a server's output or restart it; **Used by** lists what links to or includes the active page, or the heading at the cursor; **Pages** lists the project's pages by type, its fragments with what includes them, and pages nothing links to; **Content model** lists every phrase, feature, glossary term, dimension, note type, widget, and build with how many places use it, marks the unused ones, and opens each one's declaration. Filling a view never starts a server. While a page or an `ascribe.toml` is open, the status bar names its project, the build you're looking at, and whether its language server is running; click it to switch the build, show the server's output, restart the server, or open the preview. Each project has one build you're looking at: the preview renders it, and **Ascribe: Dim What the Build Leaves Out** turns on the build lens, which dims the variant arms it doesn't select and the sections, blocks, and table rows its availability filter removes, with a hover that says why. Switching the build in the status bar, with **Ascribe: Switch Build**, or in the preview's **Build** picker changes all three. The language server answers a new request, `ascribe/buildView`, from the same decisions `ascribe build` makes. See [The Ascribe sidebar](docs/content/guides/editor.md#the-ascribe-sidebar), [The status bar](docs/content/guides/editor.md#the-status-bar), and [The build lens](docs/content/guides/editor.md#the-build-lens).
- **Find All References, and more renames.** **Find All References** lists the links to a page or heading, including links through a page that includes it, the includes of a fragment, and the uses of a phrase, feature, glossary term, note type, widget, or dimension, in VS Code and, through the language server, in other editors. `F2` renames a dimension value too, a phrase rename reaches frontmatter fields and `@snippet {phrases=true}` files, and `F2` on something that can't be renamed says why. See [Navigation](docs/content/guides/editor.md#navigation) and [Refactoring](docs/content/guides/editor.md#refactoring).
- **A getting-started walkthrough in VS Code.** After you install the extension, **Get Started with Ascribe** goes through opening a project, previewing a page, the actions bar, the Ascribe sidebar, the build lens, and checking the project in CI. It's under **Help → Welcome**. See [The walkthrough](docs/content/guides/editor.md#the-walkthrough).
- **A faster `ascribe diff` and `ascribe drift`.** `diff` compares only the pages a change can reach, and reads the two versions side by side: on 3,000 pages it takes a third less time than it did, and a third less memory. `drift` takes a third less time when an example changed. The reports are the same.
- **JSON Schemas for the commands' JSON.** What `ascribe check`, `build`, `diff`, `drift`, `sources status`, `sources update`, and the commands that answer questions write with `--format json` is described by a JSON Schema, generated from the code that writes it. See the [JSON report contract](docs/content/contracts/json-reports.md).
- **The `json` output's page `format` is `"ascribe-page"`,** where it was `"tessera-page"`, the project's working name. Its `schemaVersion` stays `1`. A tool that checks the value needs the new one. See [the JSON output](crates/ascribe-emit/README.md#json).
- The language server's log lines, in the editor's output panel, start with `ascribe-lsp:`, where they started with `tessera-lsp:`.
- **A snippet whose file isn't at the pin says so.** When a file a snippet names isn't in the code's repository at the commit `ascribe.lock` pins, because it was moved or deleted there, `ascribe check` says that, and how to fix it, where it said to run `ascribe sources fetch`, which can't help. The update pull request quotes the new message. `ascribe sources fetch` and `update` record those files in `ascribe.lock`, under a new key, `missing`, and `ascribe sources status` shows them as `not_at_pin`. The lock's `version` stays `1`; a lock with `missing` can't be read by an earlier release. See [`ascribe.lock` and the copies](docs/content/contracts/content-model.md#191-ascribelock-and-the-copies).
- **Fixed:** `ascribe fmt` no longer follows a symbolic link out of the content root. A file that's a link, or is in a linked folder, that leads to a file that isn't a source file of the content root is left alone and reported as `ascribe check` reports it (`source-unreadable`), and `fmt` exits with 2. It was formatted and written where the link led. See [`ascribe fmt`](docs/content/reference/cli.md#ascribe-fmt).
- **A title with code shows it in review.** The page preview's heading in VS Code, the HTML report's list of pages and its page headings, and the list of comments in the editor and the site preview show a title's code spans as code, where they showed plain text. **Ascribe: Changed Pages** writes them between backticks. The HTML report's data, and the language server's `ascribe/preview` and `ascribe/review/changes` answers, have the formatted title beside the plain one. See [code in a field](docs/content/reference/content-model.md#53-code-in-a-field).
- **New default colors for the elements and review.** The element library, review's marks and comment threads, the review panel in Astro's dev toolbar, and the HTML report take Ascribe's palette: an indigo accent over cool grays, with notes, availability badges, and review's marks in matching hues. Added and removed differ in more than red and green, for readers with red-green color blindness. The report's frame uses Ascribe's type scale. **Behavior change:** a site that doesn't set the `--ascribe-*` and `--ascribe-review-*` custom properties looks different. Their names are unchanged, and their defaults may change again in a release, always noted here. To keep the previous look, add this to the site's stylesheet, after the element library's and review's:

  <details>
  <summary>The previous default colors</summary>

  ```css
  /* The element library */
  @supports (color: light-dark(#000, #fff)) {
    :root {
      --ascribe-border-color: light-dark(#d0d7de, #3d444d);
      --ascribe-muted-color: light-dark(#57606a, #9198a1);
      --ascribe-note-color: light-dark(#0969da, #4493f8);
      --ascribe-note-background: light-dark(#ddf4ff, #121d2f);
      --ascribe-tip-color: light-dark(#1a7f37, #3fb950);
      --ascribe-tip-background: light-dark(#dafbe1, #12261e);
      --ascribe-important-color: light-dark(#8250df, #ab7df8);
      --ascribe-important-background: light-dark(#fbefff, #1f1a33);
      --ascribe-warning-color: light-dark(#9a6700, #d29922);
      --ascribe-warning-background: light-dark(#fff8c5, #272115);
      --ascribe-caution-color: light-dark(#cf222e, #f85149);
      --ascribe-caution-background: light-dark(#ffebe9, #2d1517);
      --ascribe-steps-color: light-dark(#0969da, #4493f8);
      --ascribe-steps-marker-text-color: light-dark(#ffffff, #0d1117);
      --ascribe-tab-color: light-dark(#57606a, #9198a1);
      --ascribe-tab-active-color: light-dark(#0969da, #4493f8);
      --ascribe-tab-focus-color: light-dark(#0969da, #4493f8);
      --ascribe-tab-hover-background: light-dark(#f6f8fa, #151b23);
      --ascribe-state-color: light-dark(#57606a, #9198a1);
      --ascribe-state-background: light-dark(#eaeef2, #212830);
      --ascribe-state-ga-color: light-dark(#1a7f37, #3fb950);
      --ascribe-state-ga-background: light-dark(#dafbe1, #12261e);
      --ascribe-state-preview-color: light-dark(#8250df, #ab7df8);
      --ascribe-state-preview-background: light-dark(#fbefff, #1f1a33);
      --ascribe-state-beta-color: light-dark(#0969da, #4493f8);
      --ascribe-state-beta-background: light-dark(#ddf4ff, #121d2f);
      --ascribe-state-deprecated-color: light-dark(#9a6700, #d29922);
      --ascribe-state-deprecated-background: light-dark(#fff8c5, #272115);
      --ascribe-state-removed-color: light-dark(#cf222e, #f85149);
      --ascribe-state-removed-background: light-dark(#ffebe9, #2d1517);
    }
  }

  /* Review's marks and overlay */
  :root {
    --ascribe-review-added: #1a7f37;
    --ascribe-review-added-background: #dcf7e3;
    --ascribe-review-changed: #8a5a00;
    --ascribe-review-changed-background: #fff1c2;
    --ascribe-review-removed: #c4222d;
    --ascribe-review-removed-background: #ffe4e2;
    --ascribe-review-moved: #7a3fd0;
    --ascribe-review-moved-background: #f0e6ff;
    --ascribe-review-muted: #5b6475;
    --ascribe-review-surface: #ffffff;
    --ascribe-review-flash: #fff6bf;
    --ascribe-review-focus: #1f6feb;
    --ascribe-review-text: #1d2330;
    --ascribe-review-border: #d3d8e0;
    --ascribe-review-thread: #f5f7fb;
    --ascribe-review-input: #ffffff;
    --ascribe-review-accent-text: #ffffff;
  }

  @media (prefers-color-scheme: dark) {
    :root:not([data-ascribe-scheme="light"]) {
      --ascribe-review-added: #4fc572;
      --ascribe-review-added-background: #133520;
      --ascribe-review-changed: #dcb13a;
      --ascribe-review-changed-background: #3a2f0d;
      --ascribe-review-removed: #ff8279;
      --ascribe-review-removed-background: #401918;
      --ascribe-review-moved: #c59bff;
      --ascribe-review-moved-background: #2d2440;
      --ascribe-review-muted: #929bab;
      --ascribe-review-surface: #1e2026;
      --ascribe-review-flash: #3b3514;
      --ascribe-review-focus: #5aa2ff;
      --ascribe-review-text: #d9dde5;
      --ascribe-review-border: #343a45;
      --ascribe-review-thread: #272b33;
      --ascribe-review-input: #1e2026;
      --ascribe-review-accent-text: #0b1220;
    }
  }

  :root[data-ascribe-scheme="dark"] {
    --ascribe-review-added: #4fc572;
    --ascribe-review-added-background: #133520;
    --ascribe-review-changed: #dcb13a;
    --ascribe-review-changed-background: #3a2f0d;
    --ascribe-review-removed: #ff8279;
    --ascribe-review-removed-background: #401918;
    --ascribe-review-moved: #c59bff;
    --ascribe-review-moved-background: #2d2440;
    --ascribe-review-muted: #929bab;
    --ascribe-review-surface: #1e2026;
    --ascribe-review-flash: #3b3514;
    --ascribe-review-focus: #5aa2ff;
    --ascribe-review-text: #d9dde5;
    --ascribe-review-border: #343a45;
    --ascribe-review-thread: #272b33;
    --ascribe-review-input: #1e2026;
    --ascribe-review-accent-text: #0b1220;
  }
  ```

  </details>

  See [theming](packages/elements/README.md#theming).
- **Smaller downloads.** The `ascribe` binary is built with link-time optimization: about a third smaller on macOS and Linux (7.0 MB on macOS arm64, from 10.5 MB) and 8 percent smaller on Windows, and a little faster.
- `@ascribed/astro` declares `satteri` and `@types/hast` as optional peer dependencies. Its `@ascribed/astro/satteri` and `@ascribed/astro/rehype` declarations import types from them, so a project that type-checks those declarations (`skipLibCheck: false`) needs the one it imports, which a Sätteri or `unified()` processor already brings.
- **The VS Code extension has an icon:** the Ascribe mark, in the Extensions view and on its Marketplace page, whose header takes the icon's dark tile color.
- **Fixed:** in a high-contrast dark theme, the page preview's **Page | Site** switch shows which side is on, with the theme's active border, and the preview's buttons are in the theme's text color. They took a button background the theme doesn't set.

## 0.2.0 (2026-10-06)

This release adds **review**: reading a pull request as readers will see it, page by page, with the changed blocks marked and the pull request's review comments beside them. It works in VS Code's page preview, in your site's own pages under `astro dev`, and as an HTML report CI can attach to every pull request. Comments are ordinary GitHub review comments, so there's nothing to set up or host. See [Review](docs/content/guides/review.md).

### The language

- **`@snippet`** puts a code example from a file in the page as a fenced code block: `@snippet: code:examples/quill/ascribe.toml#dimensions`. The file is read on every check, build, and diff, so the example changes when the code does. A region is marked in the code with Bluehawk's tags (`:snippet-start:`, `:snippet-end:`, `:remove:`). `ascribe check` reports a missing address, file, or region, and tags that don't balance (`snippet-address` and five more). See [`@snippet`](docs/content/reference/directives.md#snippet) and [Drift](docs/content/guides/drift.md).
- **`[sources.<name>]`** in `ascribe.toml` names a folder of code outside the content that snippets may read, with `include` and `ignore` patterns. It's the only way a page reads outside its project's folder, and the folder must be in the project's git repository. See [`[sources.<name>]`](docs/content/reference/content-model.md#18-sourcesname).
- **A source in another repository:** `git` (and `branch`) in place of `path`. The files snippets use are copied into `sources/<name>/` and pinned to a commit in `ascribe.lock`. Both are committed with the docs, so checking, building, diffing, and drift never reach the other repository. `ascribe check` reports copies that don't match the lock or the snippets (`lock-invalid`, `source-copy-changed`, and three more). See [a source in another repository](docs/content/reference/content-model.md#a-source-in-another-repository) and [Docs kept apart from the code](docs/content/guides/drift.md#docs-kept-apart-from-the-code).
- **A table row can have an availability:** an attribute block at the end of its first cell, `` | `stream` {available="self-managed preview 3.4"} | … | ``. A badge build marks the row, and a filter build removes it where it isn't available. The spec is checked as an `@available` line's is, within the table's own availability. **Behavior change:** a `{key=value}` that ends a body row's first cell was text and is now an attribute block, so a key other than `available` is an error (`attribute-unknown-key`). See [table rows](docs/content/reference/directives.md#table-rows).
- **A page title can show code.** `inline = "code"` on a content type's `string` field reads code spans in it: `title = { type = "string", inline = "code" }` lets a page be titled "`` `ascribe.toml` reference ``". The site output writes the field as plain text, for `<title>` and search, and as HTML under the new key `formatted` (`entry.data.formatted.title` in an Astro layout). The JSON output has `formatted` too. **Behavior changes:** a type can no longer declare a field named `formatted` (`model-field-reserved`), and in a field that turns `inline` on, a backslash before punctuation is an escape and is removed, as in Markdown. See [code in a field](docs/content/reference/content-model.md#53-code-in-a-field).
- **A link can name a heading from a fragment the page includes:** `setup.md#prerequisites` works when `prerequisites` comes from a fragment `setup.md` includes. It was an error (`link-id-in-fragment`, now retired). See [`@include`](docs/content/reference/directives.md#include).

### The `ascribe` command

- `ascribe diff` shows what changed between a git revision and the working tree, as readers will see it: each build's changed pages, and the blocks added, removed, changed, or moved on them, with the words that changed. It compares resolved pages, so a page that changed only through a fragment, a phrase, or a build's settings is listed with the cause, and reformatting is no change. The default base is the merge base with the default branch, as in a pull request. `--format json` adds each block's source lines, for tools. It needs only `git`. It also says how many errors the working tree has, as do the HTML report and both previews, so a broken render isn't read as the change. See [`ascribe diff`](docs/content/reference/cli.md#ascribe-diff).
- `ascribe diff --format html` writes the changes as one self-contained HTML file: every changed page rendered with its changes marked, and the page as it will be and as it was a click away. It makes no network requests and runs nothing from the pages, so CI can attach it to a pull request. See [the HTML report](docs/content/reference/cli.md#the-html-report) and [the report in CI](docs/content/guides/review.md#the-report-in-ci).
- `ascribe drift` lists the pages whose code examples changed between a git revision and the working tree: first those whose own text didn't change, so the words around the code may be out of date, then those that changed with their examples, then those whose examples no longer resolve. A region is compared as text, so an edit elsewhere in its file isn't a change. `--format summary` writes Markdown for a CI job's summary, `--format json` is for tools, and `--exit-code` exits 1 when an example broke, or changed while its page didn't. See [`ascribe drift`](docs/content/reference/cli.md#ascribe-drift) and [Drift](docs/content/guides/drift.md#when-an-example-changes).
- `ascribe build --emit site --anchors` marks each block of the site output with its source file and lines (`data-ascribe-source`, and `data-ascribe-via` for a block from a fragment). Without the flag, the output is unchanged. See the [site-render contract](docs/content/contracts/site-render.md#7-source-anchors).
- `ascribe sources fetch` makes the copies of sources in other repositories match `ascribe.lock`. `ascribe sources update` moves a source's pin to the head of its branch (or `--to`), copies again, and reports the commits, the copies, and the pages whose examples changed, as text, JSON, or Markdown for a pull request. `ascribe sources status` shows each pin and copy, offline. `fetch` and `update` run `git` with its own credentials, and are the only commands that reach the network. See [`ascribe sources`](docs/content/reference/cli.md#ascribe-sources).
- Each command's `--help` describes its options in the words of the [command reference](docs/content/reference/cli.md), whose option lists are generated from it, and ends with a link to its section on the docs site, <https://ascribed-dev.com>. `-h` shows the first paragraph of each.
- **Fixed:** the plain-markdown output (`--emit plain`) writes an inline `<br>` as a line break, or as `; ` in a table cell or heading. It was dropped, so code spans on either side ran together.

### The editor

- **Review in the preview.** **Ascribe: Start Review** compares the active page's project with a base: the base of the branch's pull request, the default branch, or a revision you type. The preview marks what changed on the page and follows your edits. Its header shows the base and the page's changes, switches between **Changes / As it will be / As it was**, and steps through the changes and the changed pages. **Ascribe: Changed Pages** lists the pages the change touches, and a status bar item shows whether review is on and starts it. See [Review in the preview](docs/content/guides/editor.md#review-in-the-preview).
- **Comments in the preview.** When the branch has an open pull request, the preview shows its review threads beside their blocks. You can reply, resolve, comment on any block, and submit or discard your review. New comments are visible only to you until you submit. **Ascribe: Refresh Comments** reads the threads again. Threads also show on their source lines, unless the GitHub Pull Requests extension shows them already (the new setting `ascribe.review.sourceComments`). Comments use VS Code's GitHub sign-in or the GitHub CLI's. Without either, review shows the changes only. See [Comments in the preview](docs/content/guides/editor.md#comments-in-the-preview).
- **The site preview.** **Ascribe: Open Site Preview** opens the active page on its project's dev server, in the browser, at the heading the editor shows. The preview panel's **Page | Site** switch shows it in the panel, following the active file. See [Site preview](docs/content/guides/editor.md#site-preview).
- **Ascribe: Open Preview to the Side** is now **Ascribe: Open Page Preview to the Side** (the command id is unchanged). The new **Ascribe: Open Page Preview** opens the preview in place of the editor.
- The preview and the editor scroll together by block instead of by heading, in both directions, and double-clicking a block puts the cursor on its source line. `ascribe.preview.scrollPreviewWithEditor` and `ascribe.preview.scrollEditorWithPreview` turn each direction off.
- The preview takes scripts, event handlers, and `javascript:` URLs out of a page's HTML before showing it, as the review report does.
- The Settings editor describes each setting in the words of the [editor guide](docs/content/guides/editor.md#settings), whose settings table is generated from it.

### Astro

- **Review in the site preview.** In `astro dev`, an **Ascribe review** app in Astro's dev toolbar marks a change's blocks on the real page, in your site's layout, and shows the pull request's review threads beside them. Commenting goes through the GitHub CLI, which the dev server runs. It stays off when the dev server listens on the network, or when the Vite config lets other pages reach it. The `review` option (on by default) turns the app off. `astro build` output has no anchors, overlay, or review code. See [Review in the site preview](docs/content/guides/astro.md#review-in-the-site-preview).
- `astro dev` writes its address to `.ascribe/dev.json` in the project, for the editor's site preview, and removes it when it stops.
- The `anchors` option turns source anchors on: `"dev"` in `astro dev` only, `true` always.
- A code block's title, such as a `@snippet`'s, shows above it, in a `<figure class="code-title">` with a `<figcaption>`. The `codeTitles` option turns it off. See [code block titles](docs/content/guides/astro.md#code-block-titles).
- `@ascribed/astro/Availability.astro` renders a page's `available` frontmatter as the element library's badge: `<Availability available={entry.data.available} />`.
- The integration copies the generated schema into `.astro/integrations/_ascribed_astro/schema.ts` after each build. A site whose Ascribe project is outside the Astro root imports that copy, so type-checking finds `astro/zod`. See [define the collection](docs/content/guides/astro.md#4-define-the-collection).
- A clean build's check summary (`checked 14 files: 0 errors, 0 warnings`) is logged as info, not as a warning.
- The npm packages' source maps include their sources, so Vite no longer warns that they point to missing files.

### The language server

- It answers `ascribe/review/setBase` and `ascribe/review/changes`, and `ascribe/preview` with `review: true` adds the page's changes, for editors other than VS Code. See [`crates/ascribe-lsp/README.md`](crates/ascribe-lsp/README.md).

### Installing

- The Linux binaries run on glibc 2.28 or later, instead of 2.39, so `@ascribed/cli` installs and runs in the build images of Vercel, AWS Amplify, and Cloudflare Pages as well as Netlify's.
- A canary of the npm packages, built from `main`, is published every night under the `next` tag: `npm install @ascribed/cli@next @ascribed/astro@next`. It comes with no promise of stability, and its `ascribe --version` names the commit it was built from. Releases still publish under `latest`.

### `@ascribed/review`

- A new package, for hosts that show review. Its Node part finds the open pull request for a checkout's branch, reads its review threads, places each on its rendered block, and posts comments and replies into the reviewer's pending review. It talks to GitHub through the GitHub CLI or a token its host supplies, and stores no token. Its browser part marks a rendered page's changed blocks and draws the threads beside them. Comment bodies render from a safe subset of Markdown, with no raw HTML, and no image loads without a click. See [`packages/review`](packages/review/README.md).

## 0.1.1 (2026-10-02)

### The language

- **Behavior change:** a directory under the content root that holds an `ascribe.toml`, other than the project's own folder, is another project's folder, and is skipped like a directory whose name starts with `.`. `ascribe check`, `ascribe build`, `ascribe fmt`, and the language server leave its files to that project, so an editor shows each file's diagnostics from one project only. A link or an include from the outer project to a Markdown file in it is reported as a missing file.

### The editor

- A workspace can hold several Ascribe projects. Every `ascribe.toml` is a project with its own language server, and a file belongs to the nearest `ascribe.toml` above it, so projects can be nested. Diagnostics, completion, the preview, format on save, and the other features use the project that owns the file. See [Workspaces with several projects](docs/content/guides/editor.md#workspaces-with-several-projects).
- A project's server starts the first time one of its files is opened or previewed. The new setting `ascribe.startServers` (`onDemand` or `all`) can start every project's server when the workspace opens instead. The Problems panel lists only the projects whose server is running; `ascribe check` covers any project.
- The extension serves at most 50 projects in a workspace, the first 50 by path, and says so in the first project's output when there are more.
- Each project uses its own `node_modules/.bin/ascribe`, and logs to its own output channel, `Ascribe (<project>)`.
- The extension warns about a project's `ascribe` older than 0.1.1: an older server checks a nested project's files as its own.
- **Ascribe: Restart Language Server** restarts every server that has started. **Ascribe: Show Server Output** shows the active file's project, or asks which project. The preview's build choice is kept for each project.

### The language server

- **Behavior change for editors other than VS Code:** `ascribe lsp` looks for `ascribe.toml` only in its workspace folder and the folders above it. A workspace folder whose projects are all in subfolders gets no project; the server doesn't pick one of them. Start one `ascribe lsp` per project, rooted at the folder that holds its `ascribe.toml` ([Other editors](docs/content/guides/editor.md#other-editors)).
- It logs the project it uses, or why it has none.
- `ascribe lsp --config` is an error (exit code `2`). The server takes its project from the editor's workspace folders, and ignored the option.

## 0.1.0 (2026-10-01)

The first release. It implements version 0.1 of the [Ascribe specification](SPEC.md).

### The language

- Directives: `@id`, `@include`, `@variant`, `@available`, `@note`, `@steps`, and `@details`, with attributes, titles, line and container forms, groups, and binding to headings and blocks.
- Project widgets: directives a project declares in `ascribe.toml`.
- Phrases (`{key}`), links and includes by file path, image attributes, heading ids, and a glossary.
- The content model, `ascribe.toml`: content types and frontmatter schemas, fragments, dimensions, versions, lifecycle states, features, note types, phrases, glossary, image attributes, widgets, the consumer profile, builds, and the editor's build.

### The `ascribe` command

- `ascribe check`: every diagnostic, for every build, as text or JSON.
- `ascribe build`: the site output (markdown with web components, and a generated Zod schema), the plain-markdown output, and the JSON output, for each build.
- `ascribe fmt`: rewrites Ascribe constructs into canonical form; `--check` for CI.
- `ascribe lsp`: the language server.
- 126 diagnostics, each with a code (`ASC001` to `ASC126`) and a stable name. See [docs/content/reference/diagnostics.md](docs/content/reference/diagnostics.md).
- Binaries for macOS on Apple silicon, Linux (arm64 and x64, glibc 2.39 or later), and Windows (x64). Intel Macs aren't supported. Each binary's archive and package carries a `THIRD-PARTY-NOTICES` file with the license text of the crates in it.

### The editor

- The VS Code extension: diagnostics as you type, completion, hover, go to definition, document links, CodeLens, inline hints for empty links, rename and move refactoring, quick fixes, formatting, semantic highlighting, and a live preview that renders pages as the site does.
- It uses the project's own `ascribe` when the project installs one, and the binary bundled with it otherwise.

### Astro

- `@ascribed/astro`: builds the site output before Astro loads content, provides the content collection and its generated schema, applies heading ids and image attributes in Astro's markdown pipeline, loads the element library, serves linked files, and rebuilds in `astro dev`.
- `@ascribed/elements`: the web components the site output uses (`<ascribe-note>`, `<ascribe-tabs>`, `<ascribe-availability>`, and the rest), styled with CSS custom properties.
