# Proposal: an `ascribe.toml` that says only what's the project's own

A proposal, not a plan. It says what would change and what it would look like, so the ideas can be judged before anything is designed in detail. It picks up section 20 of the [brainstorm](../brainstorm.md#20-ascribetoml-in-more-than-one-file), and answers it differently: the file is long because it repeats itself, not because it holds too much.

## The goal

A project's `ascribe.toml` holds what's true of that project and nothing Ascribe could work out: no fact written twice, no setting that only restates what every project wants, and no table copied from another project's file. A small docs site's model should read in one sitting, and a second project beside it should say only how it differs.

Comments aren't the problem. Ascribe's own `docs/ascribe.toml` is a teaching surface, and its comments stay.

## What exists

On 21 October 2026, `docs/ascribe.toml` is 218 lines: 82 of comments, 30 blank, and 106 of settings. `site/home/ascribe.toml`, the landing site's, is 116, of which about 40 are copies of the docs' file that `scripts/docs-site/facts.test.ts` keeps in step.

The 106 lines of settings, sorted by why they're there:

| Lines | What | Why it's written |
|---|---|---|
| 18 | `title` and `description` in each of six types' `frontmatter` | SPEC §7.2 says every type requires a string `title`, yet a type must declare it; `description` needs its role named on each type |
| 10 | `template` and `guide` on five types | `ascribe templates eject` writes them, and `templates/templates.lock` records the same files by the same type name |
| 4 | `[dimensions.release]`, one value, `ascribe` | A release target has to be a dimension's value before `[releases.ascribe]` can name it |
| 3 | `[features.next]` | The planned line, written a second time so pages can say `next`; `ascribe versions release` rewrites it each release |
| 13 | `output-dir`, `[consumer] profile` and `trailing-slash`, `[builds.site]`, `[editor]`, `[glossary] match`, `[crawlers]` | Each is already the default: a project with no build gets `site` with these settings, as the editor's build, and a crawler purpose left out writes what `allow` writes |
| 1 | `ignore = ["**/node_modules/**"]` | Every source that includes a JavaScript project's folders needs it |
| 57 | The rest | The project's own: its types' purposes, its dimensions, phrases, sources, site, checks, and retired permalinks |

The second project repeats a third kind of thing: the release dimension, the releases, `[features.next]`, the `version` phrase, `[sources.code]`, and three of `[consumer]`'s keys. Because of the copy, a release runs `ascribe versions release` twice, once for each project (RELEASING.md).

## How opinionated Ascribe can be

A default is a decision made for every project that doesn't make its own. Ascribe can take one when all of these hold:

1. **The common case is overwhelming.** Nearly every project would write the same value, and the few that don't know they're different.
2. **A wrong default is visible and costs one line.** The project sees the effective value somewhere it already looks, and changes it with one key.
3. **It publishes nothing the author didn't mean to.** A default may change how something is published; it may not make public what wasn't, or send anything anywhere.
4. **It doesn't hide a decision a team should make on purpose.** A check's level, a build's audience, and what a site's address is are the team's.

Where the effective value shows is what makes defaults safe to have: `ascribe model` prints it, Project Settings lists every setting the file leaves out with its value, and the reference's tables, generated from the registry (`crates/ascribe-model/settings.toml`), give each default. Before 1.0, breaking changes are acceptable: a default can change, and a key can go, without a migration path.

Judged against those rules:

| Candidate | Default | Holds? |
|---|---|---|
| A type's `title` | Every type has a required string `title`, as SPEC already requires; a type declares it only to change it, such as `inline = "code"`. The loader keeps the rule, and `model-type-title` goes | Yes: the contract already demands it |
| A field named `description` | A string field named `description` has the `description` role, unless another field of the type has it | Yes: the name says what it is, and a type that means something else gives the role elsewhere |
| Frontmatter every type shares | A top-level `[frontmatter]` table, whose fields every type has, and which a type's own adds to | Yes: one place for what's true of every page |
| A type's template and guides | From `templates.lock`, by the type's name, when the type doesn't set them | Yes: the lock is the one home, and eject already writes it |
| A release target's dimension | `[releases.<target>]` declares `<target>` as a value of a dimension named `release`, unless a declared dimension already has it | Yes: the dimension adds nothing a project chose |
| A word for the planned line | `planned` in a spec's version place means the target's planned line, so a page can mark work not released yet without a `[features]` entry | Yes |
| A source's `ignore` | `**/node_modules/**` and `.git/` are always ignored, before the source's own `ignore` | Yes: no project wants either |
| `[consumer] agents` | `true` when `[consumer] site` is set, `false` when it isn't: `llms.txt` and each page's Markdown are published with a site that has an address | Yes, by rule 3: the Markdown is the page's own text, already public; a project that doesn't want it sets `false`. Unconditional, it would break rule 2: SPEC §9.4 needs the site's origin for the agents' outputs, so every project without `site` would fail `model-consumer-agents-site` |
| `[consumer] permalink-path` | Stays `go/` under `base-path` | No: the site's root may not be the project's to write to |
| `prose-not-checked` | Stays advice, but is said only where the project asked for prose checks: `[checks.vale] in-check = true`, or `--vale` | Yes: a machine without Vale isn't told so unless the project asked for Vale everywhere |
| `spec` | Stays required | No: it's what lets a later binary read an older project correctly |
| A check's level | Stays each check's own | No: rule 4 |
| `example-stale` as an error | Stays advice | No: rule 4; the docs' `error` is a choice a team makes |

## The pieces

### 1. Frontmatter every type shares

```toml
[frontmatter]
title = { type = "string", inline = "code" }
description = "string"

[types.how-to]
purpose = "Doing one thing with Ascribe once you know what it is."

[types.index]
```

- **`[frontmatter]`** declares fields every type has. A type's own `frontmatter` adds fields; a type that declares a field `[frontmatter]` has replaces it whole, for that type, as an index page's optional `description` would. Fields aren't merged key by key: a field reads as one declaration, wherever it is, and the editor's hover and `ascribe model` say which table declared it, so a type's copy that drifts from the shared one is seen.
- **`title`** is implicit: every type has a required string `title`, declared or not. Declaring it changes only how it's read (`inline`).
- **`description`** takes the `description` role by its name. So does `review-date`, a date field, and `owner`, a string or enum field: the three roles SPEC defines, each taken by the field of its name unless the type gives the role to another field.
- **A type with no keys** is `[types.index]`: a type a page can name, with the shared frontmatter.
- **The implicit `page` type** of a model with no types gains the shared frontmatter too.
- **A type's field that equals the shared one,** key for key, is advice to delete it: a new check, `frontmatter-field-repeated`, with a fix that removes it. Without it, the drift the hover shows is never cleaned up.

The docs' 38 lines of types become 14, and each type's table says what's its own: its purpose and its template.

### 2. A type's template from the lock

`templates/templates.lock` records each ejected template's files under the type's name. A type with no `template` or `guide` takes them from the lock when the lock has the type's name; a type that sets either keeps what it sets. `ascribe templates eject` stops writing the two keys, and the editor's hover on a type shows where its template comes from.

- **It amends the page templates plan's [decisions 7 and 14](https://github.com/ascribed-dev/ascribe/blob/180eee83/project-docs/page-templates/README.md#decisions),** which have a type name its template and its guide. `template` and `guide` stay on a type, for a project without the lock and for a template written by hand; the lock becomes where an ejected template's are found.
- **The model's validity then depends on a file `eject` owns.** The lock is read through `FileSystem`, as the template files are, and a lock that names a missing file is the type's error, `model-type-template`, as a wrong `template` is today.

### 3. Releases declare their targets

```toml
[releases.ascribe]
live = ["0.4.1"]
planned = ["0.4.2"]
label = "Ascribe"
```

`[releases.<target>]` declares `<target>` as a value of an implicit dimension, unless a declared dimension already has that value, as a project with `cloud` and `self-managed` targets in its own `deployment` dimension declares them today. The docs' `[dimensions.release]` goes.

- **The implicit dimension is named `release`,** since `@variant {release=…}`, a switcher's `sync`, `dimension:` targets in `ascribe refs`, `[listing]`, and the elements contract's `dimension` attribute all name a dimension. The name is reserved while the implicit dimension exists: a project that declares its own `release` dimension and leaves a target to the implicit one gets an error naming both.
- **Two labels, for two things:** `[releases] label` is the dimension's, `Release` unless set; `[releases.<target>] label` is the target's, its name unless set. The docs set the second, `Ascribe`.

**A word for the planned line:** `planned` is a version word, written where a version goes in an availability spec (SPEC §4.4): `@available: ascribe planned`, or `cloud, self-managed preview planned`. It means the target's planned line, so it reads as the planned line does, and a target with no planned line, or more than one, is an error at the mark. It isn't a feature key: a feature key is one token that stands for one spec, so a bare `planned` couldn't say which target's line it means once a project has two. As a word in the version's place, each target says its own, and states compose with it. `ascribe versions release` rewrites a target's `planned` marks to the released version, as it rewrites `next` today, and has no feature to move. The docs' pages write `next` today; the pull request that builds this rewrites them to `ascribe planned`, and the docs' `[features.next]` goes.

### 4. Defaults that change

- **A source ignores `**/node_modules/**` and `.git/`,** whatever its `ignore` says.
- **`[consumer] agents` is `true`.**
- **`prose-not-checked`** is said only where the project asked for prose checks everywhere.

Each is a row in the registry, so the reference's tables, Project Settings, and the loader change together. So are the keys the other pieces add: piece 1's `[frontmatter]`, piece 3's two `label`s, and piece 5's `[from]`, or Project Settings won't show them.

### 5. One project takes tables from another

The landing site beside its docs is the usual shape of a product's site, and a second project's model is mostly the first's. A project names what it takes:

```toml
[from."../../docs/ascribe.toml"]
take = ["releases", "dimensions.pm", "phrases.version", "sources.code", "consumer.site", "consumer.trailing-slash", "consumer.permalink-path"]
```

- **What's taken is read from the other file,** as if written here. Taking a table takes all of it; taking a key takes that key. The other project's own `[from]` isn't followed: one step only.
- **A key taken and also written here is an error,** not an override: a reader of either file must be able to tell where a value comes from.
- **Edits go to the file that holds the key.** `ascribe versions release` run in the docs moves the lines for both projects; the language server's hover on a taken key says where it is, and Go to Definition opens it. Every command that edits `ascribe.toml` in place learns which file holds what it edits, which is the one place this piece costs.
- **The path leaves the project's folder as a source's `path` may** (SPEC §7.3): it's relative to the folder `ascribe.toml` is in, may go above it, and MUST be inside the same version-control repository when there is one. The file is read through `FileSystem` as a declared file outside the content root, as a source's files are, not under the content root's boundary, which refuses it. The language server watches it as it watches `ascribe.toml`.

`site/home/ascribe.toml` loses its 40 lines of copies, the test that keeps them in step, and RELEASING.md's second `versions release`.

## What it would look like

The docs' model with every piece, settings only:

```toml
spec = "0.1"

[project]
content-root = "content"

[frontmatter]
title = { type = "string", inline = "code" }
description = "string"

[types.index]
[types.reference]
purpose = "Looking up a command, a key of ascribe.toml, a directive, a diagnostic, the editor's features and settings, or a contract's rules."
[types.how-to]
purpose = "…"
[types.tutorial]
purpose = "…"
[types.concept]
purpose = "…"
[types.troubleshooting]
purpose = "…"

[dimensions.pm]
label = "Package manager"
values = ["npm", "pnpm"]

[releases]
planned-label = "in main, not yet released"

[releases.ascribe]
label = "Ascribe"
live = ["0.4.1"]
planned = ["0.4.2"]
archived = { "0.3.1" = {}, "0.3.2" = {}, "0.4.0" = {} }

[phrases]
version = { line = "ascribe" }
node = "24"
glibc = "2.28"
vscode = "1.138"
repo = "https://github.com/ascribed-dev/ascribe"

[images.attributes]
scheme = "enum(light, dark)?"

[glossary]
pages = ["glossary.md"]

[sources.code]
path = ".."
include = [".github/workflows/**", "crates/ascribe-cli/tests/output/**", "examples/**", "packages/astro/starter/**", "site/demo/**"]

[consumer]
site = "https://ascribed-dev.com"
base-path = "/docs/"
headers = "netlify"
permalink-path = "/go/"

[checks]
example-stale = "error"

[checks.vale]
preset = "quiet"

[permalinks.retired]
cli-render = { to = "command/questions.md#cli-resolve", reason = "The command ascribe render was renamed ascribe resolve" }
```

About 50 lines of settings in place of 106, with the comments kept. The docs may still write a default where saying it teaches something, with a comment that it's the default; nothing reports it.

## What it changes that's a contract

Each of these is a recorded decision, in the pull request that makes it:

- **SPEC §7.2:** `[frontmatter]`; `title` implicit on every type, with `model-type-title` gone; the roles taken by a field's name; a type's template and guides from the lock, amending the page templates plan's decisions 7 and 14.
- **SPEC §4.4 and Appendix A:** `planned` as a version word in an availability spec.
- **The content model reference's `[releases]`:** the implicit `release` dimension, reserved, and its two `label`s.
- **SPEC §7.3:** a source always ignores `node_modules` and `.git`.
- **SPEC §9.4 and `[consumer]`:** `agents` defaults to `true` when `site` is set.
- **A new section of §7:** `[from]`, what it takes, and its path's rule.
- **The diagnostics:** `frontmatter-field-repeated` added, `model-type-title` removed, and the errors for `planned` with no planned line and for a reserved `release` dimension.

## What this leaves out

- **One file per table** (`[phrases] file = "phrases.toml"`). The brainstorm's other answer, for a project whose `[phrases]` or `[[intended]]` runs to hundreds of entries. Nothing here stops it later, and no project needs it yet.
- **A general `include`.** Rules for a key set in two files, nesting, and every in-place edit, for what `[from]` does with an explicit list.
- **A check that reports a setting equal to its default.** It would nag the docs, which write some on purpose.
- **Light and dark images as a built-in.** Both of the repository's projects declare `scheme = "enum(light, dark)"`, and the site's stylesheet does the switching. A built-in attribute with the elements doing the switch would be a feature of its own, not a default.

## What to build first

1. **Frontmatter every type shares, and the roles by name** (piece 1). The largest saving, in every project with types, and a change only to how the model is read: its thread is the docs' model with its types cut down, checked and built with no output changing.
2. **The template from the lock** (piece 2), on the same types.
3. **Releases declare their targets, with the planned line's word** (piece 3), with `versions release` simplified.
4. **The changed defaults** (piece 4), each a registry row and its reference.
5. **Taking tables from another project** (piece 5), last, since it's the one that changes every in-place edit.

Pieces 1 to 4 change no output of a project that writes everything out, which a conformance case for each holds. Piece 5 changes nothing for a project without `[from]`.

## Settled

Answered by the maintainer on 21 October 2026, with breaking changes acceptable: 1 and 2 first, and 3 to 5 taking the recommendations after the proposal's review.

1. **The planned line's word is `planned`,** the release list's word. _Why:_ it says what the mark means where `next` says only when.
2. **`[consumer] agents` defaults to `true`,** when `[consumer] site` is set: the agents' outputs need the site's origin (SPEC §9.4), so without a site the default is `false`, and only an explicit `true` is reported. _Why:_ Ascribe's position is docs that agents read, and the Markdown is the page's own text, already public.
3. **A type's field replaces a shared one whole.** A type that declares a field `[frontmatter]` has gets its own declaration for that type; fields aren't merged key by key, and a type's copy identical to the shared one is `frontmatter-field-repeated`, advice to delete it. _Why:_ types differ in exactly this way, as an index page's optional `description` does; an error would push the field out of `[frontmatter]` for every type the moment one differs, and merging would split one field's declaration across two tables. The hover and `ascribe model` say which table declared a field, so a copy that drifts is seen.
4. **`[from]` takes an explicit list,** not `extends`. _Why:_ `extends` needs syntax to undeclare what's inherited, since the landing site would take the docs' six types, `[checks]`, the glossary, and more; with it, an edit to the other project silently changes this one wherever it didn't override, where the list names exactly the keys that can change. In the repository's own case the list is about 8 keys and the differences 15 or more. A third file both projects take from, as Cargo's workspace inheritance has, adds a file neither project owns.
5. **`planned` is a version word, the only form:** `@available: ascribe planned`, not a feature key. _Why:_ a feature key is one token that stands for one spec, so a bare `planned` couldn't say which target's line it means once a project has two targets; as a word in the version's place each target says its own, states compose with it, and a mark reads the same in a project with one target or five. Answered after a review found the feature key ambiguous.
