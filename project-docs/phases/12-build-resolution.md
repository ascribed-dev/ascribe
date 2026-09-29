# Phase 12: Build resolution and linking

**Track:** Resolve · **Start after:** 11 · **Finish after:** 03 · **Parallel with:** 23 · **Unblocks:** 13, 14, 18

## Goal

Turn expanded pages into resolved pages for a given build: apply availability and build modes, substitute phrases, assign page ids, resolve links, and link glossary terms. The result is exactly what the emitters and page-level checks consume.

## Read first

- [SPEC.md](../../SPEC.md): §4.3, §4.4, §5, and §9.2–§9.4 (especially the selection rules, the "available at version V" rules, and Assets).
- `project-docs/contracts/assets.md` (phase 02), and `crates/tessera-core`'s `Router` trait.
- Handoff notes from phase 11.

## Deliverables

- `crates/tessera-resolve/src/build/`: the build resolution passes and `ResolvedPage`.

## Tasks

1. **Availability.** Resolve feature keys, apply inheritance from page to section to block, and compute each node's effective availability (SPEC §4.4). Record scope-nesting problems for phase 14.
2. **Build modes.** Apply the build's variant mode and availability mode exactly as SPEC §9.3 defines them:
   - **Selection:** remove conflicting arms and pages. A group keeps every surviving arm: one arm becomes plain content, several stay a group. Groups on unselected dimensions and labeled groups are untouched. Record groups where no arm survives.
   - **Filter:** remove content not available for the build's target and version, including versionless targets whose single state doesn't count as available (for example `cloud removed`). Content that remains keeps its availability annotations.
3. **Phrases.** Substitute declared phrase candidates in prose, headings, link text, link destinations, opted-in fences, and the frontmatter fields the model allows. Undeclared candidates stay literal.
4. **Page ids.** Assign every heading its page id (SPEC §5.5): `@id`, or the slug computed across the whole expanded page after build modes (phase 09, one scope per page).
5. **Links.** Resolve each file-path link from its source id to the target heading's page id. Fill empty link text from the target's title. Compute each link's route through `tessera-core`'s `Router`; provide a simple default router for tests, and let phase 20 supply Astro's. Record links whose target a build removes.
6. **Assets.** Carry each page's surviving asset references, with provenance, into the result, as the asset contract requires, so emitters can copy and rewrite them.
7. **Glossary.** Link glossary terms according to the model's matching settings.
8. **Results.** `ResolvedPage`: the resolved tree (every node keeping its source file and span), frontmatter, effective availability, assets, and resolution problems. The tree preserves structure: surviving groups stay groups and availability annotations stay attached, so emitters work from what survived rather than re-deriving it from the build's mode.
9. **Conformance.** Remove skip entries for resolution and build cases, and make them pass.

## Acceptance criteria

- [ ] The Quill project resolves under all three of its builds with no problems.
- [ ] Tests cover: every selection rule in SPEC §9.3, including the cloud build keeping the whole `pm` group; filtering before a history's first state, between states, at a state that doesn't count as available, and a versionless `removed` target; annotations kept in filter builds; page ids that differ from source ids because an include duplicates a heading; empty-text links to pages and to ids; phrases in link destinations.
- [ ] Every resolution and build conformance case runs and passes.

## Out of scope

- Reporting diagnostics (phase 14), emitting output (phases 18 and 20), incremental updates (phase 13), and Astro's routing rules (phase 20).

## Notes

- Keep the passes separate functions with clear inputs and outputs, in SPEC §9.2's order. They're easier to test, and to reorder if the spec changes.

## Handoff notes

### What was built

