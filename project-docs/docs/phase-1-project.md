# Phase 1: The user docs as a project

Part of [Docs](README.md). Needs no other phase. Docs, a content model, and CI.

## Goal

`docs/` is an Ascribe project. Every page is Ascribe source under `docs/content/`, the project passes `ascribe check --deny-warnings` in CI, and the pages use Ascribe's features where they fit. Nothing is published yet; this phase makes the source right.

## Context

- `docs/` as it is: `README.md` (the index), eight guides and references, and `contracts/` (four pages, converted like the rest; they stay in this project).
- `docs/content-model.md` and `docs/directives.md`: the features to use, described by the pages being converted.
- `examples/quill`, `examples/astro-site`, and `examples/monorepo/docs`: content models and pages written by hand, for how a project is laid out.
- `tests/conformance/tests/docs.rs`: writes `docs/diagnostics.md`. Its path changes in this phase; its form changes in phase 2.
- Everything that names a docs path: `grep -rn "docs/" --include="*.rs" --include="*.ts" --include="*.md" --include="*.json" --include="*.yml" .` (the binary's help text, package READMEs, the extension's manifest and README, `CONTRIBUTING.md`, workflows, and the other plans in `project-docs/`, which are left alone).
- `.github/workflows/`: where the check goes, and how the binary is built for other jobs.
- [Decisions 3, 6, and 13](README.md#decisions).

## Design

### Layout

```
docs/
  ascribe.toml
  content/
    index.md                 the landing page (today's README.md)
    getting-started.md
    guides/                  review.md, editor.md, astro.md
    reference/               directives.md, content-model.md, cli.md, diagnostics.md
    contracts/               the four contracts
    _fragments/              text shared between pages
```

Use `git mv`, so history follows the files. `docs/README.md` stays as two lines: what the folder is, and that the pages are under `content/`.

### The content model

Write it for these docs, and keep it small:

- **Types:** `guide` (the default) and `reference` (`reference/**`, `contracts/**`). Both require `title` and `description`. `reference` pages may carry `since`.
- **Phrases** for what would otherwise be repeated and go stale: the current version, the minimum VS Code and Node.js versions, the repository's address, the npm scope. Where a value has a source in the repository (a `package.json`, `Cargo.toml`), say so in a comment; phase 2 puts the ones that can be generated under test.
- **Dimensions**, only for what the pages already vary by: the package manager (`npm`, `pnpm`, `yarn`) and the platform (`macos`, `linux`, `windows`) in install steps.
- **A glossary** for the terms the docs define once and use everywhere: build, variant, dimension, fragment, phrase, content model, page preview, site preview.
- **Availability,** for what isn't in the latest release (decision 6). Read the content-model reference for what availability can express, and model "in `main`, not yet released" and "since version X" with it. Mark every feature the docs describe that the latest published version doesn't have. If availability can't say this cleanly, that's a finding: file it, and use the closest form that reads truthfully.
- **Builds:** one, `site`. `[editor] build = "site"`.
- **`[consumer]`** is left for phase 4.

### Converting a page

For each page, in this order:

1. Frontmatter: `title`, `description`.
2. Links: to other pages by file; to the repository through the repository phrase.
3. Constructs, where the page already has the shape: a "Note:" paragraph becomes `@note`; numbered procedures become `@steps`; per-platform or per-package-manager alternatives become `@variant`; a repeated product fact becomes a phrase; text repeated on two pages becomes a fragment.
4. `ascribe fmt`, then `ascribe check`.

Don't restructure a page or rewrite its prose. Don't add a construct for its own sake: a page that's a table and three paragraphs stays that.

`docs/content/reference/diagnostics.md` stays generated as one whole page in this phase, with the frontmatter it now needs; change the generator's output path and nothing else.

### CI

Until the canary exists (phase 3), a job that builds the binary and runs `ascribe check --deny-warnings` and `ascribe fmt --check` on `docs/`. It fails the pull request. This check stays on the pull request's own binary after phase 3 too: a pull request's docs are checked by that pull request's Ascribe.

### What dogfooding finds

Keep a list as you go: anything Ascribe couldn't express, any diagnostic that was wrong or unclear, any place `fmt` produced something worse. Each becomes an issue, linked in the pull request. If one blocks a page, work around it in that page with a comment naming the issue.

## Tasks

1. `docs/ascribe.toml`, the layout, and the moves (one commit of moves only, so the diff after it is readable).
2. Convert the pages, a commit per page or small group.
3. Every reference to a docs path, updated. The other plans in `project-docs/` are not edited; the plan's README covers them.
4. The CI job.
5. The list of findings, as issues.

## Out of scope

The site and publishing (phases 4 and 5); the generated fragments (phase 2); snippets and `covers` (phases 7 and 8); rewriting pages; `SPEC.md`, `CONTRIBUTING.md`, and the READMEs, which stay as they are.

## Acceptance criteria

- `ascribe check --deny-warnings` and `ascribe fmt --check` pass on `docs/`, and CI runs both.
- No link in the repository points at a docs path that no longer exists (search for the old paths).
- Each page's rendered text in the page preview says what it said before, apart from fixes named in the pull request.
- The findings are filed.

## Verify

```sh
cargo test --workspace --locked
cargo build -p tessera-cli
./target/debug/ascribe check --deny-warnings --config docs/ascribe.toml
./target/debug/ascribe fmt --check --config docs/ascribe.toml
pnpm format:check && pnpm lint && pnpm test
```

## Commits

1. "Lay out docs/ as an Ascribe project"
2. One per page or group: "Convert the command reference to Ascribe"
3. "Check the docs in CI"
