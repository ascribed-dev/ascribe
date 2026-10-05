# Ascribe documentation

- [Getting started](getting-started.md): install Ascribe, write a first page, check it, build it, and publish it with Astro.
- [Directive reference](directives.md): the language: directives, attributes, titles, phrases, links, and images.
- [`ascribe.toml` reference](content-model.md): the content model: content types, dimensions, availability, phrases, widgets, and builds.
- [Command reference](cli.md): `ascribe check`, `build`, `diff`, `fmt`, and `lsp`, their options, outputs, and exit codes.
- [Review](review.md): reviewing a pull request as readers will see it, in the page preview, the site preview, or a report from CI, and commenting there.
- [Diagnostics](diagnostics.md): every problem Ascribe reports, with its code and fix.
- [Editing](editor.md): the VS Code extension, workspaces with several projects, and other editors.
- [Astro](astro.md): publishing a site with `@ascribed/astro`.

## For implementers

These contracts say what an emitter writes and what a consumer can rely on. They matter if you're changing Ascribe's outputs or writing a consumer, and not if you're writing documentation with it.

- [The `ascribe.toml` format](contracts/content-model.md): the normative contract for the content model file, with the rules a loader enforces and the decisions behind them. The [reference](content-model.md) above is the guide for writing one.
- [Output layout](contracts/output-layout.md): where `ascribe build` writes each build's output, how it records what it wrote, and how it replaces a previous output.
- [Assets](contracts/assets.md): where assets go in each output and how pages refer to them.
- [Site render](contracts/site-render.md): the markers the site output writes and how a consumer renders them. The fixtures are in [`tests/render/`](../tests/render/).

The [Ascribe specification](../SPEC.md) is the normative definition of the language. The [changelog](../CHANGELOG.md) lists what each release changed.
