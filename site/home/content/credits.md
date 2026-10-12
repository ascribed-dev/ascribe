---
title: Credits
description: Who builds Ascribe, the projects it's built on, and where its ideas came from.
---

@hero-banner:
# Credits

Who builds Ascribe, the projects it's built on, and where its ideas came from.
@end

## Who builds Ascribe

Built by Kyle Rollins with Claude. [Kyle Rollins](https://github.com/KyleBlankRollins) maintains Ascribe, and [Claude](https://www.anthropic.com/claude), Anthropic's model, works on it through [Claude Code](https://claude.com/claude-code). The maintainer decides what's built and reads what ships. Most of the code, the tests, and the docs are written by Claude under those decisions.

## What Ascribe is built on

The projects a reader would want to know about, and what Ascribe uses each for. The manifests in the [repository](https://github.com/ascribed-dev/ascribe) list every dependency, with its version.

### Inside the binary

Ascribe's parser is a fork of [comrak](https://github.com/kivikakk/comrak), a CommonMark parser written in Rust, taken at its release {comrak-release} and changed so that a directive line is a block of its own. comrak's license travels with the fork, and every change to it is listed in [the fork's notes](https://github.com/ascribed-dev/ascribe/blob/main/crates/comrak-ascribe/FORK.md#changed-locations).

- [clap](https://github.com/clap-rs/clap) reads the command line.
- [ariadne](https://github.com/zesterer/ariadne) draws a diagnostic under the source line it's about.
- [serde](https://serde.rs) reads and writes the JSON, YAML, and TOML.
- [toml](https://github.com/toml-rs/toml) reads `ascribe.toml`.
- [schemars](https://github.com/GREsau/schemars) writes the JSON Schemas of what the commands print.

### In the npm packages

- [Astro](https://astro.build) is what `@ascribed/astro` is an integration for.
- [Sätteri](https://www.npmjs.com/package/satteri), Astro's default Markdown processor, and [unified](https://unifiedjs.com), the other one the integration supports, carry its Markdown plugin.
- [github-slugger](https://github.com/Flet/github-slugger) works out a page's entry id from its file's name, as Astro does.

### Programs Ascribe runs and doesn't ship

You install these yourself, when you use the command that runs them.

- [Vale](https://vale.sh) checks the prose, for `ascribe vale`.
- [lychee](https://lychee.cli.rs) asks each external link whether it still works, for `ascribe report links`.
- [afdocs](https://agentdocsspec.com), the Web Documentation Delivery Spec's checker, checks what a built site publishes for AI agents, for `ascribe report agents`.

### Building Ascribe and its site

- [Pagefind](https://pagefind.app) indexes this site for its search, which needs no service.
- [Playwright](https://playwright.dev) drives Chromium for the site's tests, the element library's, and the extension's.
- [Vitest](https://vitest.dev) runs the JavaScript tests.
- [insta](https://insta.rs) keeps the Rust tests' snapshots, and [proptest](https://github.com/proptest-rs/proptest) feeds the parser generated input.
- [TypeScript](https://www.typescriptlang.org), [oxlint](https://oxc.rs), and [Prettier](https://prettier.io) check the JavaScript packages.
- [esbuild](https://esbuild.github.io) bundles the editor extension, and the script `ascribe diff` puts in its HTML report.
- [sharp](https://sharp.pixelplumbing.com) and [opentype.js](https://opentype.js.org) draw the icons and social cards.
- [axe-core](https://github.com/dequelabs/axe-core) checks the element library for accessibility.

## Where its ideas came from

Each of these is said where Ascribe uses it, and linked here.

| From | What Ascribe took |
|---|---|
| [CommonMark](https://commonmark.org) | The language everything else is added to. Every CommonMark document means the same in Ascribe, apart from a line that starts with a directive ([the specification, §1.4](https://github.com/ascribed-dev/ascribe/blob/main/SPEC.md#14-relationship-to-commonmark)). |
| [Bluehawk](https://github.com/mongodb-university/Bluehawk) | The tags that mark a region of a code file for `@snippet`, kept as they are, so code already tagged for it works ([directives]({site}/go/directives/)). |
| [AsciiDoc](https://asciidoc.org) | A title on its own line, above the directive it names ([the specification's rationale](https://github.com/ascribed-dev/ascribe/blob/main/SPEC.md#appendix-c-design-rationale)). |
| [Swift's `@available`](https://docs.swift.org/swift-book/documentation/the-swift-programming-language/attributes/) | Lifecycle states named by the version they start at, so availability needs no ranges ([the specification's rationale](https://github.com/ascribed-dev/ascribe/blob/main/SPEC.md#appendix-c-design-rationale)). |
| [HTML's custom elements](https://html.spec.whatwg.org/multipage/custom-elements.html) | A hyphen in a name marks a project's own widgets ([the specification's rationale](https://github.com/ascribed-dev/ascribe/blob/main/SPEC.md#appendix-c-design-rationale)). |
| [Elastic's docs-builder](https://github.com/elastic/docs-builder) | Links with empty text, filled from their target, and a large corpus to measure choices against ([the specification's rationale](https://github.com/ascribed-dev/ascribe/blob/main/SPEC.md#appendix-c-design-rationale)). |
| Ruby and Lua | A bare `@end` to close a container ([the specification's rationale](https://github.com/ascribed-dev/ascribe/blob/main/SPEC.md#appendix-c-design-rationale)). |
| [The Web Documentation Delivery Spec](https://agentdocsspec.com/spec/web/) | What a build publishes for AI agents: `llms.txt`, and each page's Markdown at its address ([the Astro guide]({site}/go/astro-guide/)). |
| [The Good Docs Project](https://www.thegooddocsproject.dev) | The page templates, once they're built ([the plan](https://github.com/ascribed-dev/ascribe/blob/main/project-docs/page-templates/README.md)). |
| [Steel threads, by Jade Rubick](https://www.rubick.com/steel-threads/) | How every plan's phases are ordered: the first is the thinnest path through every layer, working and shipped ([the skill](https://github.com/ascribed-dev/ascribe/blob/main/.claude/skills/steel-threads/SKILL.md)). |
