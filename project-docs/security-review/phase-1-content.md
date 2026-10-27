# Phase 1: Untrusted content

Part of [Security review](README.md). Needs phase 0.

## Goal

Find what a project's content can make Ascribe do: read or write outside the project, crash or hang a long-running process, or write something into an output that runs where the author didn't put it.

## Questions

### Parsing

- Can any input panic a library? Libraries are linted against `unwrap`, `expect`, and `panic!`; look for indexing, slicing, arithmetic overflow, and recursion without a limit, in the syntax crate, the frontmatter reader, `ascribe.toml`'s loader, the TOML files beside it, the OpenAPI reader, and the JSON example validator.
- Can any input make a parse take far too long or far too much memory: deeply nested containers or lists, a YAML alias bomb in frontmatter or a spec, a huge single line, a regex the content model supplies (a field's pattern, a check's rule), a `$ref` cycle in a spec's components?
- In the language server, which runs for hours, does one bad file stop the server or only its own diagnostics?

### Paths

- Every way a path enters from content: a link, an image, an include, a snippet's source and file, a template, a spec, a nav file entry, an asset under `_ascribe/up/`, a `[sources]` folder, a permalink's or an address's key. For each, can `..`, an absolute path, a Windows drive or UNC path, a backslash, a symbolic link, a case difference, or a percent-encoded segment reach a file outside what `FileSystem` allows?
- When a build copies an asset, can its destination escape the output directory?
- When Ascribe writes a file (`fmt`, `rename`, `check --fix`, `ascribe addresses`, `permalinks sync`, `agents sync`, `templates eject`), can content steer the write outside the project?

### What the outputs write

- The site output: is every value Ascribe itself puts into an attribute or an element escaped, as opposed to the author's raw HTML (README decision 4)? Phrases substituted into attributes, a glossary term's definition in a link's `title`, a heading's `@id`, availability text, a widget's attributes, `@operation`'s values from a spec, `help.json`'s text.
- Values that come from outside the project's pages and reach HTML: an OpenAPI spec's summary and paths, a remote source's code, a git commit message in a report.
- The JSON and plain outputs, and the HTML report of `ascribe diff`: anything read as HTML later.

## Method

- Read the code along each question; try each candidate on a project in a temporary folder.
- Write fuzz targets with `cargo-fuzz` for the Markdown and directive parser, frontmatter, `ascribe.toml`'s loader, the OpenAPI reader, and the example validator; run each for an hour locally, and record crashes as findings. The targets are kept (README decision 5); a CI job runs them on a schedule.

## Output

Draft advisories for exploitable findings; `notes/phase-1.md` for hardening; the fuzz targets, in a pull request of their own with their CI job.
