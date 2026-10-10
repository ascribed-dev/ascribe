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
- **Placeholders** become whatever Ascribe's form for "fill this in" turns out to be (see the open questions). Either way, no placeholder is read as a phrase.
- **The template's own instructions,** which sit in the body as quoted paragraphs, become text that's never published.
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

One command writes the starter set into a project: the templates, the guides, Ascribe's notes, and the types in `ascribe.toml` that name them. It's a copy, made once, like ejecting the Vale preset: from then on the files are the project's.

That's a deliberate choice against "always use the newest from Ascribe". A team's templates are its own house style, and they shouldn't change under it on an upgrade. A later run of the command can say what's changed upstream since the copy, for a team that wants to look.

### Starting a page

- **In the editor,** an action: pick a type, give a title, and the page is created from the type's template in the folder the type covers, with the cursor at the first thing to fill in.
- **At the command line and for agents,** the `new-page` prompt gains the template's body, and tells the agent to read the guide first, by its path. The guide is long, and a file an agent reads itself arrives whole; a prompt that tried to carry it would have to cut it.
- **In the rules `ascribe agents sync` writes,** each type gets a line saying what it's for, so an agent choosing where a new page belongs can tell a concept from a how-to.

### Knowing a page isn't finished

A page made from a template starts full of things to replace. If Ascribe can recognize one, it can report a page that still has them, as advice under the [content checks](content-checks/README.md) contract. That turns "I forgot to fill in the prerequisites" from something a reader finds into something `ascribe check` does.

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

## Open questions

- **How is "fill this in" written?** Their curly brackets are Ascribe's phrases. The form has to be something Ascribe recognizes, so a leftover can be reported, and something that can't be published by accident. It's close to the need the brainstorm's notes found: text in a page that never reaches an output. One construct could serve both. Is that the same construct, or does a placeholder need its own?
- **Does the starter set ship inside the binary?** It's small, a few hundred kilobytes of text against a binary of several megabytes, and it would make the command work offline with no second download. The alternative is a package of its own, which keeps the binary free of content and lets the set have its own release. Which matters more?
- **Should Ascribe's own docs use these types?** The docs have two page types today, `guide` and `reference`. Adopting concept, how-to, and the rest would test the templates on real pages and sort out where the docs' guides end and their references begin, which the [surface audit](surface-audit/README.md) is going to ask anyway. It would also mean restructuring the docs.
- **What is the command called?** The audit is settling how command groups are named. This one waits for that.
