# Proposal: the glossary as pages

A proposal, not a plan. It says what the change is and what it would look like, so the idea can be judged before anything is designed in detail.

The plan for building it is in [glossary-pages/](../glossary-pages/README.md).

## The goal

A glossary's terms are written where the rest of the docs are written: in Markdown, on a page, by whoever writes the pages. A definition is written once.

## What's wrong today

A term is declared in `ascribe.toml`:

```toml
[glossary.terms.rollout]
term = "rollout"
aliases = ["rollouts"]
definition = "A plan for turning a flag on for a growing share of users."
link = "/reference/glossary.md#rollout"
```

Three things follow from that.

- **The definition is written twice.** `link` points at where the term is explained properly, which is content. `examples/monorepo/docs` shows it: its glossary page has a heading for each term, an `@id` the `link` names, and a paragraph whose first sentence is the TOML's `definition` almost word for word.
- **A definition is a string, not content.** It can't use a phrase, can't be marked with `@available`, can't differ by variant, and isn't checked by anything that checks prose. It's the only reader-facing text in a project that the language doesn't reach.
- **The file that configures builds holds the glossary.** Ascribe's docs have 8 terms, at five or six lines each. A product's glossary runs to hundreds, and `ascribe.toml` becomes mostly definitions, owned by people who shouldn't need to touch the rest of it. TOML has no way to include another file.

## What exists

- **`[glossary]`** has `match`, `case-sensitive`, and `terms` (`docs/content/content-model/index.md` §12). A term has `term`, `aliases`, `definition`, `link`, and its own `match` and `case-sensitive`.
- **Occurrences are linked for the author** (SPEC §5.4, and §9.2 step 7): whole words, in prose only, the longest term winning, the first occurrence on a page or every one. A term set to `marked` isn't linked automatically, for a word that also has an ordinary sense.
- **A term's id is the key in the table.** The site output puts it on each link as `data-ascribe-term`, with the definition as the link's title (`docs/content/contracts/site-render.md`). An acknowledgement names it (`[[intended]] term = "rollout"`), and `ascribe model` and the JSON output list it.
- **A term with no `link` is legal.** It's never linked, and its definition reaches the editor's hover and the JSON output only.
- **Two model errors and one check are about terms:** `model-glossary-duplicate-term`, `model-glossary-link`, and `glossary-term-unused`.
- **Two projects in the repository use it.** `docs/` has 8 terms and no glossary page: each links to the heading in the reference or a guide that documents the concept. `examples/monorepo/docs` has 3 terms and a glossary page.
- **Nothing teaches it.** The reference describes the table, and no guide walks through setting a glossary up.

## The change

### A term is a heading, marked

```markdown
---
title: Glossary
---

## R

### Rollout
@term {aliases="rollouts"}

A plan for turning a flag on for a growing share of users: by percentage, by segment, or on a schedule.

### Region
@term

Where a project's data is stored.
```

`@term` is a line directive at the top of a section, as `@id` and `@available` are there. It says the section defines a term.

- **The term** is the heading's text.
- **Where a link goes** is the heading: its id, from its text or from an `@id`.
- **The definition** is the section's first paragraph.
- **The term's id** is the heading's id. It's what `data-ascribe-term` carries and what an acknowledgement names, as the table's key is today.

`@term` is required on each term. A heading without it is structure: the `## R` above, or an introduction. A glossary sorted by letter or grouped by subject then works with no rule about heading levels.

It takes three attributes, each only when needed:

| Attribute | Means | As today's key |
|---|---|---|
| `aliases` | Other forms that count as the term, separated by `\|` | `aliases` |
| `match` | `first`, `every`, or `marked`, for this term | `match` |
| `case-sensitive` | `true` or `false`, for this term | `case-sensitive` |

### `[glossary]` names the pages

```toml
[glossary]
match = "first"
pages = ["reference/glossary.md"]
```

`pages` is a list of paths from the content root, each a page. It's always a list, as `files` is: one page for most projects, several for a glossary split by letter or by product.

- `match` and `case-sensitive` stay, as the settings every term starts from.
- `terms` goes. A project that still has it gets an error that names `pages` and `@term`.
- A project with no `[glossary]`, or with no `pages`, has no glossary, as now.

### `@term` is for a glossary page

`@term` on a page that isn't in `pages` is an error, with a fix that adds the page. It's a restriction chosen for now and not a limit of the design: a project that wants a term defined on the reference page that documents it can list that page, and "any page" could be allowed later without changing a project that works today.

A glossary page is an ordinary page otherwise. It has a type, frontmatter, and a route; it can include a fragment, so terms shared by two glossaries live in one; and its sections can be marked with `@available` or written as variants.

## What gets better

- **One home for a definition.** The page a reader lands on and the text a hover shows are the same words.
- **A definition is content.** It takes phrases, links, and code. Prose checks, Vale, and review read it.
- **A term can belong to a version or a product.** A section marked `@available: self-managed 3.5` is a term the 3.4 line doesn't have, and a build that doesn't publish the section doesn't link the term, which the spec already says of a target that isn't published. The [versions plan](../versions/README.md) has no other answer for a term that's new in a line.
- **`ascribe.toml` gets shorter,** and the glossary gets an owner who never opens it.
- **Review shows a changed definition as a changed page,** where today it's a diff of a TOML string that no page view shows.

## What changes underneath

