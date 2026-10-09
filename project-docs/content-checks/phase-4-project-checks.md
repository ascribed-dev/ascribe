# Phase 4: Checks across the project

Part of [Content checks](README.md). Requires phase 1; needs phase 2 for acknowledgement, which most of these depend on to be bearable. Rust.

## Goal

The checks that need every page at once: what nothing uses, what nothing links to, and what was left behind.

## Context

- README decisions 1, 4, and 6, and the inventory's second table.
- [The editor UI plan](../editor-ui/README.md#the-agents-plan) and the agents plan's `ascribe refs`: the search for where a page, fragment, phrase, feature, or glossary term is used. It lives in a crate below the language server, and is built once. If neither plan has built it, build it here as they describe, in its own commit.
- `crates/ascribe-resolve`: the snapshot's links and includes, per build. `crates/ascribe-resolve/src/fs.rs`: `FileSystem`, for listing image files.
- `docs/content/reference/content-model.md`: `[features]` (§9), `[lifecycle]` (§8), `[versions]` (§7), `[images]` (§13).
- `docs/ascribe.toml`'s `[features.next]` and `RELEASING.md`'s step for the pull request after a release: the case `availability-left-behind` is for.

## Design

All are `advice` and `configurable`, and all run in `ascribe check`. The server reports those marked "in the editor" from its snapshot, on save.

| Check | Reported when | Kind | In the editor | Evidence in the prompt |
|---|---|---|---|---|
| `page-orphan` | No page links to it, in a build that publishes it | `review` | On the page, at its title | Pages that mention its title or share its folder |
| `fragment-unused` | No page includes it | `review` | On the fragment | — |
| `phrase-unused`, `feature-unused`, `glossary-term-unused` | Nothing uses the entry | `review` | In `ascribe.toml`, at the entry | — |
| `image-unused` | An image under the content root that no page shows | `review` | No | — |
| `image-large` | An image file over a size | `review` | No | The size, and its dimensions if they can be read without a new dependency |
| `title-duplicate` | Two published pages of one build have the same title | `write` | On both | Both paths and descriptions |
| `availability-left-behind` | A feature's availability names a state or version the model says is past | `fix` | In `ascribe.toml` | The feature, and the release it shipped in |

Details:

- **An orphan** excludes a project's index pages and anything a navigation file would list, once one exists. Until then the message says it's a hint: a site's own sidebar may link to the page. That's what acknowledging it is for.
- **Per build.** A page is an orphan in a build when no page that build publishes links to it. Report once, naming the builds.
- **Unused entries in `ascribe.toml`** can't carry a directive, so their acknowledgement is whichever form phase 2 chose for model entries.
- **`availability-left-behind`** needs a definition from the model, not a guess. Read `[versions]` and `[lifecycle]` first. If the model can't say that a release has happened, stop and report: the check may need a small addition to `[versions]`, which is a content-model decision.
- **`image-large`**'s limit is `[checks.image-large] limit`, with a default chosen from what the docs and examples contain.

## Tasks

1. The shared search, if it doesn't exist.
2. Each check, in its own commit, with every field of decision 1, conformance cases, and reference entry.
3. Timing: `ascribe check` on the 3,000-page corpus before and after (`tests/corpora/RESULTS.md` has the method). The checks together add no more than 10%.
4. Run them on `docs/` and `examples/`; fix or acknowledge what they find.
5. `docs/content/`, `CHANGELOG.md`.

## Out of scope

A page missing from navigation, and a moved page with no redirect: both wait for their features (README, "The other plans"). The inventory as a report (phase 7).

## Acceptance criteria

- Each check is right per build, with a case where two builds disagree.
- A count from each "unused" check matches the editor UI's inventory for the same project, by test, once that exists.
- `ascribe check --deny-warnings` on `docs/` still exits 0.

## Verify

```sh
cargo test --workspace --locked
target/debug/ascribe check --deny-warnings --config docs
```

## Commits

One per check, then "Fix what the project checks find in the docs".