`crates/tessera-resolve/src/build/` (one pass per module, in SPEC §9.2's order), plus the finish of Q43 outside it:

- `tree.rs`: `ResolvedPage`, `ResolvedBlock` (every block keeps its `file`, `span`, and `via`, the include chain), `ResolvedKind` (`Leaf(Block)`, `BlockQuote`, `List`, `Container`, `Group`), `Availability`, `Annotation`, `HeadingIds`, `Substitution`, `ResolvedLink` and `LinkTarget`, `GlossaryUse`, `ResolvedBuild`, `DroppedPage`, `DropReason`.
- `availability.rs` (step 2): feature keys, inherited scopes, each node's effective availability, `Availability::is_available`, and the scope check.
- `modes.rs` (step 3): selection and filter, page dropping.
- `phrases.rs` (step 4): substitution in prose, headings, link text and destinations (a definition's too), opted-in fences, and frontmatter fields with `phrases = true`.
- `ids.rs` (step 5): page ids, one slugger scope per expanded page, after build modes, only for headings without `@id`.
- `links.rs` (step 6): links to routes, empty text from the target's title, the assets that survive, `link-id-removed` and `link-page-dropped`.
- `glossary.rs` (step 7).
- `router.rs`: `DefaultRouter`.
- `mod.rs`: `BuildResolver` and the `Project` methods.

**Q43, finished.** `tessera_resolve::references::destination_phrases` gives a reference form's phrase candidates from its link reference definition (found by normalized label, else by decoded destination). The source index and `ascribe check` both call it, so a `[r]: {api}x` definition is substituted like an inline link, and a definition that makes a missing file is reported at the reference (Q53). `Target::Deferred` and `Resolution::Deferred` are gone, and `reference_target` lost its `form` parameter (the phrases it's given carry the difference). A definition's candidates are indexed once (`FileIndex::phrases`, place `Destination`). `tessera-check` keeps the file's definitions in its `Ctx` (a four-line change); its parity test gained definition-form references and a test that a definition's phrases apply.

### Interfaces later phases use

- **`Project::resolve_page(page, &Build, &dyn Router) -> Option<ResolvedPage>`**, **`Project::resolve_build(&Build, &dyn Router) -> ResolvedBuild`**, and **`Project::resolver(build, router) -> BuildResolver`** (`page(path)`, `build()`, `is_published(path)`), which keeps what it works out about each page so resolving every page of a build costs one pass over each. **`Project::dropped(page, build) -> Option<DropReason>`** says why a build doesn't publish a page (`Variant` for a `variant` frontmatter conflict, `Unavailable` for page-level `available`, Q24). `resolve_page` is `None` for a fragment or a dropped page.
- **`ResolvedPage`** holds `blocks`, `frontmatter` (phrases substituted where the model says) and `title`, `route`, the page-level `availability`, `assets` (`Vec<PageAsset>`, phase 11's type, in document order, one entry per surviving reference, with provenance) and `problems`. `headings()` lists every heading that survived with its `HeadingIds { source_id, page_id, explicit, text, level, explicit_at }`. `ResolvedBuild::assets()` is the build's copy list, by source path, deduplicated.
- **What survives is in the tree.** A group reduced to one arm is that arm's blocks; one reduced to several arms is a `ResolvedKind::Group`; a group whose arms are all removed is gone. A surviving `@available` stays a directive block with `annotation: Some(Annotation { text, feature, binding })`, where `text` is the spec as shown: for a feature key, the spec it stands for (Q25). Every block has `availability: Option<Arc<Availability>>`, its **effective** spec (its own, else its section's, else the enclosing block's, else the page's) and `Availability::enclosing` chains to the scope it sits in, so an emitter never re-derives it from the build. Bindings are the ones from each block's own file (`DirectiveLine::binding`).
- **Phrases are substituted in the tree**: a declared phrase is a `Text` inline, an undeclared candidate is a `Text` of `{key}`, adjacent text is merged (the merged node's span runs from the first to the last), a link or image destination is substituted, and a fence with `phrases=true` has its `literal` substituted. `ResolvedBlock::substitutions` lists each replacement over the block's source text (`span` in the block's file), for tools that render from source.
- **Links** are `ResolvedBlock::links` (`ResolvedLink { span, kind, destination (substituted), target }`, by the link's span in its file) and rewritten in the inline tree: a page link's `destination` is its URL (route, `#`, page id) and an empty link's text is the target's title. `LinkTarget` is `External`, `Page { page, id, url, text_filled }`, `Asset { path, fragment }` (the destination is left for the emitter to rewrite, asset contract §4), or `Unresolved` (target missing, a fragment, an id the page doesn't have, or removed or dropped by the build: the reason is a problem). A `#id` alone in a fragment compiles to the heading's page id on the including page (Q64).
- **Glossary**: a linked occurrence is an ordinary `Link` inline in the tree (its pieces share the text node's span) and is listed in `ResolvedBlock::glossary` (`term`, `text`, `url`), so the JSON output can say which terms were linked.
- **`Router`**: `DefaultRouter` (`new()`, `with_base(base, trailing_slash)`, `from_consumer(&Consumer)`) for tests and for callers with no profile; phase 20 supplies Astro's. Resolution takes `&dyn Router` only. **Where the router should replace the interim mapping:** `tessera_resolve::references::route` (Q55) guesses the page a route-like link names as `route.md`, else `route/index.md`. That's the inverse of a router, so when phase 20's router can answer "which page has this route", ask it there, and in `Resolution::Route`, which the `link-route` diagnostic and fix read. I didn't change what `ascribe check` reports for routes.
- **Problems, for phase 14** (`ResolvedPage::problems: Vec<PageProblem { issue, via }>`, located where the cause is written; a page-level report goes at `via[0]`, except `include-cycle`): what expansion found (`include-cycle`, `include-id-missing`), then `available-exceeds-scope` (at the `@available` line, one per directive, message variant `version` when it starts a target too early), `variant-no-arm-survives` (at the group's first opener, arg `build`), `link-id-removed` (args `build`, `id`, `path`) and `link-page-dropped` (args `build`, `path`), both at the destination as written (Q53). **A build records only problems about content it publishes** (Q81), so a problem in an arm a build removes is not there. Not recorded, because they follow from the headings: `id-duplicate` and `heading-duplicate-without-id`; `ResolvedPage::headings()` has what they need (each heading's page id, whether it's explicit, and where the `@id` is). Not recorded either: what the source index already reports, build-independent (`link-id-missing`, `link-id-in-fragment`: `Project::problems`).

### Decisions

- **Effective availability is the innermost spec, with the chain kept.** A node is available only if every spec in its chain says so; removal is per node, so a section's spec removes its subsections and a block's spec only its block (following-block directives stacked above it go with it).
- **Selection**: a group is affected only if some arm names a dimension the selection names; then conflicting arms go, one survivor becomes its content, and none removes the group (recorded). Only `@variant` groups are along dimensions. Selection and filter run in one walk; nothing inside removed content is reported.
- **Sections are found in the expanded list of siblings**, so an included heading ends or joins the section above the include, as the page reads (Q26).
- **Q81 to Q86** are recorded in `questions.md` with the reading implemented: problems in removed content (Q81), a spec that lists a target directly and through its dimension (Q82), several `@available` for one scope (Q83), a heading-bound directive whose heading `{heading=false}` left out (Q84), glossary details (Q85), a state with no version on a versioned target (Q86).
- **`Text` merging and the glossary** use the text node's span for every piece, because a decoded text can't be mapped back to the source exactly.

### Conformance

- `SKIPS.toml`: the `resolve` entry is removed; `tests/conformance/tests/adapters/resolve.rs` handles the tag (the resolved outline is built from the resolved tree: source text with substituted phrases, an `@available` shown with its resolved spec). It also answers a case's top-level `diagnostics` with the whole file-level check when the case doesn't carry `check` (`samples/include-and-selection`).
- `cargo test -p tessera-conformance --test conformance`: **330 passed, 0 failed, 33 skipped** (246 passed, 71 skipped before). All 33 skips are `page-check` (phase 14). Every `resolve` case that doesn't need `page-check` runs and passes; the 33 cases that carry `page-check` and expect builds (among them the `resolve` ones, and `directives/available/scope-*`) run in **`tests/conformance/tests/resolve_rows.rs`** (through the adapter: published pages, resolved outlines, assets, and the six rows resolution records), which is temporary. `source_index_rows.rs` stays too, because it checks `link-id-missing` and `link-id-in-fragment`, which the adapter doesn't produce; **phase 14 deletes both** when its adapter runs those cases whole.
- **A case fixed**: `glossary/text-is-not-rewritten-in-resolved-outline` listed only `index.md` under `pages`, but `glossary.md` is a page of the case; it's listed now (the set is compared exactly).
- **Harness test**: `bundled_samples_are_discovered_and_skipped_with_recorded_reasons` (now `bundled_cases_...`) used `samples/include-and-selection` and its unimplemented `resolve` tag; it uses `projects/quill`, skipped for `page-check`, with stand-ins for the tags that landed. Phase 14 will need to pick a case skipped for `output`.
- Two adapter helpers became `pub(super)` (`structure::attributes`, `inline::attributes`).

### Left open

- **Q81 to Q86** (above), none of which changes a case; all are implemented as proposed.
- **Interim route mapping**: see Router above.
- **For phase 14: content no build publishes.** Because a build records only problems about content it publishes (Q81), content that *no* build publishes (a model whose builds are all selections, with an arm none of them selects) is never checked at page level. Phase 14 needs to decide whether page-level problems there are reported: it could run one extra `switch` plus `badge` pass over every page for that, or warn that the content is never published.
- **Duplicate ids** (`id-duplicate`, `heading-duplicate-without-id`) are phase 14's, from `ResolvedPage::headings()`.
- **Performance**: `BuildResolver` resolves a page's target pages on demand and caches them (`page_id_of` uses a per-page table), but `Project::resolve_page` builds a fresh resolver each call and `Project::expand` isn't cached (phase 11's note). Phase 13 can cache per file; phase 26 measures.
- **Glossary matching** treats text nodes one at a time: a term split by markup (`**API** key`) isn't matched.
- **Definitions** aren't in the resolved tree (they were never blocks); a reference link's destination is resolved into the `Link` node.