- **Terms are found by reading content, not the model.** Matching needs every term before any page is resolved, so the glossary's pages are read first. The source index already holds each page's headings and directives; whether it can supply the terms without a second parse is the first thing a plan should find out.
- **The short definition is derived.** The link's title, the editor's hover, and the JSON output take plain text today. They get the first paragraph rendered to text: phrases substituted, formatting dropped, links reduced to their words. It's resolved for each build, since a phrase or a variant can change it.
- **Errors move from the model to the page.** Two terms or aliases with the same text is an error at the second, naming the first, whichever pages they're on. A term with no paragraph under it is an error: there's nothing to show. A first paragraph too long for a hover is advice, at the same limit as `description-too-long` (settled below).
- **`model-glossary-link` goes.** A term can't link to a page that doesn't exist: it's on it.
- **A term with nowhere to link goes too.** Every term has a section, so every term can be linked. A term that shouldn't be linked automatically is `match=marked`, as now.
- **A term isn't linked on the page that defines it,** which the spec says already, and matching never applies to headings, so a glossary page doesn't link to itself.
- **`glossary-term-unused`** reports at the term's heading, and is acknowledged there, with `@intended` under `@term`; `term` leaves `[[intended]]` ([plan decision 17](../glossary-pages/README.md#decisions)).

## What it touches

| Where | What |
|---|---|
| `SPEC.md` | §4, a new built-in directive; §5.4; §7.2, the content model's declarations; the grammar, if `@term`'s attributes need a rule |
| `ascribe-model` | `[glossary] pages`; `terms` removed |
| `ascribe-resolve` | Terms from the glossary's pages; the definition for each build |
| `ascribe-check` | The page-level errors, and the unused check's place |
| `ascribe-emit` | The link's title and the JSON output, from the derived definition. The site output's markup doesn't change |
| `ascribe-lsp` | Hover, go to definition on a term, completion and a quick fix for `@term`, and the outline |
| `ascribe-query`, `ascribe-mcp` | `ascribe model` lists terms with the page each is on |
| `ascribe-fmt` | Canonical form for `@term` and its attributes |
| The docs | The directive reference, `ascribe.toml`'s reference and its contract, and a guide to the glossary, which doesn't exist |
| `docs/`, `examples/monorepo/docs`, `examples/content-models` | Their terms, moved |

The site output's contract for a glossary link, `data-ascribe-term` with the definition as its title, stays as it is, so a site that styles terms or shows definitions needs no change.

## Ascribe's own docs

They have no glossary page. Their 8 terms link into the reference and the guides, so a reader who follows "dimension" lands partway down the `ascribe.toml` reference.

Under this proposal they get one, `reference/glossary.md`: eight sections, each a sentence or two and a link on to the page that documents the concept in full. That's a better place to land, and it's the first real use of the feature.

The alternative, listing the reference pages in `pages` and marking the eight headings there, keeps today's targets. It would make the docs the one project that uses `@term` outside a glossary page, in the documentation that teaches people to use it on one.

## Moving a project over

Ascribe is before 1.0 and has no users, so there's no period where both forms work. `[glossary.terms]` is removed in the release that adds `@term`, and the changelog says **Behavior change**.

The move itself is mechanical for a project whose terms link to a glossary page already: add `@term` under each heading, with the aliases, and delete the table. The three projects in this repository are moved by hand in the pull request that makes the change. A command that does it isn't proposed.

## What this leaves out

- **`@term` on any page.** Above.
- **A generated glossary page,** or a directive that lists every term. The page is written.
- **A term defined by something other than a heading:** a definition list, or a row of a table. One form, the one the example already uses.
- **Splitting `ascribe.toml` into several files.** This removes the table that most needed it. `[phrases]` and `[[intended]]` can grow too, and that's a separate question.
- **A popover in the site.** The link's title is what a site gets today, and what it does with it is the site's.
- **Glossaries in more than one language.**

## Settled

The four questions this proposal left open, answered by the maintainer on 11 October 2026.

1. **A term's id is the heading's id, and `@term` takes no primary.** It's the id links already use: today's `link` names the heading, and `data-ascribe-term` carries the table's key, so one id serves both. A heading whose id could change already has its answer, `@id`, with the editor's **Add a stable @id** action and **Rename**, which rewrites the links to it and should rewrite an acknowledgement's `term` too. Nothing else names a term by id: its acknowledgement sits at its heading ([plan decision 17](../glossary-pages/README.md#decisions)), so a renamed id has nothing to rewrite. An id written on the directive would be a second name for one section, on every term, to protect the rare case.
2. **Aliases are separated by `|`, and the value is quoted when it needs to be: `aliases="content models|content model"`.** The language already reads `|` as "any of these", in `@variant`'s value sets, and `,` is what separates attribute pairs, so an unquoted comma would end the value. `aliases` isn't a set-valued key in the schema's sense, though: a set's members are tokens, which can't hold a space, and a glossary's aliases often do (`content models`, in Ascribe's own docs). So it's a string attribute whose value `@term` splits on `|`, with spaces around each member trimmed. An alias can't contain `|`; nothing a glossary needs does.
3. **Ascribe's docs get a glossary page, `reference/glossary.md`.** Eight sections, each a sentence or two and a link on to the page that documents the concept in full. A reader who follows a term lands on a definition instead of partway down a reference, and the docs that teach `@term` use it the way it's meant to be used. Listing the reference pages in `pages` would keep today's targets and make the docs the one project marking terms outside a glossary page.
4. **The definition's limit is `description-too-long`'s, and the check is advice.** Both are one sentence shown where there's no room for more: a description on its line in `llms.txt`, a definition in a link's title and a hover. One number, one constant today, and a setting for both at once if a project ever asks ([plan decision 19](../glossary-pages/README.md#decisions)). It's counted on the derived text, after phrases are substituted and formatting dropped, for each build. The fix is to end the first paragraph after the definition and put the rest in a second, which the page can hold and the hover shouldn't.
