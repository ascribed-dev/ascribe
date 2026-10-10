# ascribe-lsp

The Ascribe language server, run as `ascribe lsp` (LSP over stdio). It keeps a
project in memory, follows every change to it, and publishes the diagnostics
`ascribe check --build <editor build>` reports, as the author types. It
computes nothing itself: every diagnostic comes from `ascribe_check`, over a
`ascribe_resolve::IncrementalProject`, so the editor and the command line can't
disagree (SPEC §8, §10).

It provides initialization, document and file synchronization, diagnostics, and
semantic tokens; completion, hover, go to definition, find references, document
links, CodeLens, and inlay hints (see [Navigation](#navigation)); code actions,
rename, and formatting; and the custom requests `ascribe/preview`,
`ascribe/review/setBase`, `ascribe/review/changes`, `ascribe/context`,
`ascribe/targets`, `ascribe/edit`, `ascribe/buildView`, and `ascribe/inventory`.

## The project

A server serves one project, which it finds at startup:

- Its workspace folders are `initialize`'s `workspaceFolders`, or its
  `rootUri` when there are none.
- For each folder in order, it looks for `ascribe.toml` in the folder and then
  in each parent, and takes the first it finds (`find_config` in
  `src/core.rs`). It never looks below a folder: a folder whose projects are
  all in subfolders has none.
- It logs one line to stderr saying what it found:
  `ascribe-lsp: using the project at <path to ascribe.toml>`,
  `ascribe-lsp: no ascribe.toml at or above <folders, comma-separated>`, or
  `ascribe-lsp: no workspace folder, so no project`.
- A server with no project publishes nothing, and the preview request answers
  with a problem saying so. When the file watcher reports an `ascribe.toml`
  created or changed directly in one of its workspace folders (not in a parent
  or a subfolder), it loads that project and logs `using the project at …`.
  That needs the client to allow the dynamic registration of
  `workspace/didChangeWatchedFiles`.
- Once a server has a project, it keeps it until it exits. It doesn't follow
  `workspace/didChangeWorkspaceFolders`.

A client with several projects starts one server per project, with the folder
that holds its `ascribe.toml` as the workspace folder, and sends each server
the documents of its own project; the VS Code extension does
(`packages/vscode/src/registry.ts`). The server checks only its own project's
sources: a directory below its content root that holds an `ascribe.toml`,
other than the project's own folder, is another project's folder, and nothing
in it is a source, whether on disk or open in the editor. When the file
watcher reports such an `ascribe.toml` created or deleted, the server loads
its project again and logs
`ascribe-lsp: a nested project appeared or went away; loading the project again`.

## Semantic token legend

**This legend is a contract with the VS Code client. The order of the
types and modifiers is what the wire format indexes, so entries are only ever
appended, never reordered or renamed.** The server sends the legend in its
`initialize` result; a client that reads it from there needs nothing from here
except the suggested theme scopes.

Token types, in legend order:

| # | Type | What it marks | Suggested TextMate scope (for `semanticTokenScopes`) |
|---|---|---|---|
| 0 | `ascribeDirective` | The `@` and name of a built-in directive (`@note`, `@include`, `@variant`, `@id`, …) | `keyword.control.directive.ascribe` |
| 1 | `ascribeWidget` | The `@` and name of a project widget (declared in `[widgets]`) | `entity.name.function.widget.ascribe` |
| 2 | `ascribeAttributeKey` | An attribute key in a directive's or an image's `{…}` block | `entity.other.attribute-name.ascribe` |
| 3 | `ascribeAttributeValue` | An attribute value: a token, a quoted string (with its quotes), or one member of a value set | `string.unquoted.attribute-value.ascribe` |
| 4 | `ascribeColon` | The `:` that ends a directive's head: the container colon, or the colon before a primary | `punctuation.separator.directive.ascribe` |
| 5 | `ascribeEnd` | The `@end` of an end line | `keyword.control.end.ascribe` |
| 6 | `ascribeTitle` | A title line: the `.` and the title's text (SPEC §3.7). Its own type, so a paragraph that accidentally became a title stands out (SPEC §10) | `markup.heading.title.ascribe` |
| 7 | `ascribePhrase` | A declared phrase, `{key}` with its braces (SPEC §5.1) | `variable.other.phrase.ascribe` |
| 8 | `ascribePhraseUndeclared` | A `{key}` whose key the content model doesn't declare: literal text, marked so a typo shows | `invalid.illegal.phrase-undeclared.ascribe` |
| 9 | `ascribeAvailability` | An availability spec: the primary of `@available` (`cloud, self-managed preview 3.3`) | `constant.other.availability.ascribe` |

Token modifiers, in legend order (bit *n* of a token's modifier set is
modifier *n*):

| # | Modifier | Set on |
|---|---|---|
| 0 | `unknown` | An `ascribeAttributeKey` the directive's schema (or, for an image, `[images.attributes]`) doesn't declare |

A token never overlaps another: a phrase inside a title line splits the title
token around it. Tokens that span lines are split per line. Positions and
lengths use the negotiated position encoding (below). The server serves
`textDocument/semanticTokens/full` and `…/range`; there is no delta.

## Position encoding

The server uses UTF-16 unless the client offers UTF-8 in
`general.positionEncodings`, and answers with the one it chose in
`positionEncoding`. Every conversion between a byte offset and a position goes
through `ascribe_core::LineIndex`. (The command line's JSON counts columns in
characters; the two differ for a line with an astral-plane character, and the
parity test converts before comparing.)

## The preview request: `ascribe/preview`

A custom request that renders a page the way the published site
does, for the editor's preview. It answers from the current snapshot, so it
includes unsaved edits, and it writes nothing. For a document and a build it
resolves the page for the build (`Project::resolve_page`), writes the site
markdown with `SiteEmitter` (`ascribe_emit::emit_page`), and renders it with
`ascribe_emit::render_site_html`, which implements the site-render contract
that the Astro plugin implements; both pass the fixtures in `tests/render/`,
so heading ids and image attributes are the site's. The preview is never
published, so it always has source anchors (site-render contract §7): every
block's element carries `data-ascribe-source`, and `data-ascribe-via` when it
came through includes, which the client uses to scroll the preview with the
editor by block. No capability is advertised: a client that wants it sends
the request.

**Params**

```jsonc
{
  "textDocument": { "uri": "file:///…/docs/install.md" },
  "build": "cloud",       // optional: a build name; the default is `[editor] build`
  "review": true          // optional: include what changed against the review base
}
```

**Result.** Always an object; `page` is `null` when there is nothing to show,
and `problems` says why. Field names are camelCase.

| Field | Meaning |
|---|---|
| `build` | The build the answer is for. |
| `builds` | Every build of the content model, in order: `{ name, editor, description }`. `editor` marks `[editor] build`, the picker's default. |
| `projectRoot`, `contentRoot` | Absolute paths. |
| `assetRoots` | Directories outside the content root that the page's assets are in and the preview may read: the directory of each asset that is in the project but not in the content root. The client may serve the content root and these, and nothing wider. |
| `documentVersion` | The version of the open document the answer is from, or `null` when the file isn't open. A client that sent version *n* and gets an older one has raced its own change notification and asks again. |
| `problems` | `{ severity: "error" \| "warning" \| "info", message }`: no project, an unknown build, a file that isn't a page (a fragment names the pages that include it), a page the build drops, an emit error, an asset the preview can't show. |
| `page.path`, `page.route` | The page's content path and its route on the site. |
| `page.title` | The page's title, phrases substituted. |
| `page.formattedTitle` | The title formatted, when its field sets `inline = "code"`: a list of `{ type: "text" \| "code", value }` pieces, as the JSON output's `formatted.title`, for the preview's heading. `null` when the field doesn't set `inline`. |
| `page.frontmatter` | What the site output writes as frontmatter, as JSON. `available` is the list of targets a layout hands to `<ascribe-availability>`. |
| `page.html` | The page's content as HTML, with source anchors, without frontmatter and without a layout. |
| `page.assets` | Each asset the page uses: `{ reference, path, kind, servable }`. `reference` is what the HTML writes, before any `#fragment`: an image's `src` is relative to the page (`./_fragments/a.png`), a link target's `href` is the site URL. `path` is the absolute source file, **resolved from the file the reference is written in** (asset contract §7), so a fragment's image is the one beside the fragment. `servable` is `true` when the file is in the content root or in a directory of `assetRoots`; a file directly in the project root, in `node_modules` or `.git`, or in the output directory isn't served, and `problems` says so. References are percent-encoded as URLs are; compare them after normalizing (`packages/vscode/src/preview/refs.ts` does). |
| `page.links` | Each link to a page: `{ href, path, id }`, `href` as the HTML writes it, `path` the target file, `id` the heading it names. |
| `page.sections` | The headings written in the previewed file itself, in order: `{ id, line }`, `line` from 0. The HTML's anchors locate every block, headings included. |
| `review` | With `review: true` and a base set (below): `{ base, changes, wasHtml }`. `base` is what the base resolved to. `changes` is the page as `ascribe diff --format json` reports it (`status`, `own_file_changed`, `because`, `page_changed`, `counts`, `changes`; its keys are snake_case), or `null` when the page didn't change. `wasHtml` is the page as it was at the base, rendered as `page.html` is, with anchors, for showing removed blocks and changed blocks as they were; `null` for a new page or no change. `null` without a base or a page. |

Every problem is in `problems`, not in a JSON-RPC error: a malformed request
(parameters that don't parse) is the only error, `InvalidParams`. The request runs on the server's main loop, so it must not grow with the
project: it emits one page, indexes only the files a position is asked for
(`EmitContext` builds line indexes lazily), and finds pages that share a route
once per set of pages (a cache keyed by the model revision and a hash of the
page paths), not per request. See the table under Performance.

## Review: `ascribe/review/setBase` and `ascribe/review/changes`

Two custom requests compare the project with a git revision, its **base**,
as `ascribe diff` does (`ascribe-diff`), so the preview can mark what changed.
Nothing runs `git` until a base is set: a server without `git` on the path
works as before, and `setBase` says why it can't.

**`ascribe/review/setBase`** resolves a revision as `ascribe diff` does,
from the merge base of it and `HEAD`, reads the project as it is there through
`ascribe_diff::Revision` (one `git cat-file --batch`), and keeps it beside the
live snapshot. The base isn't watched. Setting it again resolves the revision
again (two quick `git` calls) and reads the project there only when the commit
compared with moved, after a pull, rebase, or fetch; otherwise it keeps the
base it has. The editor does that when the preview regains focus. The request
runs `git` on the main loop, without the server's lock.

```jsonc
{ "base": "main" }  // a branch, tag, or commit
{}                  // the default branch: the first of origin/HEAD, origin/main, origin/master, main, master
{ "base": null }    // stop comparing, and free the base
```

The result is `{ base, problem }`. `base` is `{ requested, commit, merge_base }`
(as in `ascribe diff`'s report), or `null` once dropped or when it couldn't be
set. `problem` says why it couldn't: no project, not a git repository, an
unknown revision, no default branch, a shallow clone, `git` missing. A base
that fails leaves the one set before.

**`ascribe/review/changes`** (`{ "build": "cloud" }`, optional, the editor's
build by default) lists the build's changed pages against the base, computed
from the current snapshot, so unsaved edits count. The result is
`{ build, base, contentRoot, pages, problem }`: `pages` are `ascribe diff`'s
pages without their `changes`, each with its `title` and `formatted_title`
(the title's pieces, as `page.formattedTitle`, or `null`), in path order;
`problem` is set when review is off. It compares every page of the build, so
it's for listing on demand, not per keystroke: the preview's `review: true`
compares only its page (`ascribe_diff::compare_page_in`). The base keeps the
last list with the snapshot it came from, so asking again before the next
edit is free, and the pages as they were, rendered once each.

A base costs about as much memory as the project: on the synthetic
3,000-page project, the project's source index is 67 MB and the base adds
another 67 MB, all freed when it's dropped. Reading it takes about a second.

## What's at a position: `ascribe/context`

A custom request that says what is at a position or a selection of a page,
so a client knows which actions apply there: the editor's actions are built
on it, and any client can use it. It answers from the current snapshot, so it
includes unsaved edits, and holds nothing about the rest of the project (that's
`ascribe/targets`). No capability is advertised: a client that wants it sends
the request.

```jsonc
{
  "textDocument": { "uri": "file:///…/docs/install.md" },
  "range": { "start": { "line": 20, "character": 5 }, "end": { "line": 20, "character": 5 } }
}
```

An empty range is the cursor. Every range in the answer is in the negotiated
position encoding. The result, whose TypeScript type is `ContextResult` in
`packages/vscode/src/shapes.ts` (`schemas/lsp-context.schema.json`):

| Field | Meaning |
|---|---|
| `project` | `{ root, editorBuild }`: the directory of `ascribe.toml`, and `[editor] build`. |
| `at` | What contains the start of the range, innermost first, each `{ kind, range, … }`. |
| `selection` | For a non-empty range, `{ kind, text, inline }`; `null` for the cursor. |
| `token` | The token under the start of the range, `{ kind, range, … }`, or `null`. |
| `insertable` | Whether the cursor's line is blank and between blocks, where a block can go: not in a code block or the frontmatter. |

The kinds in `at`, and what each adds to `range`:

| Kind | Adds |
|---|---|
| `frontmatter` | `variant` and `available`: each `{ range, value }` of the key's value, or `null` |
| `section` | `headingId` |
| `heading` | `level`, `id` (its source id), `explicitId` (written with `@id`) |
| `paragraph`, `listItem`, `blockQuote`, `table` | |
| `list` | `ordered`, and `steps` when a `@steps` binds it |
| `note` | `type`, `form`: `line` (a primary), `block` (binds the next block), or `container` |
| `details` | `title` as written, `form`: `block` or `container` |
| `steps` | The `@steps` line and its list |
| `variantGroup` | `dimension` (the key every arm has; `null` for labeled arms), `arms` (`{ value, label, range }`), `arm` (the index the position is in) |
| `availability` | `spec`. At the top of a section, the line; binding a block, the line and the block |
| `include` | `path` as written, `section` |
| `snippet` | `address` as written |
| `widget` | `name`, `attributes` (`{ key, value }`; of the arm the position is in, for a group), `form`: `line`, `block`, `container`, or `group` |
| `codeBlock` | `info`, `fenced` |
| `tableRow` | `header`, `available` (its `available` attribute) |
| `link` | `destination`, `textEmpty` |
| `image` | `src`, `alt` as written, `attributes` |
| `phrase` | `key`, `declared` |

A directive that binds the block below it (`@note` without a colon,
`@details`, `@steps`, `@available`, a widget) contains that block: the
cursor in the block has the directive in its chain.

`selection.kind` is decided with whitespace at either end left out:

| Kind | When |
|---|---|
| `prose` | Inside one paragraph's or heading's text, or all of it (`inline` is `true` only for this kind) |
| `blocks` | Whole blocks of any kind, side by side: whole lists, whole containers, whole groups |
| `code` | Inside one code block |
| `mixed` | Anything else that spans blocks: part of one of them, a directive's own lines (an opener, its title, an `@end`) with some of what it holds, or some of a list's items or a group's arms |
| `other` | Part of one table or directive line, or the frontmatter |

`token.kind` is `link` (`destination`, `textEmpty`), `image` (`src`),
`include` (the path and its `#id`: `path`, `section`), `phrase` (`key`,
`declared`), `directiveName` (`name`), or `attribute` (`directive`, `key`,
`value`; on any part of `key=value`).

A document that isn't a source file of the project (outside the content
root, or in a nested project's folder) gets the empty answer: `project` is
`null` and `at` is empty. The request is answered in `src/context.rs`, from
the syntax tree and the file's index, as navigation is (`nav.rs`).

## What actions can point at: `ascribe/targets`

A custom request that lists what an action can point at or use in the page's
project, so a client can offer choices instead of asking for syntax. The
client asks for the kinds it needs:

```jsonc
{
  "textDocument": { "uri": "file:///…/docs/guide/install.md" },
  "kinds": ["pages", "headings", "fragments", "images", "snippets", "phrases",
            "notes", "dimensions", "widgets", "features", "builds", "occurrences"],
  "range": { "start": { "line": 8, "character": 4 }, "end": { "line": 8, "character": 15 } }
}
```

`range`, the cursor or selection, is optional; only `occurrences` reads it.

The result has a list for each kind asked for and no others, and `modelUri`,
the `file:` URI of `ascribe.toml`, which declaration ranges are in. Its
TypeScript type is `TargetsResult` in `packages/vscode/src/shapes.ts`
(`schemas/lsp-targets.schema.json`). Paths are written from the requesting
page, as completion writes them (`relative_path`, percent-encoded where a
destination needs it):

| Kind | Each entry |
|---|---|
| `pages` | `path` (content path), `title`, `type` (content type), `link` (the destination from the requesting page), `rootLink` (`/page.md`, from the content root) |
| `headings` | `page`, `text`, `id`, `level`, `link` (`page.md#id`, or `#id` on the requesting page), `rootLink` (`/page.md#id`, from the content root). A page's headings include those of the fragments it includes, each id once |
| `fragments` | `path`, `include` (the `@include` path from the requesting page), `startsWithHeading` |
| `images` | `path` (from the project root), `link`. Image files under the content root, not in a folder whose name starts with `.`, `node_modules`, a nested project's folder, or the output directory |
| `snippets` | One entry per `[sources.<name>]`: `name`, `files` (`{ path, address, regions }`, each region `{ name, address }`) |
| `phrases` | `key`, `value`, `range` (its key in `ascribe.toml`) |
| `notes` | `type`, `label` |
| `dimensions` | `name`, `label`, `values` (`{ value, label, versionless, range }`, `range` inside the value's quotes in `values`) |
| `widgets` | `name`, `description`, `line`, `container`, `groupable`, `primary` (`none`, `identifier`, `text`), `binding`, `attributes` (`{ key, type, values, required, default, description }`) |
| `features` | `key`, `name`, `availability` (the spec as written), `range` (its table header) |
| `builds` | `name`, `editor` (whether it's `[editor] build`) |
| `occurrences` | `path` (content path), `range`: each other whole-word occurrence, in the project's prose, of the text `range` selects, which `makePhrase` with `everywhere` replaces. Empty when the selection isn't text a phrase can take the place of |

Images and the files of sources are listed through the project's
`FileSystem`. A source's files are those its `include` and `ignore` take in,
read by the code that resolves `@snippet` (`ascribe_resolve::source_files`),
so every address listed resolves; a file that isn't text, or whose tags have
problems, isn't listed. A source in another repository lists its copies under
`sources/<name>/`; nothing reaches the network. A declaration's `range` is
found by reading `ascribe.toml`'s text (`find_entry` in `src/definition.rs`),
since the model keeps no spans; it's `null` when the entry isn't written as
a table and key there.

Everything comes from the current snapshot and model, so it includes unsaved
edits. The request can be sent for any of the project's files: for
`ascribe.toml`, or another file in the project's folder that isn't a source
(not in a nested project's folder), the lists are the same, with paths written
from the content root. A document that isn't one of the project's files gets
`{}`.

## Page edits: `ascribe/edit`

A custom request that performs one action on a page and returns the edit,
so a client can offer actions without writing syntax itself. It answers
from the current snapshot, so it sees unsaved edits. No capability is
advertised: a client that wants it sends the request.

```jsonc
{
  "textDocument": { "uri": "file:///…/docs/install.md" },
  "range": { "start": { "line": 20, "character": 0 }, "end": { "line": 20, "character": 0 } },
  "action": "wrapNote",
  "args": { "type": "tip" },
  "version": 12
}
```

`range` is the cursor or selection the action was invoked on, `args` what
the action needs (each operation's are below; missing means `{}`), and
`version`, optional, the document version the client saw. The result, whose
TypeScript type is `EditResult` in `packages/vscode/src/shapes.ts`
(`schemas/lsp-edit.schema.json`), is one of:

| Result | Meaning |
|---|---|
| `{ edit, select }` | `edit` is a `WorkspaceEdit` whose `changes` hold plain text edits to the requested document, and, for the content model's actions, to `ascribe.toml` and any other page they change. `select` is the placeholder text the edit wrote, as a range in the document after the edit, for the client to leave selected; `null` when it wrote none. |
| `{ error }` | A plain-language sentence for the client to show: the document's version isn't `version` (the page changed after the action was chosen), the action doesn't apply at the range, an argument is invalid (naming the valid choices), or the edit would make a problem the page didn't have. |

The operations, where each applies, and its arguments (`?` is optional):

| Action | Applies to | Args | Writes |
|---|---|---|---|
| `wrapNote` | One paragraph, or whole blocks | `type?` (default `note`) | `@note` before one paragraph, or a container (`@note:` … `@end`) around several blocks; `{type=…}` unless the type is `note` |
| `setNoteType` | A note | `type` | The note's `type` attribute, or none for `note` |
| `unwrapNote` | A note | | The note's content, without the directive |
| `noteToDetails` | A note | `title` | `.Title` and `@details`, keeping the content and the form |
| `wrapDetails` | One block, or whole blocks | `title` | `.Title` and `@details` (or `@details:` … `@end`) |
| `unwrapDetails` | A details block | | Its content, without the title and the directive |
| `makeSteps` | An ordered list | | `@steps` above it |
| `removeSteps` | Steps | | The ordered list, without `@steps` |
| `addHeadingId` | A heading without `@id` | `id?` (default its slug) | `@id: …` after the heading |
| `insertNote` | An insertable line | `type?`, `text?` | A note; without `text`, a placeholder |
| `insertSteps` | An insertable line | `count?` (1 to 50, default 3) | `@steps` and a numbered list of placeholders |
| `insertVariantGroup` | An insertable line | `dimension`, `values` | One `@variant {dimension=value}:` arm per value, each with a placeholder, and `@end` |
| `addVariantArm` | A group of one dimension's arms | `value` | A new arm, in the dimension's declared order |
| `removeVariantArm` | An arm of a group with others | | The group without it |
| `insertDetails` | An insertable line | `title` | `.Title`, `@details:`, a placeholder, `@end` |
| `insertInclude` | An insertable line | `path` (with `#id` for a section) | `@include: …` |
| `insertSnippet` | An insertable line | `address`, `lang?`, `title?` | `@snippet: …`, with the attributes given |
| `insertImage` | An insertable line | `path`, `alt`, `attributes?` | An image of `path`, with `alt` as its text and the attributes after it |
| `insertWidget` | An insertable line | `name`, `primary?`, `attributes?` | The widget, attributes in declared order; the container form with a placeholder when it takes one, and a `.Title` placeholder when its title is required |
| `markAvailable` | A heading (its section), a block, or a table's body row | `spec` (a spec or a feature key) | `@available: …` under the heading or before the block; for a row, `{available=…}` at the end of its first cell |
| `setPageVariant` | Anywhere in a page | `dimension`, `value` | That dimension's value under `variant:` in the frontmatter, keeping the rest |
| `setPageAvailable` | Anywhere in a page | `spec` | `available:` in the frontmatter |
| `linkSelection` | Prose selected in one paragraph or heading | `destination` | A link to `destination`, with the selection as its text |
| `insertLink` | A cursor in prose | `destination` | A link to `destination` with empty text, so the target's title fills it |
| `insertPhrase` | A cursor in prose | `key` | `{key}` |
| `setLinkTarget` | A link | `destination` | The link's destination |
| `useTargetTitle` | A link to a page | | Empty link text, so the target's title fills it |
| `setImageWidth` | An image | `width` | The image's `width` attribute, when the model declares one |
| `setImageAlt` | An image | `alt` | The image's alt text |
| `makePhrase` | Plain text selected in prose, on one line | `key`, `everywhere?` (default `false`) | `key = "<the text>"` in `[phrases]`, and `{key}` in place of the selection; with `everywhere`, in place of each of its `occurrences` too |
| `addGlossaryTerm` | Anywhere in a page | `id`, `term`, `definition`, `aliases?` (a list), `link?` (a page from the content root, with `#id`) | A `[glossary.terms.<id>]` table with those keys |
| `promoteFeature` | Anywhere in a page | `key`, `spec` | The feature's `available` in `[features.<key>]` |

An insertable line is where `ascribe/context` says `insertable`: a blank
line between blocks. `attributes` is an object of key to value (a string,
number, boolean, or a list for a set). Names, types, dimensions, values,
specs, keys, pages, fragments, images, snippet addresses, and widget
attributes are checked against the project and its model; the valid choices
are what `ascribe/targets` lists.

What every edit holds to:

- **Canonical.** `ascribe fmt` changes nothing the edit wrote: directives,
  attributes, and images are written by `ascribe-fmt`'s own functions.
- **Minimal.** The edit touches only what the action changes, and keeps the
  rest of the page, the frontmatter's other fields and comments included, as
  written.
- **In place.** Inside a list item or a block quote, what's written is
  indented or prefixed to match, and a blank line is added where the blocks
  around it need one.
- **No new problems.** An edit that would add a diagnostic in the editor
  build, to the page, to a page that includes it, or to a page that links
  to either, is refused with an error instead, naming the other page when
  the problem is there.
- **The content model in place.** The content model's actions edit
  `ascribe.toml` as text (`src/model_file.rs`), as it is in the editor when
  it's open: an entry goes after the last one of its table, a new table after
  the last of its section, and a value is replaced where it's written, so
  comments, blank lines, and order stay. `toml_edit` finds the place from
  its spans and renders the new text, and the edited file is checked to hold
  what `toml_edit`'s own API would have made of it. A key or value the model
  doesn't allow (a key's syntax, a key that's taken, a spec that names
  nothing) is an error before any edit, and so is a model that doesn't load
  as it is. The edits are checked as the whole project, with the model as it
  would be: an action that would add a diagnostic anywhere is refused.
- **Every wrap has an unwrap.** `wrapNote` and `unwrapNote`, `wrapDetails`
  and `unwrapDetails`, `makeSteps` and `removeSteps` undo each other: one
  and then the other gives back the original text.

A document that isn't a source file of the project gets an error. The
request is answered in `src/edit.rs` and the files under `src/edit/`, with
the targets found as `ascribe/context` finds them.

## The project's inventory: `ascribe/inventory`

A custom request that answers what the project has and how much each part is
used, for the editor's Pages and Content model views:

```jsonc
{ "textDocument": { "uri": "file:///…/ascribe.toml" } }
```

Like `ascribe/targets`, it can be sent for any of the project's files. Its
TypeScript type is `InventoryResult` in `packages/vscode/src/shapes.ts`
(`schemas/lsp-inventory.schema.json`):

```jsonc
{
  "modelUri": "file:///…/ascribe.toml",
  "contentUri": "file:///…/docs",
  "pages": [{ "path": "guide.md", "title": "Guide", "type": "page", "incoming": 3 }],
  "fragments": [{ "path": "_setup.md", "includedBy": ["guide.md"] }],
  "orphans": ["old.md"],
  "model": [{ "kind": "phrase", "key": "product", "label": "Quill", "uses": 14,
              "declaration": { "start": …, "end": … } }]
}
```

- Paths are content paths, relative to `contentUri`, the content root.
- `incoming` is how many links from other files, and includes, name the page.
- `orphans` are the pages whose `incoming` is 0, other than index pages
  (`index.md`, in any folder). A reader may still reach one through a
  navigation the project doesn't know about, so it's a hint, not an error.
- `model` lists the phrases, features, glossary terms, dimensions, note types,
  widgets, and builds, each kind in declaration order. `kind` is `phrase`,
  `feature`, `term`, `dimension`, `note`, `widget`, or `build`; `label` is the
  phrase's value, the feature's name, the term, the dimension's label (when it
  isn't its name), the note type's label, or the widget's description.
  `uses` is `null` for a build, which pages don't name, and for a glossary
  term with `match = "marked"`, whose uses are the links to its page.
  `declaration` is the entry's table header in `ascribe.toml` (a phrase's
  key), in `modelUri`, found as `ascribe/targets` finds ranges; it's `null`
  for a built-in note type.

Every count is the length of the list Find All References returns for the same
thing ([Navigation](#navigation)): both come from
`ascribe_resolve::Project::uses`, and the inventory counts every kind in one
pass over the snapshot (`Project::use_counts`). It answers from the current
snapshot, so it includes unsaved edits. See [Performance](#performance) for
its cost.

## What a build leaves out: `ascribe/buildView`

A custom request that says what a build leaves out of a page, for a client
that dims it in the source (the editor's build lens). The client names the
page and the build; without `build`, the editor's build (`[editor] build`):

```jsonc
{ "textDocument": { "uri": "file:///…/docs/guides/rollouts.md" }, "build": "self-hosted" }
```

The result's TypeScript type is `BuildViewResult` in
`packages/vscode/src/shapes.ts` (`schemas/lsp-build-view.schema.json`):

```jsonc
{
  "build": "self-hosted",
  "documentVersion": 4,          // the open document's version, or null
  "pageIncluded": true,          // false when the build drops the whole page
  "pageDetail": null,            // why, when it does
  "excluded": [
    { "range": …, "reason": "variant", "detail": "Shows only edition=self-hosted" },
    { "range": …, "reason": "availability",
      "detail": "Scheduled rollouts: available on Lantern Cloud (preview), not Self-hosted 2.5" }
  ]
}
```

What's excluded is what `ascribe build` removes, from the resolver's own
decisions (`Project::removed` and `Project::dropped` in `ascribe-resolve`),
not worked out a second way: the variant arms the build's selection doesn't
select (a group none of whose arms survives, whole), and the content its
availability filter removes, a section from its heading through its last
block, and a table row as its line. A range runs from a directive line
through the last line, `@end` included, and neighbors left out for the same
reason are one range. Only the page's own text is listed: what an
`@include` brings in is in another file. The detail uses the content model's
display labels. A `switch` and `badge` build leaves nothing out.

The answer comes from the current snapshot, so it includes unsaved edits. A
document that isn't a source file of the project, or a build the content
model doesn't have, gets an empty `build` and nothing excluded.

## A prompt for an agent: `ascribe/agentPrompt`

A custom request for **Prompt agent**: a prompt for the user's agent about one
problem, a file's problems, or the project's. The client says which, and lists
the documents with unsaved changes, so a prompt about one says to save it:

```jsonc
{ "kind": "problem",                  // or "file", or "project"
  "textDocument": { "uri": "file:///…/docs/guides/install.md" },  // optional for "project"
  "diagnostic": { "range": …, "code": "ASC036", "message": "…" },  // for "problem", as published
  "unsaved": ["file:///…/docs/guides/install.md"] }
```

The result's TypeScript type is `AgentPromptResult` in
`packages/vscode/src/shapes.ts` (`schemas/lsp-agent-prompt.schema.json`):
`{ "prompt": "Fix this problem in …" }`, or `null` when there's no problem: the
file has none, or the diagnostic isn't reported any more.

The answer comes from the current snapshot, so it includes unsaved edits, and
from what `ascribe check --editor-build` checks: the file-level checks and the
editor's build (`ascribe_check::diagnose_editor_build`). The prompt is built
by `ascribe_check::prompt`, which `ascribe check --format prompt` calls too,
so for a saved file the two give the same prompt; `lsp_parity` in
`crates/ascribe-cli/tests/` holds them to it. A diagnostic is found by its
code and range, and its message when two share a place.

While review is on (`ascribe/review/setBase`), two more kinds prompt about
changes, against the base the server holds:

```jsonc
{ "kind": "pageChanges",              // what changed on this page
  "textDocument": { "uri": "file:///…/docs/guides/install.md" },
  "build": "site",                    // optional: the editor's build
  "unsaved": [] }
{ "kind": "fragmentReach",            // the pages that changed through a fragment
  "fragment": "_fragments/prereqs.md", // a content path, as a page's `because` names it
  "build": "site",
  "unsaved": [] }
```

They answer `null` when review is off, or the page or fragment didn't change
in the build. The prompts are built by `ascribe_diff::prompt`, as `ascribe diff
--format prompt [PAGE]` builds them, so for saved files the two give the same
prompt.

## Capabilities

Advertised: incremental text document sync (open/close, no save), semantic
tokens (full and range), the position encoding, and completion
(triggered by `@ { ( # / = , |` and a space), hover, definition, references,
document links, CodeLens, inlay hints, code actions, document formatting, rename (with
`prepareRename`), and one command (`ascribe.openFile`, below). Workspace file-rename handling is advertised
for files. Registered dynamically after `initialized`, when the client allows it:
`workspace/didChangeWatchedFiles`.
Diagnostics are pushed (`textDocument/publishDiagnostics`); the server doesn't
advertise pull diagnostics.

Code actions carry the checker's existing diagnostic fixes (including the
router's reverse route suggestion) and the Ascribe-specific repairs. Rename and
file-move edits use the current project snapshot, including open buffers, its
source index, and reverse references. A rename starts on a heading or an `@id`,
a phrase (a `{key}` use or its key in `ascribe.toml`), or a dimension value (in
a `@variant` attribute or a `[dimensions.<name>] values` item). A phrase rename
reaches every use the source index records: prose, link text and destinations,
frontmatter fields with `phrases = true`, code blocks with `phrases=true`, and
the files `@snippet {phrases=true}` reads (a remote source refuses the rename).
A dimension value rename edits the dimension's `values`, `labels`, and
`versionless`, every `@variant` attribute and `variant` frontmatter field, every
availability spec (markers, frontmatter, `[features]`, and a build's `filter`),
and a build's `variants`. `ascribe.toml` is edited in place, keeping its
comments and layout. A new name that isn't valid or is already taken gets no
edit. `prepareRename` answers with the range and the current name, or says why
nothing at the cursor can be renamed. Formatting returns only the minimal edits
from `ascribe-fmt`; the VS Code client applies those edits on save when
`ascribe.formatOnSave` is enabled, and requests file-move edits before renaming.

## How it works

- **The project** is an `ascribe_resolve::IncrementalProject` built from the
  `ascribe.toml` found as [The project](#the-project) says. File ids follow `ascribe-resolve`: source files
  have ids from 1, `ascribe.toml` is 0, an id names a path and is never reused.
- **Changes** reach it as `Change`s from three sources: open documents
  (`didOpen`, `didChange`, `didClose`; an open document's text wins over the
  file on disk, and closing one reverts to the disk), the file watcher (files
  and directories created, changed, deleted, or moved by anything else, sources
  and assets alike), and `ascribe.toml` (open or on disk).
- **The model.** An `ascribe.toml` that loads is applied as `Change::Model`
  (`ascribe-resolve`'s tiers decide what is re-parsed, re-indexed, or re-resolved). One
  that doesn't load has its problems published on `ascribe.toml`, and the
  project keeps the last model that did. A change to the content root or
  output directory can't be applied in place (`ApplyError::LayoutChanged`),
  so the server reloads the whole project and republishes everything.
- **Diagnostics** are `ascribe_check::check_file` for each file in
  `Affected::recheck`, plus the page-level diagnostics of the editor's build
  (`[editor] build`; `ContentModel::editor_default_build`) located in those
  files, published for every affected file, open or not. Each one's code links
  to its entry in the diagnostics reference (`codeDescription.href`, the same
  address as `docs` in `ascribe check`'s JSON), so the code is a link in the
  Problems panel. A deleted file's diagnostics are cleared. The file-level checks probe the disk with the files
  the editor and the watcher have reported layered over it
  (`Project::from_parts_with_fs`), as the source index does.
- **Stale results.** A worker thread computes from a `Snapshot`. Before it
  publishes a file's diagnostics it checks, under the lock that serializes
  every change, that `IncrementalProject::is_file_current` holds for the file
  and that an open document still has the version the result was computed for.
  A result that fails either check is dropped, and the file stays queued for the
  newer snapshot. Work overtaken by a newer edit is abandoned between stages.
- **Robustness.** A panic while handling a request or a notification, or while
  computing diagnostics, is caught, logged to stderr, and answered with an
  internal error where a response is owed; the server keeps running. Nothing is
  ever written to stdout except LSP messages.

## Navigation

Every request is answered from the snapshot the project has when the request
is handled (taken under the lock, computed without it), so an answer never
comes from a stale project: a request that follows an edit sees the edit. A
document that isn't a source file of the project (an `untitled:` buffer, a file
outside the content root, `ascribe.toml`) gets `null`. All positions go through
the negotiated encoding and `LineIndex`.

**One implementation of the reference rules.** What a link, an image, or an
include names, and whether it's there, comes from the source index
(`Project::resolutions`, `FileIndex::includes`), which calls
`ascribe_resolve::references`, the code `ascribe check` uses. The features
below only *read* those results; where a request needs to resolve text that
isn't in the index yet (a destination being typed), they call
`references::reference_target` and `references::include_target`. Availability
text and labels come from `ascribe_emit::labels`, the code the emitter builds
`<ascribe-availability>` from (element contract §0 and §4).

| Module | Feature |
|---|---|
| `complete.rs` | Completion |
| `hover.rs` | Hover |
| `definition.rs` | Go to definition |
| `references.rs` | Find all references |
| `links.rs` | Document links, CodeLens, inlay hints, and the `ascribe.openFile` command |
| `nav.rs` | What they share: the request's state (`Ctx`), what is under the cursor (`hit_at`), previews, relative paths |

**Completion** is decided from the line up to the cursor (a file being typed
rarely parses into the construct the author is in the middle of), and, for what
each context offers, from the content model and the index:

| Cursor | Offers |
|---|---|
| `@` at the start of a line (also after `>` and list markers) | Built-in directives and project widgets, with their descriptions, and `end` |
| In `{…}` after a directive's name | Its schema's attribute keys (not those already written); after `=`, the allowed values: an enumeration, `true`/`false`, note types, or a dimension's values with their display labels; after `\|`, the members not yet chosen. `@variant`'s keys are the model's dimensions |
| In `{…}` after an image | `[images.attributes]`'s keys |
| `@available:` and frontmatter `available:` | At the start: targets (values with labels), dimension names, feature keys (only as the whole spec). After a target: lifecycle states. In a history `(…)`: states |
| `{` in prose, a title, a heading, a text primary, or a `phrases=true` fence | Declared phrase keys, each showing its value |
| `@include:` | Source files, relative to the file (or from the content root after a `/`); after `#`, the ids of the file named |
| A link destination, `](…`, or a reference definition | Pages and headings by **title**: inserts the relative path and the source id, shows the path as the detail |

Nothing is offered inside inline code or a code fence. A search is cut at 100
items and marked incomplete, so the client asks again as the author types.

**Hover**: a link or image (its project-relative path, the title, a
plain-text preview of the first paragraph), an include (the same; a fragment
says so), a phrase (its value), an availability spec or feature key (SPEC
§9.4's text), and a directive or attribute key (its schema).

**Go to definition**: links and includes to the file or heading;
`@id`'s primary to its heading; phrases and feature keys to their entries in
`ascribe.toml`.

**Find all references** lists every place that uses what's under the cursor,
from `ascribe_resolve::Project::uses`, the search the inventory counts with:

| Cursor | Lists |
|---|---|
| A link or an `@include` | What it names: the links and includes of the heading after `#`, else of the file |
| A heading, or its `@id`'s primary | The links whose `#id` names it, on its page or on a page that includes its file, and the includes of its section |
| The start of the file, its frontmatter, or its title | The links from other files to the file, and its includes |
| A `{key}` | Every declared `{key}` |
| An availability spec that is a feature key | The specs that are that key (`@available`, a row's `available`, frontmatter `available`); a spec of targets lists the uses of its dimension |
| A glossary term in prose | Its occurrences in prose, and its aliases', matched as the build matches them (whole words, its case rule), whether it links them first or every time. A term with `match = "marked"` has none: its uses are the links to its page |
| A `@note`, or its `type` | The notes of that type (`note` when none is given) |
| A widget's name | Its directives |
| A `@variant` or its attribute | The dimension's `@variant` attributes, frontmatter `variant` keys, and the availability specs that name it or its values |

With `includeDeclaration`, the declaration comes first: the file's start, the
heading, or the entry in `ascribe.toml`.

**Document links** make every link, image, and include destination
clickable, with `#L<line>` for a heading. **CodeLens** puts
`Includes <file> › <heading>` above each `@include`. Its command,
`ascribe.openFile`, is the server's own (`workspace/executeCommand`, which the
language client wires from the capability): it answers by sending
`window/showDocument`, so **no client code is needed**. **Inlay hints**
show the resolved title inside the `[` of an empty-text link.

## Performance

`cargo bench -p ascribe-lsp --bench keystroke` types into a page, and into a
fragment that pages include, in a project of each size (every page has an
include, three links, an image, and headings) and times the wait from
`didChange` to the `publishDiagnostics` for that version. Release build, one
run on a 4-core 2.1 GHz Xeon container (a developer laptop is faster):

| Pages | Load and first diagnostics | Page keystroke, median (p95) | Fragment keystroke, median (p95), pages including it |
|---|---|---|---|
| 20 | 12 ms | 0.9 ms (1.2) | 0.8 ms (1.1), 0 |
| 100 | 36 ms | 1.0 ms (1.4) | 0.8 ms (1.1), 1 |
| 300 | 78 ms | 1.0 ms (1.4) | 1.5 ms (1.9), 3 |
| 1,000 | 286 ms | 2.0 ms (2.5) | 3.8 ms (4.7), 10 |
| 3,000 | 825 ms | 3.0 ms (3.6) | 9.0 ms (11.8), 30 |

The same benchmark then sends `ascribe/preview` for the page after each
keystroke, as the editor does after its debounce (release build, same machine):

| Pages | Change to preview answer, median (p95) | Change to diagnostics with the preview open, median (p95) |
|---|---|---|
| 20 | 0.5 ms (0.8) | 0.7 ms (1.0) |
| 100 | 0.8 ms (1.0) | 0.7 ms (1.0) |
| 300 | 0.6 ms (0.7) | 0.9 ms (1.0) |
| 1,000 | 0.9 ms (1.1) | 1.4 ms (1.7) |
| 3,000 | 1.1 ms (1.7) | 3.0 ms (5.8) |

The preview's cost doesn't grow with the project, and the diagnostics numbers
are what they are without it (3.1 ms at 3,000 pages). A first version listed
every page, tested whether the build drops it, and slugged every path for the
route check, and took 9.6 ms at 3,000 pages (and slowed diagnostics to 4.0 ms);
it also built a line index of every file for each request.

Both are far under the 50 ms target at 3,000 pages. What is left that grows
with the project is small: building the checked project from the snapshot's
texts (about 1 ms per 1,000 files), and the per-file lookups.

How: the file-level checks run for the files of a round only
(`ascribe_check::check_file`). The page-level checks use
`PageChecker::with_index`, over the snapshot's own index, and
`PageChecker::check_resolved`, for the pages that can have a diagnostic located
in a file of the round (the file, when it's a page, and the pages that include
it); their resolved forms come from a `ResolvedCache` that forgets what each
update's `Affected` lists. A round also covers what its files include and
included at the previous round, because a page that starts or stops including a
fragment changes the diagnostics located in that fragment (an `include-cycle`),
and `Affected::recheck` doesn't list the fragment.

### Completion

`cargo bench -p ascribe-lsp --bench completion` (release build) types the text
of each completion context into a page of the same generated projects
(`benches/synthetic`, shared with `keystroke`) and times `didChange` to the
completion response, with the server checking the edit in the background as it
would in an editor. 100 requests per context, median / 95th percentile; one run
on a 4-core 2.1 GHz Xeon container:

| Context | 20 pages | 300 | 1,000 | 3,000 pages |
|---|---|---|---|---|
| directive name | 0.11 / 0.15 ms | 0.12 / 0.22 ms | 0.12 / 0.19 ms | 0.12 / 0.21 ms |
| attribute values | 0.08 / 0.11 ms | 0.06 / 0.13 ms | 0.07 / 0.28 ms | 0.08 / 0.34 ms |
| phrase | 0.06 / 0.10 ms | 0.06 / 0.14 ms | 0.06 / 0.19 ms | 0.06 / 0.24 ms |
| include path | 0.35 / 0.59 ms | 0.67 / 0.87 ms | 1.4 / 2.2 ms | 3.6 / 4.7 ms |
| include id | 0.22 / 0.43 ms | 0.58 / 0.81 ms | 0.60 / 1.1 ms | 0.79 / 2.7 ms |
| link, nothing typed | 0.20 / 0.30 ms | 0.70 / 0.86 ms | 1.1 / 1.3 ms | 1.7 / 2.0 ms |
| link, page title | 0.36 / 0.50 ms | 0.90 / 1.8 ms | 2.9 / 4.7 ms | 5.9 / 8.1 ms |
| link, heading | 0.23 / 0.30 ms | 1.1 / 1.2 ms | 1.6 / 2.2 ms | 2.8 / 3.2 ms |

Every context is far under the 50 ms target at 3,000 pages (the worst 95th
percentile is 8 ms). Link and include completion scan the snapshot's pages and
headings (no cache, so it can't be stale), rank the matches, and cut the list at
100; the cost grows linearly with the project and the ranking is a sort of the
matches.

### Inventory

`cargo bench -p ascribe-lsp --bench inventory` (release build) changes a page
of the same generated projects, with a glossary term added that is in every
page's prose, and times `ascribe/inventory` after each change, with the server
checking the edit in the background. 100 requests, median / 95th percentile;
one run on a 4-core 2.1 GHz Xeon container:

| Pages | Inventory, median (p95) |
|---|---|
| 20 | 0.8 ms (1.1) |
| 100 | 1.1 ms (1.6) |
| 300 | 1.9 ms (2.3) |
| 1,000 | 5.5 ms (7.4) |
| 3,000 | 24 ms (29) |

The target is a median under 50 ms at 3,000 pages, since the views ask on
every save. It counts from the snapshot's index in one pass, with no cache to
go stale: the links and includes the project resolved, the phrases and
availability markers it indexed, each directive line, and the glossary's terms
in each page's prose. Without the glossary term, the 3,000-page median is
15 ms.

## Library choice

Chosen: **`lsp-server` with `lsp-types`** (rust-analyzer's synchronous
scaffold and the protocol types), over `tower-lsp-server`.

- The work is CPU-bound and has to be cancellable and strictly ordered: a
  computation overtaken by a newer edit must never publish, and a publish must
  be atomic with the check that it's still current. That is a lock and a
  worker thread, which `lsp-server`'s plain channels fit directly; under
  `tower-lsp-server` every handler is an async task on a runtime, and the same
  guarantees need explicit ordering around its concurrent request handling.
- `lsp-server` owns the message loop, the `initialize` handshake, and the
  shutdown protocol, and nothing else: no async runtime (`tokio`) in the
  dependency tree of the binary that also runs `check` and `build`.
- Handling a panic per request is a `catch_unwind` around a plain function call.
- `tower-lsp-server`'s advantages (typed request handlers, less boilerplate)
  matter more for a server with many small requests. More requests can be added
  as a dispatch table over the same loop; if they ever need async, the state and
  the worker in this crate don't depend on the loop.

## Testing

- `tests/all/` drives the server in-process over `Connection::memory()`, as one
  program (`tests/all.rs`; `cargo test -p ascribe-lsp --test all scenarios::`
  runs one file's tests): the
  scripted scenarios of the acceptance criteria, multi-byte positions, stale
  computations (a hook holds a computation until a newer edit lands), file
  watching, model changes, and which project a workspace folder gets.
- `crates/ascribe-cli/tests/all/lsp_parity.rs` starts the real `ascribe lsp` binary
  over stdio and compares its published diagnostics with
  `ascribe check --build <name> --format json`, for every build of
  `examples/quill` and fixture projects with known problems.
- `tests/all/navigation.rs` scripts every navigation feature over a copy of
  `examples/quill` (with a features registry added): each completion context,
  hover, definition, document links, CodeLens and its command, inlay hints,
  answers after an edit or a deletion, the negotiated encodings, files outside
  the project, and a request of every kind at every position of several files
  (including multi-byte text) that must never fail.
- `benches/keystroke.rs` records the time from an edit to its diagnostics;
  `benches/completion.rs` the time from an edit to its completion.
