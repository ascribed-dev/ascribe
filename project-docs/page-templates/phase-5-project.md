# Phase 5: A project from nothing

Part of [Page templates](README.md). Requires phase 3. The command, and getting started.

## Goal

A project doesn't begin by copying a block out of the docs:

```sh
$ mkdir handbook && cd handbook
$ ascribe init --templates
wrote ascribe.toml
wrote templates/concept.md …
added six page types. Pages go under docs/, a folder for each type
$ ascribe check
checked 0 files: 0 errors, 0 warnings
```

It's the surface audit's finding 53: "Nothing starts a project, or a page, for a person."

## Context

- The proposal: "What else starts from a copy".
- `docs/content/getting-started.md`, "Create `ascribe.toml`", and `examples/getting-started/`, which the page takes its `ascribe.toml` from and a test holds to it.
- The surface audit's findings 53, 54, and 55, and [journey 1](../surface-audit/journeys/start-a-project.md): what an agent given the page and an empty folder did.
- `docs/content/content-model/index.md` §21: what an absent section means. A first `ascribe.toml` should set as little as that allows.
- `ascribe templates eject`, as phase 3 left it: it needs an `ascribe.toml` to add to, and always will (README decision 24).
- `docs/content/content-model/index.md` §4, rule 4: a page that no type's `files` match, in a project with no default type, is an error.
- `project-docs/checklists.md`: a command finds its project from its path arguments or the current folder.

## Design

### `ascribe init`

A command of its own (README decision 24). In a folder with no `ascribe.toml` at or above it, it writes one, in the current folder. With `--templates` it then takes the starter set, by calling what `ascribe templates eject` calls, so the two can't differ.

- Named by the checklist: a verb for what it does to the project. It isn't under `templates`, since a project isn't templates.
- `--format text|json`, the JSON listing what was written.
- It exits 0 when the project is there, and 2 when it wrote nothing.
- **Where there's a project already,** at or above the current folder, it refuses, names the `ascribe.toml` it found, and says that `ascribe templates eject` adds the set to it. Starting a second project inside a repository that has one takes a path: `ascribe init <DIR>`, from the current folder.

### The first `ascribe.toml`

Small and commented:

- `spec`, and `[project]` with its content root said outright, since it's the first thing someone changes;
- **a default type,** `page`, with a title and a description, for a page no other type's `files` match. Without one, the first `docs/index.md` is an error: six types that each cover a folder leave every page outside those folders with no type. `--templates` adds the six beside it;
- one comment for each thing most projects add next (a phrase, a dimension, `[consumer]`), pointing at its section of the reference by permalink, and adding none of them.

It doesn't guess: no site address, no builds, no dimensions.

A project that takes the set with `eject` later, having started without it, keeps whatever default it has. One with no default and pages outside the six folders is told so by `ascribe check`, as today, and `eject`'s output says which folders the new types cover.

### Getting started

The page's "Create `ascribe.toml`" becomes the command, with the file it writes shown from the example by `@snippet`, as now. The hand-written route stays as a short section for a project that wants no templates.

The page's own example project is made by the command, and the test that holds the page to it still passes.

This is where the audit's findings about the page meet this plan: read their issues first, and don't redo what one has done.

### What it doesn't start

A site. The four files an Astro site needs are the integration's to write, and that's an issue of its own (README, "Carved out"). The command's last line says where to go next: the Astro guide, by permalink.

## Tasks

1. `ascribe init`, with tests: an empty folder; with `--templates`; a folder inside a project, which is refused; a path given; and a folder with a file named `ascribe.toml` that doesn't parse, which is refused.
2. The default type, with a test that a page outside the six folders checks clean in a project made with `--templates`.
3. The text of the file, read by the maintainer: it's the first thing of Ascribe's a new user opens.
4. `examples/getting-started/`, made by the command, and its tests.
5. Getting started, the command reference, the page on templates, `CHANGELOG.md`, and the map of the surface.

## Out of scope

A site's files. Questions asked at a prompt: the command takes arguments and asks nothing. Starting a project from someone else's repository.

## Acceptance criteria

- In an empty folder, the goal's two commands give a project with six types and a default that checks clean, and a `docs/index.md` added to it is a page of the default type. A test does exactly this.
- The `ascribe.toml` it writes loads with no warning, and each comment's link is a permalink that exists.
- Getting started's steps, followed in an empty folder, work as written. A session in the audit's form, run once and kept, is the test: an agent given the page's address and an empty folder.
- `ascribe templates eject` in a project that already has an `ascribe.toml` does exactly what it did after phase 3, and in a folder with none still exits 2, now naming `ascribe init`.

## Verify

```sh
cargo test --workspace --locked
pnpm test
cargo run -p ascribe-cli -- check docs
```

## Commits

1. "Write a project's first ascribe.toml"
2. "Start getting started from a command"
