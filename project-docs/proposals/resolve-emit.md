# Proposal: a page in the site output, from the command line

A proposal, not a plan. It picks up section 22 of the [brainstorm](../brainstorm.md#22-a-page-in-any-output-from-the-command-line), "A page in any output, from the command line", and settles its questions.

## The goal

`ascribe resolve PAGE --emit site` writes one page as the site output writes it: Markdown with Ascribe's elements (`<ascribe-note>`, `<ascribe-tabs>`, availability badges), as `@ascribed/astro` receives it, without building the project. An agent without a shell gets the same from `ascribe_resolve` with `emit: "site"`.

It shows what a site's integration is given, which the plain output can't: what a switcher, a badge, or a glossary link becomes before a site draws it.

## Who it's for

- **An agent** checking what a reader of the site gets, not only a reader of the Markdown version. Without a shell, it can't get this any other way.
- **A writer or a reviewer** asking what a directive becomes, without building and opening the site; the docs audit needed a browser to see tabs and badges.
- **A test of a site's integration** that starts from one page.

## What exists

- **`ascribe resolve`** (`crates/ascribe-cli/src/commands/resolve.rs`, `ascribe_query::resolve`) resolves a page for one build and writes it as the plain output writes it, with `--build`, `--frontmatter`, and `--format`; or, for `go:<name>`, a permalink's help. Its JSON (`schemas/resolve.schema.json`) is `schema_version`, `ascribe_version`, `page`, `build`, `not_published`, `route`, and `text`. The MCP server's `ascribe_resolve` takes the same arguments.
- **One page in the site output** is already written outside a build: the editor's preview (`crates/ascribe-lsp/src/preview.rs`) resolves the page for its build and calls `ascribe_emit::emit_page` with a `SiteEmitter`, the steps `ascribe build --emit site` takes, then renders the result with `render_site_html`.
- **`--emit`** is `ascribe build`'s word for which outputs to write (`site`, `plain`, `json`).
- **Decision 4:** a JSON document gains fields without a new schema version. **Decision 56:** the MCP tools' arguments are a contract, and no tool writes a file.

## Settled

Answered by the maintainer on 24 October 2026.

1. **No HTML.** Every `--emit` value is an output a build writes, and HTML isn't one. The site output's Markdown already shows what a badge or a switcher becomes, and a person has the preview. If a test or an agent needs the HTML, it comes back as `--html` on `--emit site`, not as a third meaning for `--emit`: `render_site_html` over the page, without source anchors. The preview's HTML has them, so the two wouldn't be byte for byte the same, and nothing should expect them to be.
2. **The JSON says which output it is** with an `emit` field, `plain` or `site`. A later `json` would put the page's JSON under a key of its own, not as a string in `text`.
3. **`site` alone for now.** `--emit json` waits for a use; the field in 2 leaves room for it.

## The design

### The command

- **`--emit <plain|site>`**, one of the two, default `plain`, so today's command is unchanged. `ascribe resolve PAGE --emit plain` is what `ascribe resolve PAGE` writes now. It's `ascribe build`'s word with the same meaning, but `build` takes a comma-separated list (`--emit site,plain`) and this takes one output, so its help says "one of", and a list is a usage error naming the values, exit `2`.
- **`--emit site`** writes the page as `ascribe build --emit site` writes its file: resolved for the build, through `emit_page` with the build's `SiteEmitter`, the code the build and the preview use, so the three can't disagree. Asset references are as the site output writes them, relative to the page's file, and nothing is copied. An asset is named where a build would put it (asset contract §3): at its mirrored path, under `_ascribe/up/` when it's from outside the content root, or under `_ascribe/files/` for a linked file. None of those exists until a build writes it, so a test reading the answer can't follow an `_ascribe/` path to a file.
- **`--frontmatter`** means what it means today, the page's frontmatter first, but under `--emit site` it's a different document from the plain form's: the site output's, which an Astro content collection reads, with the reserved keys the output-layout and element contracts give it, such as `type`, `formatted` (the title's formatted form, as HTML), and `available` as a list of targets with their states, versions, and text. A reader who knows the plain form's frontmatter won't expect it, so the reference page shows it, and it's what makes `--emit site` useful to a test of a site's integration.
- **A line of a versioned build:** `--build site@3.4` works as it does for the plain form, and the page's links are the line's, under its base path (`/3.4/`), which is the case someone checks a line's page for.
- **No source anchors,** as `ascribe build` writes none without `--anchors`. The option isn't added: anchors are for the preview and review, which have their own way to ask.
- **`go:<name>` with `--emit`,** either value, is a usage error, exit `2`, saying a permalink's help has one form, `help.json`'s. `--emit` is absent unless given, so the default doesn't trip it. A `go:` answer's JSON is `resolve-permalink.schema.json`'s, which has no `emit`.
- **The text** is the file, written to standard output, as the plain form is.

### The JSON

`resolve.schema.json` gains `emit`, `"plain"` or `"site"`, present in every page answer, and stays at `schema_version` 1 (decision 4). A `go:` answer is `resolve-permalink.schema.json`'s document, which doesn't change. `text` holds the page as the output named writes it.

### For agents

`ascribe_resolve` gains `emit`, under the command's name and values, with `plain` the default. With `emit: "site"`, its text is the site output's Markdown and its `structuredContent` the command's JSON. It's an argument added to a tool, so it's recorded as a decision with the change (decision 56), though no caller breaks. The rules `ascribe agents sync` writes mention it where they mention `ascribe_resolve`, to see what a reader of the site gets, and point at the reference page's section on reading the site output (below).

### Elements without their script

The site output's elements are custom elements that `@ascribed/elements` defines. Their Markdown is readable without the script, and the reference page says what an agent reading it should know:

- **Tabs hold every panel,** as a page without JavaScript shows them; a reader sees one at a time.
- **An attribute marker isn't content.** `<ascribe-attributes id="connect"></ascribe-attributes>` after a heading or an image carries its attributes for the site to apply; the heading's text ends before it.
- **A glossary link's `title` is the term's definition,** written by the site output so a reader sees it on hover, beside a marker naming the term (element contract §7).
- **An availability badge** is an element with its marks as attributes.

## No thread

One layer: the command and its tool, over a path the preview already uses (`emit_page`). Nothing here is an unproven seam, so the steel-threads ordering doesn't apply.

## What it would look like

From `examples/quill`, the end of what `ascribe build --emit site` writes for the page today:

````console
$ ascribe resolve install-agent.md --build site --emit site
…
## Connect to Quill <ascribe-attributes id="connect"></ascribe-attributes>

<ascribe-tabs sync="deployment">

<ascribe-tab value="cloud" label="Quill Cloud">

Sign in to Quill Cloud and copy an API key from **Settings → Keys**, then add it to `quill.yaml`:

```yaml
cloud:
  api_key: ${QUILL_KEY}
```

</ascribe-tab>
…
````

The plain output writes the same section as a line saying a reader sees one arm at a time, then each arm as a section of its own. The site output's page has no `#` title: the title is in its frontmatter, which `--frontmatter` adds.

## What it changes that's a contract

Each with a decision recorded in the change:

- **The command:** `ascribe resolve`'s `--emit`, and `resolve.schema.json`'s `emit`.
- **The MCP tool:** `ascribe_resolve`'s `emit` argument (decision 56).

The site output itself doesn't change: this writes what it already writes.

## What to build

One pull request:

1. `--emit` on `ascribe resolve`, through `emit_page`, with output tests: a page with tabs, a badge, a glossary link, and an include; `--frontmatter` under `--emit site`, showing the site output's keys; a line of a versioned build, its links under the line's base path; a page the build doesn't publish; a list given to `--emit`, and `go:<name>` with `--emit`, each refused with exit `2`.
2. `emit` in the JSON and its schema, the generated schema page (`docs/content/contracts/json-reports/questions.md`), and a decision file.
3. `emit` on `ascribe_resolve`, its test, and the agents' rules; the surface map regenerated.
4. The docs: `docs/content/command/questions.md`'s `ascribe resolve`, with "What it would look like"'s example, quill's Connect section, as the tested example `ascribe.examples.toml` lists, observed again, since it shows the tabs the plain form can't; the section on reading the site output; `docs/content/agents/commands.md`; `changes/command/resolve-emit.md`.

## What this leaves out

- **HTML** (Settled 1), and **`--emit json`** (Settled 3), until a use turns up.
- **Source anchors** in the command's answer.
- **A whole build's site output for one page's assets:** the answer names assets as the site output does, and copies none.
