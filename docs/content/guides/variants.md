---
title: Content that varies
description: Writing one page for readers on different platforms or products, with alternatives, availability, and a build for each audience.
---

Some of a page is the same for every reader, and some isn't: the install command differs by platform, and a section applies to some platforms and not others. Ascribe keeps it all in one page, and lets each output decide what its readers get. This guide adds a section that differs by platform to an install page, marks one that doesn't apply everywhere, and builds the page twice: once for the site, with every platform, and once for Windows readers alone.

It takes four parts:

- a **dimension**, in `ascribe.toml`: what content varies by, such as the platform;
- a **variant group**, in the page: the alternatives, one for each value;
- an **availability mark**, in the page: where a section applies;
- **builds**, in `ascribe.toml`: what each output keeps.

The example is the project in [`examples/platforms`]({repo}/tree/main/examples/platforms), and every example below is taken from it.

## 1. Declare a dimension

A dimension is an axis your content varies along, with the values it takes. Declare it in `ascribe.toml`:

@snippet: code:examples/platforms/ascribe.toml#dimension

- `values` are what a page names in its alternatives and availability marks, in the order the site shows them.
- `labels` and `label` are what readers see: a tab reads **macOS**, not `macos`.
- `versionless` says the platforms have no versions of their own, so a mark names a platform and nothing more. A product whose releases matter leaves it out, and marks content with the version it arrived in, such as `self-managed 3.3`. See [availability specs](../reference/directives.md#availability-specs).

See [`[dimensions.<name>]`](../reference/content-model/content.md#6-dimensionsname) for every key.

## 2. Write the page

@snippet {lang=markdown}: code:examples/platforms/docs/install.md

**The variant group** is the three `@variant` arms under "Install the CLI". Each arm opens with the value it's for, and its content runs to the next arm; one `@end` closes the group. An arm can hold anything a page can: paragraphs, lists, notes, more code. An arm can be for several values (`{platform=linux|macos}`), and a group doesn't need an arm for every value, though `ascribe check` warns when a build would leave a group with no arm at all. See [`@variant`](../reference/directives.md#variant).

**The availability mark** is `@available: linux, macos` under "Start it at login". Directly under a heading, it applies to the whole section; anywhere else, to the block it touches. Unlike a variant, it doesn't choose between alternatives: it says where the content applies, and every reader of a build that keeps it sees the mark. See [`@available`](../reference/directives.md#available).

"Sign in" has neither, so it's the same everywhere.

`ascribe check` reports a value the dimension doesn't declare, an arm with neither a value nor a title, and a group that isn't closed, at the line that has it. In VS Code, **Insert content that varies** and **Mark where it's available** write both for you, from the dimensions the project declares; see [Actions](editor.md#actions).

## 3. Declare the builds

A build is a named set of choices about what the output keeps. Every page goes through every build, so the source stays one page:

@snippet: code:examples/platforms/ascribe.toml#builds

- **`site`** keeps every arm of the group, for readers to switch between (`variants = "switch"`), and every section, with its availability shown (`availability = "badge"`). It's what a project without builds gets.
- **`windows`** keeps the arms for Windows (`variants = { platform = "windows" }`), and removes what isn't available on Windows (`availability = { filter = "windows" }`).

A build can do one and not the other: select a platform and still badge, or keep every arm and filter for an older release. See [`[builds.<name>]`](../reference/content-model/output.md#16-buildsname).

## 4. Build it

```sh
npx ascribe build
```

Each build writes its outputs to its own folder: `.ascribe/build/site/` and `.ascribe/build/windows/`, each with the site output, plain Markdown, and JSON. To see what one build keeps of a page, without writing anything, run `ascribe render` with the build:

```sh
npx ascribe render docs/install.md --build site
```

@snippet {lang=markdown}: code:crates/ascribe-cli/tests/output/platforms-site.txt

In the Markdown, each arm is a section under a heading that names the platform, after a line saying a reader sees one of them at a time, so an agent reading the page knows they're alternatives. In the site output, the arms are tabs, one for each platform. Groups that vary by the same dimension switch together, and the reader's choice holds on every page they open next. The section's mark is a badge under its heading.

```sh
npx ascribe render docs/install.md --build windows
```

@snippet {lang=markdown}: code:crates/ascribe-cli/tests/output/platforms-windows.txt

The Windows build has only the Windows command, with no tabs and no label, since there's nothing to choose between. "Start it at login" is gone: it isn't available on Windows. A Windows reader sees a page written for them.

In VS Code, the preview's **Build** picker shows a page as any build has it, and the [build lens](editor.md#the-build-lens) dims what the build you're looking at leaves out, in the source.

## Publishing more than one build

A site publishes one build: `ascribe({ build: "site" })` in Astro's configuration names it (see [Astro](astro.md)). Another build is for another output: a second site, a PDF, or plain Markdown for a product's help panel, built from the same pages. Run `ascribe build --build windows` to build one alone.

## Where to go from here

- **A whole page that differs.** When most of a page is different, write a page for each value, with `variant` in its frontmatter. A build that selects another value leaves the page out. See [whole-page variants](../reference/directives.md#whole-page-variants).
- **Alternatives that aren't a dimension.** For a one-off choice, such as two ways to do the same thing, give each arm a title instead of a value. See [`@variant`](../reference/directives.md#variant).
- **A word that differs.** `@variant` varies whole blocks. For a name or a version in a sentence, use a [phrase](../reference/directives.md#phrases).
- **Versions and lifecycle.** Availability can say when content arrived and what state it's in, such as `self-managed preview 3.4`, and a feature can be named once in `[features]` so going GA is one edit. See [availability specs](../reference/directives.md#availability-specs), [`[lifecycle.<state>]`](../reference/content-model/content.md#8-lifecyclestate), and [`[features.<key>]`](../reference/content-model/content.md#9-featureskey).
- **A table row.** A row takes `{available=...}` at the end of its first cell. See [table rows](../reference/directives.md#table-rows).
