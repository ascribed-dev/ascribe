# Conformance suite

This directory holds Ascribe's conformance cases and the harness that runs them. A case pairs Ascribe source with what a conforming processor must produce from it: the parsed structure, the diagnostics, and per-build results. The cases are written by hand from [SPEC.md](../../SPEC.md), not from any implementation, so they're an independent check.

This file documents the case format and the harness. Each crate connects through an adapter and removes the skip entries for what it handles.

## Running

```sh
# Everything: prints each failure and skip, then the totals.
cargo test -p tessera-conformance --test conformance

# Only cases with a tag (repeatable), or only cases whose id contains some text.
cargo test -p tessera-conformance --test conformance -- --tag structure
cargo test -p tessera-conformance --test conformance -- samples/appendix-b

# The harness's own tests.
cargo test -p tessera-conformance --lib --test harness
```

`cargo test --workspace` runs all of these. The run fails if any case fails or `SKIPS.toml` is stale.

Every case runs through the runner: each area tag has an adapter (`tests/adapters/`), and `SKIPS.toml` is empty, so nothing is skipped. (No case carries the `output` tag; the emitters' outputs are tested in `tessera-emit`. A case that uses it needs an adapter first.) The `page-check` adapter reports a build's diagnostics from `tessera_check::check_pages`, the entry point `ascribe check`, `ascribe build`, and the language server share.

## Layout

```
tests/conformance/
  README.md          this file
  SKIPS.toml         skipped tags and cases, each with a reason
  diagnostics.toml   the diagnostics registry
  _model/            the shared fixture model, ascribe.toml
  cases/             the cases, grouped by area: cases/<area>/<case>/
  snapshots/         insta snapshots of expected outputs
  src/               the harness library (crate tessera-conformance)
  tests/
    conformance.rs   the runner
    adapters/        adapters connecting the harness to the Ascribe crates
    harness.rs       tests of the harness itself, with fixtures in fixtures/
```

## The suite

The cases were written from SPEC.md, not from any implementation. They're grouped by what they check:

| Directory | Checks |
|---|---|
| `recognition/`, `attributes/`, `primary/` | Which lines are directives, attribute blocks and their diagnostics, identifier and text primaries (SPEC §3.1–§3.4) |
| `forms/`, `groups/`, `titles/`, `binding/`, `lists/`, `nesting/` | Container and line forms, groups and the arm rule, title lines, binding, lists and blockquotes, nesting (§3.5–§3.10) |
| `directives/<name>/` | Each built-in directive: `id`, `include`, `variant`, `available`, `note`, `steps`, `details` (§4) |
| `widgets/` | Project widgets (§6) |
| `phrases/`, `links/`, `images/`, `headings/`, `glossary/` | Inline constructs, source ids and page ids (§5) |
| `frontmatter/`, `model/` | Frontmatter, content types, fragments, name roles (§2, §7.2) |
| `format/` | Canonical form (§8.3): each rule, what the formatter leaves alone, and constructs with errors |
| `builds/selection/`, `builds/filter/`, `builds/assets/`, `builds/pages/` | Resolution and build modes, availability filtering, assets, which files are pages (§9.2–§9.4) |
| `projects/quill/` | The whole Quill project of `examples/quill`, with no diagnostics under any build |
| `samples/` | The two worked examples |

Every row of SPEC §8.2 has at least one case that expects it; `tests/suite.rs` keeps that true. [`INTERPRETATIONS.md`](INTERPRETATIONS.md) lists the readings of the spec the cases lock in, and what the suite doesn't cover.

Conventions the cases follow:

- **A page needs a title.** The shared model's default type requires `title`, so a single-file case that expects diagnostics starts with a three-line frontmatter block. Diagnostic lines count it. A case that expects only an outline has none, and neither does a fragment.
- **A case checks one kind of thing.** Recognition and structure are separate from diagnostics (`parser` and `structure` cases carry an outline; `check` cases carry diagnostics), so a case runs when its own adapter exists. A case carries several tags only when it needs several adapters.
- **Page-level diagnostics need the `page-check` tag**, and top-level (file-level) diagnostics need `check`.

## Resolved outlines

A build's page `outline` is the page after resolution steps 1 to 4 (SPEC §9.2): includes replaced, availability resolved, the build's modes applied, and phrases substituted. Its text is still the page's source text, so:

- links keep their file-path destinations and empty link text, heading ids aren't shown, and glossary terms aren't linked (steps 5 to 7); those show up as diagnostics and in the emitters' outputs;
- an `@available` directive that survives a filter stays in the outline as the annotation, with its binding; content a filter removes takes its directive with it, and a section is its heading and everything up to the next heading of the same or a higher level, subsections included;
- a group that a selection reduces to several arms stays a group with the arms' own attributes, and one arm becomes that arm's blocks;
- an escape stays as written (`\{product}`).

## The shared model and the content root

`_model/ascribe.toml` sets `content-root = "files"`, the content root of every project case. A single-file case has `input.md` in its directory, so its content root is the case directory: adapters use `Case::content_root()` as the content root whatever the model says, and the case directory as the project root (the directory a case's `ascribe.toml` is in, when it has one). Files a case needs beside the project or outside it (`../shared/logo.png`, `../../shared.png`) sit next to `files/` or next to the case directory.

A diagnostic in `ascribe.toml` (a loader rule) is expected in a single-file case, with `file: ascribe.toml`, since the case directory is the content root there.

## Cases

A case is a directory under `cases/` that contains `expect.yaml`. Its **id** is its path relative to `cases/`, with `/` separators, for example `structure/groups/one-arm`. Group cases in directories by area; the grouping is for people, and tags drive the runner.

Discovery rules:

- A directory with `expect.yaml` is a case. The harness doesn't look for cases inside it.
- Directories whose names start with `_` or `.` are skipped, so shared fixtures can sit next to cases.
- A directory with `input.md` or `files/` but no `expect.yaml` is reported as a broken case, so a forgotten file never silently drops a case.

There are two kinds of case:

| Kind | Contains | Content root | Use for |
|---|---|---|---|
| Single file | `input.md` | the case directory; `input.md` is the only source file | Everything about one file: recognition, structure, file-level diagnostics |
| Project | `files/`, a tree of source files | `files/` | Fragments, includes, links, assets, page-level diagnostics, builds |

A case has one or the other, never both. Paths in `expect.yaml` (diagnostic files, page names, assets) are relative to the content root and use `/`. Single-file cases refer to their file as `input.md`.

### The content model

A case may have its own `ascribe.toml` in the case directory. Otherwise it uses the shared fixture model, `tests/conformance/_model/ascribe.toml`, written from the Quill model. Prefer the shared model; give a case its own only when it tests the content model itself or needs declarations the shared model shouldn't have.

### Line endings

`.gitattributes` checks every file out with LF line endings on every platform, because inputs and expected outputs are compared byte for byte. A case that needs CRLF input marks its file `-text` in `.gitattributes`.

## `expect.yaml`

```yaml
description: A trailing colon opens a container.   # optional; say what the case checks
spec: ["3.5"]                                     # optional; SPEC.md sections covered
tags: [structure]                                 # required; at least one area tag

outline: [...]         # single-file cases only; the expected parse (see Outline)
formatted: formatted.md  # single-file cases only; what formatting input.md gives (see Formatting)
diagnostics: [...]     # expected file-level diagnostics (see Diagnostics)
builds: {...}          # per-build expectations (see Builds)
```

A case must expect at least one of `outline`, `diagnostics`, `builds`, or `formatted`. Unknown keys are errors, so a typo can't silently turn off a check. Omitting `outline`, `diagnostics`, or a build means "not checked"; an empty list (`diagnostics: []`) means "expect none".

### Tags

Tags route a case to adapters and let a runner select cases. **Area tags** name the part of Ascribe a case exercises. A case can carry several; it runs only when every one of its area tags is handled by an adapter.

| Tag | Covers | Implemented in |
|---|---|---|
| `parser` | Ascribe-line recognition, directive heads, attributes, primaries (§3.1–§3.4) | 05 |
| `structure` | Forms, containers, groups, titles, binding, lists and blockquotes, nesting (§3.5–§3.10, §4) | 06 |
| `inline` | Phrases, escapes, image attribute blocks (§2.3, §5.1, §5.3) | 07 |
| `model` | Loading and validating `ascribe.toml` (§7) | 08 |
| `slug` | Heading slugs and source ids (§5.5) | 09 |
| `check` | File-level diagnostics (§8.1, §8.2) | 10 |
| `include` | Includes and the source index (§4.2) | 11 |
| `resolve` | Resolution passes and build modes (§9.2, §9.3) | 12 |
| `page-check` | Page-level diagnostics (§8.1, §8.2) | 14 |
| `output` | Plain, site, and JSON output (§9.4) | 18, 20 |
| `format` | Canonical form (§8.3) | 23 |

A new area tag needs an entry in `SKIPS.toml` (or an adapter) before any case uses it.

## Outline

The `outline` describes a parsed file abstractly: its blocks in order, each with a kind and a few fields. It doesn't mention node types, spans, or anything else internal to an implementation. Test authors write it by hand, and adapters produce the same shape from the real parser.

An outline is a YAML list of blocks. Each block is a mapping with exactly one **kind key**, whose value is the block's main value, plus any **fields**:

```yaml
- heading: 2                 # kind key `heading`, main value: the level
  text: Install the agent    # a field
- paragraph: Some text.      # kind key `paragraph`, main value: the text
- thematic-break             # a kind with no value can be written bare
```

### What's in an outline

- Every block CommonMark produces, in source order, nested the way CommonMark nests them, plus Ascribe's directives and groups.
- Directive lines, end lines, and title lines are not blocks of their own. A directive line becomes a `directive` block (or opens a group arm); a title line becomes the `title` field of the directive below it; an end line only closes a container.
- Not included: frontmatter, blank lines, and link reference definitions.
- A line that looks like a directive but isn't recognized (an unknown keyword, `\@`, `@` inside code) stays whatever CommonMark makes it, usually paragraph text.

### Block kinds

| Kind | Main value | Fields |
|---|---|---|
| `heading` | level, 1–6 (required) | `text` |
| `paragraph` | text (optional) | — |
| `image` | source, as written (required) | `alt`, `title`, `attributes` |
| `code` | content (optional) | `info`, `fenced` |
| `blockquote` | list of child blocks | — |
| `list` | `ordered` or `bullet` (required) | `start`, `children` (the `item`s) |
| `item` | list of child blocks | — |
| `thematic-break` | none | — |
| `html` | source text (optional) | — |
| `table` | none (contents aren't described) | — |
| `directive` | name, without `@` (required) | `form`, `attributes`, `primary`, `title`, `binding`, `children` |
| `group` | name of the groupable directive (required) | `arms` |

`image` is a paragraph that consists of a single image and, optionally, its attribute block (§5.3). An image inside other text is part of the paragraph's text.

### Directives

```yaml
- directive: note
  form: container            # `line` (default) or `container`
  attributes: {type: warning}
  primary: Back up your database first.
  title: Before you upgrade
  binding: following-block   # line form only
  children: [...]            # container form only
```

- `form` is `container` when the directive line ends in an empty primary after its colon (§3.5), and `line` otherwise.
- `attributes` maps keys to values (see [Attribute values](#attribute-values)).
- `primary` is the primary, if any. A text primary that continues onto following lines (§3.4) is written as one string.
- `title` is the text of the title line above the directive, without the `.`.
- `binding` says what a line-form directive applies to (§3.8):

  | Value | Meaning |
  |---|---|
  | `self` | its own primary, or nothing (it stands alone, like `@include`) |
  | `heading` | the heading at the start of its section |
  | `following-block` | the next block in the same list of blocks, which is the next entry in the outline |
  | `none` | nothing, because binding failed (an error case) |

  A container has no `binding`. A following-block directive and the block it binds stay siblings in the outline, as they are in the source.
- `children` holds a container's blocks. A line-form directive has none.

### Groups

A run of openers of one groupable directive (§3.6) is a single `group` block. Each opener starts an arm. The group's end line isn't written.

```yaml
- group: variant
  arms:
    - attributes: {pm: npm}
      children:
        - code: "npm install -g @quill/agent\n"
          info: shell
    - title: Using other images      # a labeled arm
      children: [...]
```

Each arm has `attributes`, `title`, and `children`, with the same meaning as a directive's.

### Attribute values

The outline records an attribute's meaning, not its spelling:

- A token or quoted string is a string, unescaped: `label="Using other images"` is `label: Using other images`.
- A value set is a list: `platform=cloud|on-prem` is `platform: [cloud, on-prem]`.
- YAML booleans and integers are accepted and read as their text: `heading: false` is the string `false`.
- YAML floats are rejected, because YAML reads `3.10` as `3.1`. Quote version-like values: `since: "3.4"`.
- `{}` and no attribute block are the same: no attributes.

### Text

- `text`, `primary`, `title`, and `alt` are the **raw source text** of the inline content: markdown syntax, phrases (`{product}`), and escapes are left as written, not rendered or substituted. Container prefixes (list indentation, `>` markers) are removed.
- They're compared after collapsing each run of whitespace, including line breaks, to one space and trimming the ends. Hard-wrapped text can be written on one line, or with YAML's folded style (`>-`).
- A code block's content is its literal text, compared exactly except for trailing newlines. YAML's literal style (`|`) is the easiest way to write it.
- Headings' `text` excludes the `#` markers (or the setext underline).

### Structural and content fields

Every field is either structural or content, and that decides what omitting it means.

- **Structural fields are always compared.** When the expectation omits one, it's compared against its default: `form: line`, no `attributes`, no `primary`, no `title`, no `children`, and a list's `ordered`/`bullet` must always be written. So writing `- directive: note` asserts that the note has no attributes, primary, or title.
- **Content fields are compared only when written**: `text` (and a paragraph's, code block's, or HTML block's main value), `binding`, `info`, `fenced`, `start`, and `alt`. Leave them out when a case isn't about them, so it doesn't break on unrelated details.

Adapters fill in every field they can; the omission rules apply only to expectations.

### Error cases

When a case expects an error, the outline is often a matter of error recovery, which the spec doesn't define. For those cases, expect `diagnostics` and leave out `outline`, unless the spec says what the source means (for example, that an unknown keyword stays paragraph text).

## Diagnostics

```yaml
diagnostics:
  - slug: container-unclosed    # illustrative; real slugs come from diagnostics.toml
    line: 12                    # 1-based
    column: 1                   # optional, 1-based
    file: guides/setup.md       # required in project cases
```

- `slug` names an entry of the diagnostics registry, `tests/conformance/diagnostics.toml` (its header documents the format). Cases refer to diagnostics by slug, never by code or message. The runner checks every expected slug before the case runs, even when the case is skipped: the slug must be registered; a `file`-level slug goes in the top-level `diagnostics` and and a `page`-level slug under a build.
- `line` is 1-based and counts every line of the file, including frontmatter.
- `column` is 1-based and counts Unicode scalar values (characters), not bytes or UTF-16 units. It's compared only when written.
- `file` is relative to the content root. It defaults to `input.md` in single-file cases.
- The expected list must match the reported diagnostics exactly, as a multiset: every expected diagnostic must be reported, and every reported diagnostic must be expected. Order doesn't matter.
- Top-level `diagnostics` holds **file-level** diagnostics (§8.1) from every source file in the case. **Page-level** diagnostics depend on a build, so they go under that build (see below).

## Builds

`builds` maps build names, as declared in the case's content model, to what each build must produce. Builds work for single-file cases too, where the one page is `input.md`.

```yaml
builds:
  cloud-pdf:
    pages:                     # every page the build publishes, exactly
      index.md:
        outline: [...]         # the resolved page
        outputs:
          plain: snapshot
          site: {file: expected/cloud-pdf/index.site.md}
      guides/keys.md:          # published; nothing else checked
    assets: [images/settings.png]
    diagnostics: []            # page-level diagnostics for this build
```

Every key is optional; an omitted key isn't checked.

- **`pages`** lists every page the build publishes, by source path. The set must match exactly, so a page a build drops (§9.3) is checked by leaving it out. Fragments are never pages.
- A page's **`outline`** describes the **resolved** page (§9.2): includes replaced by their content, the build's variant and availability modes applied (a group reduced to one arm becomes that arm's blocks), and phrases substituted in text. It uses the same schema as the file outline.
- **`assets`** lists every local file the build copies (§9.4, Assets), by source path. The set must match exactly.
- **`diagnostics`** holds this build's page-level diagnostics, in the format above. `file` is required in project cases.
- **`outputs`** maps an emitter (`plain`, `site`, or `json`) to where its expected output is recorded:
  - `snapshot`: an [insta](https://insta.rs) snapshot in `tests/conformance/snapshots/`, named `<case>__<build>__<page>__<emitter>` with `/` and `.` replaced by `_`. New or changed snapshots show up as `.snap.new` files; review them with `cargo insta review` and never accept them blindly.
  - `{file: <path>}`: a file relative to the case directory, compared byte for byte.

## Formatting

A case tagged `format` checks canonical form (SPEC §8.3). It is a single-file case whose `expect.yaml` names, in `formatted`, a file in the case directory: what formatting `input.md` must give, byte for byte. `formatted: input.md` says the input is already canonical.

```yaml
tags: [format]
spec: ["8.3"]
formatted: formatted.md
```

The runner formats `input.md` through the adapter (`ConformanceAdapter::format`, under the case's content model), compares the result with the file, and formats the result again: canonical form is a fixed point, so the second pass must change nothing. A case names its rule (one rule per case where it can) and tests the rule's edges next to it. `format/markdown-untouched` shows ordinary markdown staying byte for byte, and the `format/errors-*` cases show constructs with errors left as written.

## Skips

`SKIPS.toml` lists everything the runner deliberately doesn't run. Every entry has a reason, and the runner prints every skip.

```toml
[[skip]]
tag = "structure"
reason = "Adapter for tag `structure` not yet implemented."

[[skip]]
case = "check/attributes/bare-key"
reason = "The bare-key diagnostic isn't implemented yet."
checks = ["diagnostics"]
```

- Each entry names exactly one `tag` or one `case` (by id), and has a non-empty `reason`.
- Without `checks`, the entry skips whole cases. With `checks` (any of `outline`, `diagnostics`, `builds`, `formatted`), only those checks are skipped and the rest of the case runs.

For each case, the runner decides:

1. A full skip entry for the case skips it.
2. Otherwise, each of its area tags must be handled by an adapter or have a full skip entry. A tag with neither **fails the case**, even if another of its tags is skipped.
3. If any area tag is skipped, the case is skipped, with every applicable reason.
4. Otherwise, the case runs, minus any checks named by partial skip entries for the case or its tags.

The run also fails when `SKIPS.toml` is stale: a full skip for a tag that an adapter now handles, or a skip for a case that doesn't exist. So when an adapter is registered, it must remove its tag's entry, and its cases start running.

## Adapters

An adapter connects the harness to an implementation. The harness depends on no Ascribe crate; adapters live in `tests/adapters/`, and `tests/adapters/mod.rs` registers them.

```rust
pub trait ConformanceAdapter {
    fn name(&self) -> &str;
    fn handles_tag(&self, tag: &str) -> bool;
    fn outline(&self, case: &Case) -> AdapterResult<Outline> { Ok(None) }
    fn diagnostics(&self, case: &Case) -> AdapterResult<Vec<Diagnostic>> { Ok(None) }
    fn format(&self, case: &Case, source: &str) -> AdapterResult<String> { Ok(None) }
    fn build(&self, case: &Case, build: &str) -> AdapterResult<BuildResult> { Ok(None) }
}
```

- `handles_tag` decides which cases reach the adapter. Handling a tag means its cases run.
- Each method returns `Ok(Some(result))`, `Ok(None)` when this adapter doesn't produce that result, or `Err` when it can't process the case. For each check, the runner asks the adapters that handle any of the case's area tags, in registration order, and uses the first `Some`. An expectation that no adapter produces fails the case, unless a skip entry names that check.
- `Case` gives an adapter the case's `kind`, `content_root()`, `input()` (single-file cases), `source_files()`, and `model` (the `ascribe.toml` to load).
- `diagnostics` returns file-level diagnostics for every source file, with `file` relative to the content root. `build` returns the published pages, copied assets, page-level diagnostics, and, per page, the resolved outline and any emitted outputs.
- `format` returns `source` in canonical form under the case's content model (see Formatting).
- A panic inside an adapter fails that case; the run continues.

To connect a crate: add the crate as a dev-dependency of `tessera-conformance`, implement the trait in a module under `tests/adapters/`, register it, remove the tag's entry from `SKIPS.toml`, and make the cases pass.

## Worked example: the Appendix B page

The case `cases/samples/appendix-b` is the complete page from SPEC Appendix B, as `input.md`. It's a single-file case that expects only an outline: the page includes and links to files a single-file case doesn't have, so its file-level diagnostics wouldn't be clean. (`examples/quill` is the full project.)

Its `expect.yaml`:

```yaml
description: >-
  The SPEC Appendix B page parses to the expected outline: title lines,
  heading-bound and following-block directives, a variant group inside a list
  item, a dimensional group, and a container note.
spec: ["B", "3.5", "3.6", "3.7", "3.8", "3.9"]
tags: [structure]

outline:
  - paragraph: >-
      The {product} agent watches your docs repository and syncs changes to
      {product}. This page covers installing the agent with a package manager,
      configuring it, and connecting it to {cloud} or a self-managed server. If
      you only want to try {product}, see [Try {product} in the
      browser](quickstart.md#try-in-browser).

  - directive: note
    attributes: {type: tip}
    title: Try it without installing
    binding: following-block
  - paragraph: "You can run {product} in the browser at play.quill.dev with no local setup."

  - heading: 2
    text: Prerequisites
  - directive: id
    primary: prerequisites
    binding: heading
  - directive: include
    primary: _fragments/prerequisites.md
    binding: self

  - heading: 2
    text: Install the agent
  - directive: id
    primary: install-agent
    binding: heading
  - directive: steps
    binding: following-block
  - list: ordered
    start: 1
    children:
      - item:
          - paragraph: "Install the agent package:"
          - group: variant
            arms:
              - attributes: {pm: npm}
                children:
                  - code: "npm install -g @quill/agent\n"
                    info: shell
              - attributes: {pm: pnpm}
                children:
                  - code: "pnpm add -g @quill/agent\n"
                    info: shell
              - attributes: {pm: yarn}
                children:
                  - code: "yarn global add @quill/agent\n"
                    info: shell
      - item:
          - paragraph: "Verify the install:"
          - code: "quill --version\n"
            info: shell
          - paragraph: "The command prints the installed version, {version}."
          - directive: note
            binding: following-block
          - paragraph: "The agent needs write access to your repository's `.quill/` directory."
      - item:
          - paragraph: "Create `quill.yaml` at the root of your repository:"
          - code: |
              agent:
                version: {version}
                watch: docs/
            info: yaml phrases=true

  - heading: 2
    text: "Connect to {product}"
  - directive: id
    primary: connect
    binding: heading
  - group: variant
    arms:
      - attributes: {deployment: cloud}
        children:
          - paragraph: "Sign in to {cloud} and copy an API key from **Settings → Keys**, then add it to `quill.yaml`:"
          - code: |
              cloud:
                api_key: ${QUILL_KEY}
            info: yaml
      - attributes: {deployment: self-managed}
        children:
          - paragraph: "Point the agent at your server. Self-managed servers must run {product} Server 3.3 or later."
          - code: |
              server:
                url: https://quill.internal.example.com
            info: yaml

  - heading: 2
    text: Streaming sync
  - directive: id
    primary: streaming-sync
    binding: heading
  - directive: available
    primary: cloud, self-managed preview 3.4
    binding: heading
  - paragraph: "Streaming sync pushes changes as you save, instead of on each commit."
  - paragraph: "{cloud}'s streaming sync is enabled by default for new {cloud}-hosted workspaces."
  - paragraph: "For event formats, see the [streaming API reference]({api}streaming)."

  - heading: 2
    text: Troubleshooting
  - directive: id
    primary: troubleshooting
    binding: heading
  - directive: note
    form: container
    attributes: {type: warning}
    children:
      - paragraph: "If the agent exits immediately, check the log at `~/.quill/agent.log`."
      - paragraph: >-
          A common cause is an expired API key. Generate a new key, then restart
          the agent. See [](keys.md#rotate-keys).
```

How the page maps to the outline:

- **Frontmatter** isn't part of the outline.
- **Text is raw source.** `{product}` and the link markup stay as written; this is the file's parse, not its resolved output. The first paragraph is hard to fit on one line, so it uses YAML's folded style, and whitespace is normalized before comparison.
- **`.Try it without installing`** isn't a block. It's the `title` of the `@note` below it. That note has no colon and no primary, so it's in line form and binds the paragraph it touches: `binding: following-block`, with the paragraph as the next entry.
- **`@id`** under each heading is heading-bound: `binding: heading`. So is `@available` under "Streaming sync": it's at the top of its section (§4.4), so it applies to the section rather than to the paragraph below it. The blank line between it and the paragraph doesn't change that.
- **`@include`** stands alone: `binding: self`, with its path as the `primary`. `{heading=false}` is `heading: false`, which the harness reads as the string `false`.
- **`@steps`** binds the ordered list after it. The list and its three items are ordinary CommonMark blocks; the directive lines inside the items belong to the items because they're indented to the items' content column (§3.9).
- **The `pm` group** is inside the first item. Its three openers form one group with three arms, and the single `@end` closes the group, so the end line doesn't appear.
- **The `@note` in the second item** is a line-form note binding the paragraph below it, inside the same item.
- **`yaml phrases=true`** is the fence's whole info string. `{version}` in the code is literal in the file outline; whether it's substituted is a matter for the resolved outline and outputs.
- **The `deployment` group** is at the top level, with two dimensional arms, each holding a paragraph and a code block.
- **The troubleshooting note** ends its line with a colon, so it's `form: container`, holding both paragraphs until `@end`.
- **Fields left out on purpose.** The code blocks omit `fenced`, a content field the case isn't about. The directives with no attributes, primary, or title omit those structural fields, which asserts that they have none.

The second sample, `cases/samples/include-and-selection`, is a small project case showing `builds`: a page includes a fragment and has a `deployment` group, and the expected resolved outlines show the fragment's content in place of the include, and the group reduced to its cloud arm in a cloud build.

## Writing good cases

- Prefer many small cases to a few large ones. A failing small case points straight at the broken rule.
- Put each SPEC.md example in at least one case, and record the section in `spec`.
- Expect only what the case is about. Leave out content fields that don't matter to it.
- For every rule that can be broken, write a passing case and a failing one.
- Never guess where the spec is silent. Raise it so the spec can decide, and leave the case out until it does.
