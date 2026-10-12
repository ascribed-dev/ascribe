# Phase 2: The thread

Part of [Versions](README.md). Needs phase 1.

## Goal

A project with two live lines builds an output for each, and one site serves both. In `examples/quill`, with `live = ["3.4", "3.5"]` for `self-managed` and `versions = "self-managed"` on the site build:

```console
$ ascribe build
site        42 pages   self-managed 3.5, current
site@3.4    40 pages   self-managed 3.4
```

A page marked `self-managed 3.5` is in the first and not the second. The pages about the cloud service are in both. The Astro example serves the second under `/3.4/`, and each page lists the lines it's in.

This is the thinnest slice through every layer the feature touches. It exists to find out whether one build can become several outputs that know about each other (README, "The thread").

## Context

- README decisions 1 to 4 and 7 to 11, and phase 1's decisions 2, 5, 6, 7, and 11.
- SPEC §9.3, "Availability mode": what "available for target *T* at version *V*" means, which a line widens from a point to a range.
- `crates/ascribe-model/src/model.rs`: `AvailabilityMode`, and a build.
- `crates/ascribe-resolve/src/build/`: `modes.rs` and `availability.rs`, where a filter decides what stays, and `router.rs`, where a route is made from one base path.
- `crates/ascribe-emit/src/`: `write.rs` and `store.rs`, the output directory's ownership rules, which work for each build through its manifest, and `site/`, where `_ascribe/pages.json` is written.
- `docs/content/contracts/output-layout.md` §1: "Nothing is shared between emitter roots or builds."
- `packages/astro/src/`: `projects.ts`; `content.ts`, where a collection is Astro's `glob` over one site output; and `index.ts`, where a base path is checked against Astro's `base`, the plain output is published, and each permalink gets its forwarding route.
- `examples/quill/ascribe.toml`, and `examples/astro-site`.

## Design

### The model

`[releases.<target>]` takes one key in this phase, `live`: a list of versions, each a line (phase 1, decision 2).

- `<target>` is a declared dimension value that isn't versionless.
- The lines of one target have the same number of parts, and no two are the same.
- The current line is the newest.
- Any other key is the unknown-key error, as everywhere in the file. Phase 3 adds `planned`, `ends`, `archived`, and `planned-label`.

A build takes `versions = "<target>"`:

- The target has a `[releases]` table with at least one live line.
- `availability` is `"badge"`, written or by default. With a filter it's an error: a filtered build is already one version of one target.
- It doesn't change `variants`.

A build with `versions` and one live line is one output, named as the build is, at the root. That's a project's first day at the third level, and nothing about its output moves.

### What a line's output holds

For line *L* of target *T*, content is **in the line** when its effective availability:

1. has no spec, or doesn't list *T*, directly or through *T*'s dimension name; or
2. lists *T*, and some state that counts as available is in effect for *T* at a version in *L*.

A version is in *L* when it starts with *L*'s parts. So content that begins at `3.4.2` is in line 3.4, and content removed at `3.4.2` is in it too, since it was there at `3.4.0`. The second is the one place a line differs from a point, and it gets a conformance case of its own.

Content that isn't in the line is removed, as a filter removes it: a block, a table row, a section, or a page. What remains is marked as a badge build marks it, with one difference, phase 1's decision 11: a version at or before the oldest live line isn't shown, in any line's output. With 3.4 and 3.5 live, `self-managed 3.3` shows the target alone, `self-managed deprecated 3.2` shows "deprecated", and `self-managed 3.4.2` shows its version. A build that isn't versioned shows every version, as today.

In this phase the current line also keeps content whose versions are all later than every live line, marked as it is today ("3.6+"). Phase 3 says "planned" there. An older line doesn't.

### Routes

A line's pages are under the project's base path and then the line: `/docs/3.4/install/`. The current line's are where they'd be with no versions.

Every address in a line's output starts there: links between pages, the files under `_ascribe/files/`, the permalinks list, the page records' `canonical`, and the plain output's absolute links. The router takes its base path from the line and not from `[consumer]` alone, in one place.

A page whose route would fall under another line's (`3.4/index.md` in a project with a line `3.4`) can't be published, and the build fails naming both, as it does for two pages with one entry id.

### Names and folders

The outputs are `<output-dir>/site/` and `<output-dir>/site@3.4/`. `--build site` builds the current line, `--build site@3.4` that one, and `ascribe build` with neither builds every line of every build. `--build` with a line that isn't live is exit 2, naming the lines that are.

### The file the lines share

`<output-dir>/_ascribe/versions.json`, as phase 1's decision 5 has it: for each versioned build, its target, its lines, and for each page the lines that publish it with its route in each. It's written by every build of a project with `[releases]`, whichever build was asked for.

Nothing owns a file above the builds today: the store's rules work for each build, through that build's manifest. So the folder gets a manifest of its own, `<output-dir>/_ascribe/manifest.json`, in the same format, and the store is pointed at it as it is at a build's folder. The rules then hold one level up with no new ones: Ascribe replaces and removes only what the manifest lists, and refuses an `_ascribe` folder it didn't write. A build of a project that no longer has `[releases]` removes the file, the manifest, and the folder when nothing else is in it.

