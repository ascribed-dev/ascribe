# Interpretations the cases lock in

SPEC.md is normative, and the cases in `cases/` were written from it by hand. Where the spec leaves something open, `project-docs/questions.md` has an entry (Q13 to Q26 are this suite's; all are resolved, and SPEC.md states them) and, while an entry is open, the cases that depend on it are tagged `provisional`. This file lists the other choices: readings the spec supports, but where a case had to pick one, so an implementer knows what the cases assume, and what the suite doesn't cover.

## Readings of the spec

**Recognition and attributes**

- The name is the whole run of lowercase letters, digits, and hyphens after `@`, and only a space, tab, `{`, `:`, or the end of the line may follow it (Appendix A's `directive-line`; the fork's rule). `@note.`, `@notes:`, `@note-x:`, and `@Note:` are text.
- A malformed attribute block is one diagnostic for the block, however many things are wrong in it (Q14 chooses which one).
- `@steps {…}` and other directives with no attributes report `attribute-unknown-key` (the registry's `none` variant).
- A directive indented less than a list item's content column is outside the item and, following a list item, is reported as `list-ended-by-directive`, the same as an unindented one (Q1).

**Structure**

- A directive's diagnostic is reported at its directive line, not its title line, except the title diagnostics, which are reported at the title line.
- `binding-no-block` is reported at the directive, when the container or document ends before a block. A directive followed by a heading is `binding-heading` only.
- An `@end` inside a container's own list item, blockquote, or arm is an ordinary end line for that container.
- A note in line form with a primary is `binding: self`; without one it is `following-block`.

**Directives**

- `@id`'s value is checked against letters, digits, hyphens, underscores, and periods (Q196: `my:id` and `a/b` are invalid; `3-steps`, `Setup-2`, `my_id`, and `v1.2` are valid).
- `@available` states may appear with no version (`cloud beta`, `self-managed sunset 3.1`); a history entry needs a version. A dimension name as an enclosing target covers all its values, and a name inside a single value exceeds it.
- `available-exceeds-scope` is page level, as the registry says, so its cases expect it under a build.
- A feature key is checked as the spec it stands for, including against its enclosing scope.
- `@variant` arm problems (`variant-unknown`, `variant-arm-kind`) are reported at the arm's opener; frontmatter `variant` problems at the key's line.
- Where a project widget's schema requires an attribute or a title and it's missing, the diagnostic is `widget-schema`; a wrong value is the general `attribute-type-mismatch`.

**Includes, ids, and links**

- Include and link paths resolve from the file they're written in; a bare name is never searched for (§4.2). Names match exactly, including case, on every platform, and a file outside the project root or in the output directory is reported as not existing (SPEC §9.4, Q10).
- Source ids are per file (§5.5): a link's `#id` names a heading of the target file, computed with duplicates numbered (`steps`, `steps-1`) and phrases substituted. Page ids appear on the expanded page, so a page-id such as `setup-1` is not a link target.
- A link to an empty-text target, a same-file `#id`, an external URL, and `mailto:` are valid. A link to a non-page local file is an asset link.
- `heading-duplicate-without-id` and `id-duplicate` are checked on the expanded page, so a fragment's headings count. They are reported once each, at the later occurrence (Q20).

**Builds**

- `switch` keeps everything; a selection acts only along the dimensions it names (§9.3). Arms that survive keep their own attributes.
- A build's `pages` are the pages it publishes, by source path, and never fragments. Its `assets` are the local files referenced by surviving content, once each, by source path (`../shared/logo.png` for a file outside the content root).
- A Markdown file outside the content root is an asset, not a page (`project-docs/contracts/assets.md` §1).
- Phrases in raw HTML, code spans, indented code, and fences without `phrases=true` are literal; a phrase's value is not scanned again.

## What the suite doesn't cover

- **Canonical form (§8.3).** Phase 23 added the `format` tag's cases (`cases/format/`) and the harness's formatting interface (`formatted`). Where §8.3 is silent, the cases are `provisional` on Q71 to Q77: errors and the formatter, trailing whitespace outside a container's colon, block quote and marker-line indentation, awkward blank-line gaps, attribute blocks with undeclared keys, image blocks, and title lines.
- **Outputs (§9.4).** The plain, site, and JSON outputs are out of scope for this phase; phases 18 and 20 add snapshots. So glossary links, rewritten link destinations, page ids in output, and availability rendering are not observed anywhere yet.
- **Registry-change reports (§5.1).** "When a key is added to the registry, the pages whose existing literal `{key}` text would change" is a report between two versions of a model, which a case can't state.
- **Phrases in frontmatter (§5.1).** Nothing in a resolved outline shows them.
- **Loader rules (content-model.md §20).** Only the one that is a §8.2 row (`model-name-multiple-roles`) has cases; phase 08 writes fixtures for the rest. `model-dimension-value-shared` overlaps that row (a value in two dimensions) and has no case here for that reason.
- **Editor features (§10).** Completion, hover, and refactoring are tested in phases 15, 16, and 24.
