@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/diff.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
The fields of [`schemas/diff.schema.json`]({repo}/blob/v{version}/schemas/diff.schema.json):

**`DiffReport`**: The whole report, as `ascribe diff --format json` writes it.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `schema_version` | `integer` | yes | no | The version of this schema. It changes only when a field is removed or changes meaning. |
| `ascribe_version` | `string` | yes | no | The release of Ascribe that wrote it, as `0.4.5`. |
| `base` | `object` | yes | no | What was compared with. |
| `base.requested` | `string` | yes | no | The revision asked for, or the default branch used. |
| `base.commit` | `string` | yes | no | The commit it names. |
| `base.merge_base` | `string` | yes | yes | The merge base of that commit and `HEAD`, which the comparison reads; `null` with `--base-exact`, which reads `commit`. |
| `repository` | `object` | yes | no | Where the project is. |
| `repository.root` | `string` | yes | no | The repository's top-level folder, as git prints it. |
| `repository.project_prefix` | `string` | yes | no | The project's folder inside it, with a trailing `/`, or empty when the project is at the repository's root. |
| `working_tree_errors` | `integer` | yes | no | How many errors `ascribe check` finds in the working tree, for the builds compared. The comparison runs regardless, but a page with an error may not render as it will once it's fixed, so a reviewer should know. `diff_project` sets it; `Report::new` leaves it zero. |
| `builds` | `object[]` | yes | no | What changed, per build, in the order asked for. |
| `builds[].build` | `string` | yes | no | The build's name. |
| `builds[].pages` | `object[]` | yes | no | The pages that changed, in path order. |
| `builds[].pages[].path` | `string` | yes | no | The page's content path. |
| `builds[].pages[].route` | `string` | yes | no | Its route: the new one, or for a removed page the old one. |
| `builds[].pages[].status` | `PageStatus` | yes | no | Whether the build publishes it only now, only before, or in both with a difference. |
| `builds[].pages[].own_file_changed` | `boolean` | yes | no | Whether the page's own file changed (or exists on one side only). |
| `builds[].pages[].because` | `string[]` | yes | no | The other changed files the page's change can come from: fragments it includes and pages its links take a title or a heading from, in path order; then the snippets whose code changed, by address (`code:app.py#main`); then the operations of a declared API whose spec changed what the page writes for them, an `@operation`'s method, path, summary, or deprecation, or a link's address, each by its spec's path from the project root, `#`, and the fragment the page names it by (`api/openapi.yaml#createWidget`); and `ascribe.toml` (last) when the content model is a cause. The kinds are told apart by their form: a content file's path ends in `.md`, a snippet's address has its source's name and `:` before its path, and an operation's has a `#` and neither. |
| `builds[].pages[].page_changed` | `string[]` | yes | no | What changed about the page itself besides its blocks, in this order: `title`, `frontmatter`, `availability` (the page-level one), and `route`. Empty for an added or removed page. A moved page's route moves with it, so it isn't listed for one. |
| `builds[].pages[].counts` | `object` | yes | no | How many changes of each kind. |
| `builds[].pages[].counts.changed` | `integer` | yes | no | Blocks whose content changed. |
| `builds[].pages[].counts.added` | `integer` | yes | no | Blocks only in the new version. |
| `builds[].pages[].counts.removed` | `integer` | yes | no | Blocks only in the old version. |
| `builds[].pages[].counts.moved` | `integer` | yes | no | Blocks in both, somewhere else. |
| `builds[].pages[].changes` | `object[]` | yes | no | The block-level changes, in the page's order, a removed block where it was. Empty for an added or removed page, and for every page when the report's `blocks_omitted` is `true`. |
| `builds[].pages[].changes[].kind` | `ChangeKind` | yes | no | What happened to it. |
| `builds[].pages[].changes[].after` | `Anchor` | no | no | For a removed block, and a moved block's old place: the block of the new version it came after, among its siblings. Absent when it was first. |
| `builds[].pages[].changes[].now` | `Anchor` | no | no | Where it's written now: every kind but `removed`. |
| `builds[].pages[].changes[].parent` | `Anchor` | no | no | For a removed block, and a moved block's old place: the block of the new version it was inside. Absent at the top of the page. |
| `builds[].pages[].changes[].text` | `string` | no | no | For a removed block, its text, whitespace collapsed, so it can be shown where it was. |
| `builds[].pages[].changes[].was` | `Anchor` | no | no | Where it was written: every kind but `added`. |
| `builds[].pages[].changes[].words` | `object` | no | no | For changed prose, the words that differ. |
| `builds[].pages[].changes[].words.now` | `integer[][]` | yes | no | Ranges in `now_text`: words added or replacing others. |
| `builds[].pages[].changes[].words.was` | `integer[][]` | yes | no | Ranges in `was_text`: words removed or replaced. |
| `builds[].pages[].changes[].words.now_text` | `string` | yes | no | The block's text now. |
| `builds[].pages[].changes[].words.was_text` | `string` | yes | no | The block's text before. |
| `builds[].pages[].moved_from` | `string` | no | no | For a page that moved, the content path it was at: git sees its file renamed, or `ascribe.addresses` records the move. The page is listed once, at `path`, compared with the page as it was at `moved_from`; `status` is `changed`, and its `counts` and `changes` are what differs besides the move, none for a page that only moved. Absent for a page that didn't move. |
| `builds[].pages[].moved_links` | `object[]` | no | no | The pages among `because` whose only change is that they moved, each by where it was and where it is: a link on this page leads to its new address. Absent when there's none. |
| `builds[].pages[].moved_links[].from` | `string` | yes | no | Where it was, in the base. |
| `builds[].pages[].moved_links[].to` | `string` | yes | no | Where it is now. |
| `blocks_omitted` | `boolean` | no | no | Whether every page's `changes` was left empty, as `ascribe diff --pages-only` leaves them: its `counts` still count them. Only there when it's `true`. |

**`PageStatus`**: Whether a page is new, gone, or different. One of:

| Value | Description |
| --- | --- |
| `"added"` | The build publishes it now, and didn't before. |
| `"removed"` | The build published it before, and doesn't now. |
| `"changed"` | The build publishes it in both, and it differs. |

**`ChangeKind`**: The kinds of block change. One of:

| Value | Description |
| --- | --- |
| `"changed"` | The block's content changed. |
| `"added"` | The block is new. |
| `"removed"` | The block is gone. |
| `"moved"` | The same block is somewhere else. |

**`Anchor`**: Where a block's text is written: the README's anchor grammar, the same string the rendered page carries in `data-ascribe-source` and `data-ascribe-via`.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `source` | `string` | yes | no | `<path>:<first>-<last>`: the file's content path, percent-encoded by segment, and the block's first and last lines, from 1. |
| `via` | `string[]` | yes | no | The includes the block came through, outermost first, each `<path>:<line>`. Empty for a block written in the page itself. |