The output layout contract gains a section for the folder and the file, and the sentence that nothing is shared between builds gains its one exception.

### Check

`ascribe check` checks each line as a build. A problem in more than one line is said once, as phase 1's decision 6 has it. Nothing is done to save the work: each line is resolved in full. Phase 6 saves it.

A link to a page that isn't in a line is `link-page-dropped`, as it is in any build. Phase 4 replaces that.

### Astro

As phase 1's decision 7 has it:

- A project whose build is versioned gives one collection with every line's pages. An entry's id is its route under the base path.
- **The collection has a loader of the integration's own.** `ascribeCollection` is Astro's `glob` over one site output today, and the lines are separate folders, `site/` and `site@3.4/`. For a versioned build the integration reads `versions.json`, walks each live line's output, and gives each entry its id with the line in front and its data from that line's file. It renders and watches as `glob` does, and a build that isn't versioned keeps `glob`. This is half of what the thread is for: if a loader can't match `glob`'s behavior for images, for rendering, and for `astro dev`'s reloads, a site needs a collection for each line after all, and phase 1's decision 7 is reopened before anything ships.
- An entry's data gains `line` and `current`.
- `@ascribed/astro/Versions.astro`, given a page's entry, writes a list of the lines that publish the page, each a link, with the page's own line marked. A line that doesn't have the page isn't listed. It reads `versions.json`.
- `astro dev` rebuilds every line on a change.
- **A permalink forwards to the current line.** Each line's output has its own `_ascribe/permalinks.json`, with routes under its number, and the integration mounts one forwarding route for each name and refuses two lists that share one. For a versioned build it reads the current line's list alone. That's the rule and not a stopgap: a permalink is the address a product prints, and it should land on the newest docs.
- **The plain output is published for the current line only.** An older line's Markdown and `llms.txt` are built, in its output, and the integration doesn't copy or serve them. Published as they are, each would say nothing of which version it's for, and its `llms.txt` would sit beside the current one as its equal. Phase 5 gives each line its own and publishes them.

`examples/astro-site` gains a versioned project and a layout that includes the component, and its end-to-end tests fetch a page in each line.

## What this phase doesn't handle, said where a user will look

- **The reference's section on `[releases]`** says `live` is the only key yet.
- **The guide,** `docs/content/guides/versions.md`, opens with the four levels and says which exist: the second with one key, and the third without links between lines, a banner, or canonical links to the current line.
- **The Astro guide** says an older line has no Markdown version and no `llms.txt` at its addresses yet, and that a permalink always forwards to the current line.
- **`[editor] build`** takes a line's name. The build lens lists a versioned build once; phase 6 lists its lines.

## Tasks

1. `[releases.<target>] live` and a build's `versions`, with their rules and diagnostics.
2. A line's range, and what's in a line, in `ascribe-resolve`, with unit tests and conformance cases, the `3.4.2` cases among them.
3. A line's base path through the router, the assets, the permalinks list, the page records, and the plain output.
4. The outputs' names and folders, and `--build` naming a line.
5. `versions.json`, the folder's manifest, its removal when `[releases]` goes, and the output layout contract.
6. Each line checked, and a problem said once.
7. The loader, `line` and `current`, `Versions.astro`, the permalinks' one list, and the plain output's one line.
8. `examples/quill` and `examples/astro-site`.
9. The reference, the contract, the start of the guide, an entry in `changes/`, and the decisions recorded.
10. The map of the surface, regenerated.

## Out of scope

- `planned`, `ends`, `archived`, and every check the list makes possible.
- Links between lines, and `replaces`.
- Anything a crawler or an agent is told.
- Speed.

## Acceptance criteria

- `ascribe build` in `examples/quill` writes both outputs and `versions.json`, and the page marked `self-managed 3.5` is in one.
- Every link in `site@3.4`'s pages starts with the base path and `3.4/`, and the link test over the built Astro example follows each.
- The example site serves a page at `/3.4/…` and at the root, and each lists the other. An image on the older line's page is processed as one on the current line's is, and editing a source in `astro dev` reloads both.
- A permalink forwards to the current line's page, and the site builds with both lines' lists present.
- Deleting `[releases]` and building removes `versions.json` and its folder, and an `_ascribe` folder Ascribe didn't write is refused and left alone.
- A mark naming a version before the oldest live line shows no version in either line's output, and shows it in a build that isn't versioned.
- A project with `versions` and one live line builds byte for byte what it built without `versions`, apart from `versions.json`.
- `node scripts/compare/outputs.ts --base main` shows no change in a project without `[releases]`.
- The full matrix passes from the branch: `site@3.4` is a folder on Windows.

## Verify

```sh
cargo test --workspace --locked
cargo run -p ascribe-cli -- build --config examples/quill
pnpm --filter @ascribed/example-astro-site test:e2e
node scripts/compare/outputs.ts --base main --accept 'quill: *'
```

## Commits

1. "Let a project say which lines of a target are live"
2. "Build an output for each live line"
3. "Write what each line knows about the others"
4. "Serve every line of a project from one Astro site"
5. "Start the guide to versions"
