# Proposal: page templates and their guides

A proposal, not a plan. It says what the feature is and what it would look like, so the idea can be judged before anything is designed in detail.

## The goal

Someone starting a page of a known kind (a how-to, a concept, a troubleshooting page) starts from the right shape, with guidance on what belongs in it. So does an agent asked to write one.

Ascribe would get that shape and guidance from [The Good Docs Project](https://www.thegooddocsproject.dev/template), which has spent years working out both, and stay current with it as it changes.

## What exists

**In Ascribe:**

- A project declares page types in `ascribe.toml`, each with its frontmatter and the files it covers. That's all a type is: fields, not a body.
- The `new-page` prompt writes a new page's frontmatter for a type, and where its file goes. It has nothing to say about what comes after the frontmatter.
- Nothing ships a ready-made content model. A new project starts from one line, `spec = "0.1"`.

**In The Good Docs Project** (release v1.6.0, June 2026):

- **25 templates in three packs.** The Core pack is the one that matters here: concept, how-to, tutorial, reference, troubleshooting, release notes, and README.
- **A guide with every template,** saying what the document is for, what goes in each section, and what doesn't. Often a worked example, a process document, and a list of sources too. The guides are about 37,000 words; the templates, 17,000.
- **An index,** `index.json`, naming each template's files, pack, summary, and the date it last changed.
- **The MIT No Attribution license.** They can be copied, changed, and shipped with no notice required.

**What happens when Ascribe reads them as they are.** Six Core templates, checked unchanged, gave 5 errors and 18 warnings, from two causes:

- **Placeholders are written in curly brackets,** 561 times across the set. A one-word placeholder such as `{concept}` is an undeclared phrase to Ascribe.
- **Each template links to its guide** by a relative path that isn't there once the template is alone.

## The balance

The templates have to be changed to suit Ascribe, and they have to keep following their source. Those pull against each other: every hand edit is something to redo, or to lose, when upstream moves.

The proposal's answer is to keep their words and Ascribe's apart, in three layers:

| Layer | Whose words | How it's made | When upstream changes |
|---|---|---|---|
| **The template and its guide** | Theirs | Converted by a script, from a pinned release | Regenerated. Nothing to merge |
| **A few structural rules** | Ours, as data | A short table the script reads: "this numbered list is a procedure" | Reapplied. A rule that no longer matches fails loudly |
| **Ascribe's notes on a type** | Ours | Written by hand, in a file beside the guide | Untouched |

The rule that holds this together: **convert, don't rewrite.** Nobody edits a converted template. A sentence of theirs that's wrong is fixed upstream, in their repository, where it helps everyone. What's particular to Ascribe ("write the alternatives by platform as a variant group") goes in Ascribe's notes, which upstream never sees and never overwrites.

So the templates stay recognizably theirs, and following a new release is a version number changed and a diff read.

## The conversion

A script takes a pinned release of their repository and writes Ascribe's copies. What it does is mechanical, and listed, so the result can be predicted from the source:

- **Frontmatter,** for the page type the template is for.
- **Placeholders keep their curly brackets, escaped.** Ascribe already reads `\{` as a literal brace, so `\{insert a brief description of the task}` is ordinary text: it passes `ascribe check`, `ascribe fmt` leaves it alone, and it shows as `{insert a brief description of the task}`. Their wording isn't touched.
- **The template's own instructions,** which sit in the body as quoted paragraphs, become text that's never published (below).
- **The link to the guide and the footer** go. Ascribe knows where the guide is.
- **Ascribe's canonical form,** by running the result through `ascribe fmt`.
- **The structural rules:** a numbered list of steps gains `@steps`; a callout becomes a `@note`. Each rule names the template and the place, and the script stops if the place isn't there any more.

The test that the committed copies match the pinned release is the kind the repository already has for generated files. A scheduled job looks for a newer release, as the docs repository's example does for a code source, and opens a pull request with the regenerated set, so a person reads what changed in their text before it ships in Ascribe's.

## In a project

### A type names its template and its guide

```toml
[types.how-to]
files = ["how-to/**"]
template = "templates/how-to.md"
guide = "templates/how-to.guide.md"

[types.how-to.frontmatter]
title = "string"
description = "string"
```

Both are files in the project, outside its content, that the project owns. A team that wants its how-tos to open with a different section changes its template. Nothing in Ascribe overwrites a project's files afterwards.

A type with neither key works as it does today.

### The starter set gets in by being copied

The starter set is inside the binary: it's a few hundred kilobytes of text, and it changes rarely. One command writes it into a project: the templates, the guides, Ascribe's notes, and the types in `ascribe.toml` that name them. It's a copy, made once, like ejecting the Vale preset: from then on the files are the project's.

That's a deliberate choice against "always use the newest from Ascribe". A team's templates are its own house style, and they shouldn't change under it on an upgrade. A later run of the command can say what's changed upstream since the copy, for a team that wants to look.

### Starting a page

- **In the editor,** an action: pick a type, give a title, and the page is created from the type's template in the folder the type covers, with the cursor at the first thing to fill in.
- **At the command line and for agents,** the `new-page` prompt gains the template's body, and tells the agent to read the guide first, by its path. The guide is long, and a file an agent reads itself arrives whole; a prompt that tried to carry it would have to cut it.
- **In the rules `ascribe agents sync` writes,** each type gets a line saying what it's for, so an agent choosing where a new page belongs can tell a concept from a how-to.

### Knowing a page isn't finished

A page made from a template starts full of things to replace, and an escaped placeholder is only text. Left in, it would be published.

Ascribe can still find one, without a new construct, because it knows which template a page's type names. A piece of text in a page that's also a placeholder in its type's template was never filled in. That's an exact comparison, and it's reported as advice under the [content checks](../content-checks/README.md) contract: "this page still has its template's `{Task name}`". It turns "I forgot to fill in the prerequisites" from something a reader finds into something `ascribe check` does.

It only knows the placeholders of the template as it is now. A project that rewrites its template after pages were made from it loses the check for the old wording, which is a fair trade for adding nothing to the language.

### Text that's never published

A template's instructions to its author ("this section is optional; delete it if…") shouldn't be escaped text: they're paragraphs, and they shouldn't reach a reader even when forgotten.

That's the need the [brainstorm](../brainstorm.md)'s notes found: text in a page that no output carries. It's one construct, and a template's instructions are its first use. This proposal depends on it and doesn't design it; the [notes proposal](notes.md) does, as `@comment`. A leftover instruction is reported the same way as a leftover placeholder.

## What ships

**The Core pack, less the README:** concept, how-to, tutorial, reference, troubleshooting, and release notes. Six templates, six guides, six short notes of Ascribe's, and a content model that declares the six types.

The content model is useful by itself. A new project gets a real starting structure instead of one line, and the getting-started page has something to point at.

The rest of their templates fall into three groups:

- **Worth adding next:** quickstart and the installation guide, then the API pair.
- **Overlapping something Ascribe has:** the glossary and terminology templates (`[glossary]`), the style guide (the Vale preset), the changelog (the [versions proposal](versions.md)'s report of what changed). Each is a prompt to check that Ascribe's feature and their guidance agree, more than a template to ship.
- **Not for a docs tool:** the code of conduct records, the team page, the user personas.

**Credit.** The license asks for none. Each converted file says where it came from and at which release anyway, and so does the docs' page on templates.

## What this leaves out

- **Checking a page's shape against its type:** "a how-to has a procedure". Their guides say enough to write such checks, and it would be the strongest use of them. It also adds a way to describe a page's outline to the content model, which is its own design. Later.
- **Templates for anything but a page:** a fragment, a section.
- **A template language.** A template is a page with things to fill in. It has no conditions, loops, or variables beyond the title.
- **Converting a project's existing pages** to a type.

## Ascribe's own docs

The docs adopt the types. They have two today, `guide` and `reference`. Taking on concept, how-to, tutorial, troubleshooting, and the rest is the first real test of the templates, on pages that already exist, and it settles where the docs' guides end and their references begin.

It's a restructuring of the docs, so it's its own step, after the feature works. The [surface audit](../surface-audit/README.md) looks at the docs' structure too, and what it finds should shape how the pages are sorted.

## Settled

- **One construct for text that's never published,** shared with notes. Templates don't get their own.
- **Placeholders are their curly brackets, escaped.** A leftover is found by comparing a page with its type's template.
- **The starter set ships inside the binary.**
- **Ascribe's docs use the types.**

## Waiting on the audit

**What the command is called.** The [surface audit](../surface-audit/README.md)'s first phase writes the conventions for naming a command and a group of subcommands, and its last phase checks this proposal's names against them. Until then the command has no name here.
