@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/report.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
The fields of [`schemas/report.schema.json`]({repo}/blob/v{version}/schemas/report.schema.json):

**`ReportReport`**: What `ascribe report --format json` writes: one document, whatever the outcome. A reader ignores fields it doesn't know.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `schema_version` | `integer` | yes | no | The version of this schema. It changes only when a field is removed or changes meaning. |
| `ascribe_version` | `string` | yes | no | The release of Ascribe that wrote it, as `0.4.5`. |
| `error` | `string` | yes | yes | Why the report couldn't be made (exit code 2), or `null`. When it isn't `null`, no section is there. |
| `sections` | `string[]` | yes | no | The sections asked for, in the order the report has them: `problems`, `inventory`, `builds`, `pages`, `links`, `delivery`. |
| `builds_checked` | `string[]` | yes | no | The builds `problems` checked and `builds` and `pages` report on, in `ascribe.toml`'s order. |
| `not_run` | `string[]` | yes | no | The sections asked for that couldn't run. Each says why in its own `not_run`. With `--exit-code`, any is exit code 2. |
| `summary` | `Summary` | yes | no | How many problems of each severity the sections that ran have, and of each kind of next step: the diagnostics of `problems`, `links`, and `delivery`. |
| `next_command` | `string` | yes | yes | The command that lists everything a list here leaves out, when one does; `null` otherwise. |
| `builds` | `object[]` | no | no | `builds`, when asked for: one for each build checked. |
| `builds[].build` | `string` | yes | no | The build. |
| `builds[].pages` | `object` | yes | no | The pages it doesn't publish, in path order. |
| `builds[].pages.items` | `object[]` | yes | no | The items, at most `--limit` of them. |
| `builds[].pages.items[].file` | `string` | yes | no | The page, relative to the project root, as a diagnostic's `file` is. |
| `builds[].pages.items[].kind` | `string` | yes | no | `availability` when its `available` frontmatter leaves it out, or `variant` when its `variant` frontmatter does. |
| `builds[].pages.items[].mark` | `string` | yes | no | That frontmatter's value, as written: `self-managed 3.5`, or `deployment=cloud` for `variant`, with a dimension's values joined by `\|`. |
| `builds[].pages.items[].kept_by` | `string[]` | yes | no | The builds that publish it. |
| `builds[].pages.shown` | `integer` | yes | no | How many are listed. |
| `builds[].pages.total` | `integer` | yes | no | How many there are. |
| `builds[].pages.truncated` | `boolean` | yes | no | Whether some are left out: `shown` is less than `total`. The report's `next_command` lists them all. |
| `builds[].content` | `object` | yes | no | The marks that take content out of the pages it publishes, in file and source order: each `@available` and variant arm once, however many blocks it takes out. |
| `builds[].content.items` | `object[]` | yes | no | The items, at most `--limit` of them. |
| `builds[].content.items[].file` | `string` | yes | no | The file the mark is written in: the page, or a fragment it includes. |
| `builds[].content.items[].range` | `Range` | yes | no | Where in the file: the `@available` line, the table row, or the variant arm. |
| `builds[].content.items[].page` | `string` | yes | no | The page it's taken out of, as `file` is. |
| `builds[].content.items[].heading` | `string` | yes | yes | The text of the heading the mark is under, or `null`: the section's own heading, for a section's `@available`. |
| `builds[].content.items[].kind` | `string` | yes | no | `variant` for a variant arm, or `availability` for content a feature's availability takes out. |
| `builds[].content.items[].scope` | `string` | yes | no | What the mark takes out: `section`, `block`, `row` (a table row), or `arm` (a variant arm). |
| `builds[].content.items[].why` | `string` | yes | no | Why, with the content model's labels: `Shows only os=linux`. |
| `builds[].content.items[].kept_by` | `string[]` | yes | no | The builds that keep it. |
| `builds[].content.shown` | `integer` | yes | no | How many are listed. |
| `builds[].content.total` | `integer` | yes | no | How many there are. |
| `builds[].content.truncated` | `boolean` | yes | no | Whether some are left out: `shown` is less than `total`. The report's `next_command` lists them all. |
| `delivery` | `object` | no | no | `delivery`, when asked for. |
| `delivery.not_run` | `NotRunJson` | yes | yes | Why it couldn't run, or `null` when it ran. When it isn't `null`, the rest is empty. |
| `delivery.site` | `string` | yes | no | The site checked, as `--site` gives it; empty when none was given. |
| `delivery.results` | `object[]` | yes | no | Each check's result, in the checker's order. |
| `delivery.results[].id` | `string` | yes | no | The check's id, such as `llms-txt-exists`. |
| `delivery.results[].category` | `string` | yes | no | Its category in the spec. |
| `delivery.results[].status` | `string` | yes | no | `pass`, `warn`, `fail`, `skip`, or `error`. |
| `delivery.results[].message` | `string` | yes | no | What the checker said. |
| `delivery.results[].owner` | `string` | yes | no | Who changes what it checks: `ascribe` (what Ascribe writes), `hosting` (where the site is hosted), or `pages` (the pages, which `ascribe check` reports on). |
| `delivery.diagnostics` | `Capped_Entry` | yes | no | A diagnostic for each check that failed or warned that the hosting or Ascribe owns. One a page owns is `ascribe check`'s, and isn't repeated. |
| `delivery.previous_run` | `string` | yes | yes | The day of the last report that checked the same site, `YYYY-MM-DD`, or `null` on a first run, with `--no-history`, or when the last report checked another site or none. |
| `delivery.new` | `integer` | yes | yes | How many checks fail or warn now that passed on the last report or weren't in it, or `null` when `previous_run` is. A check is told from another by the site's address and its id. |
| `delivery.resolved` | `integer` | yes | yes | How many checks that failed or warned on the last report pass now or aren't in this one, or `null` when `previous_run` is. |
| `delivery.new_checks` | `string[]` | yes | no | The ids of the checks `new` counts, in the checker's order; empty when `previous_run` is `null`. |
| `inventory` | `object` | no | no | `inventory`, when asked for. |
| `inventory.pages` | `integer` | yes | no | How many pages. |
| `inventory.by_type` | `object[]` | yes | no | How many pages of each type, most first. |
| `inventory.by_type[].type` | `string` | yes | yes | The type's name; `null` for the pages no one type applies to. |
| `inventory.by_type[].pages` | `integer` | yes | no | How many pages. |
| `inventory.by_owner` | `object[]` | yes | no | How many pages each owner has, most first: the value of the frontmatter field the page's type marks `role = "owner"`. |
| `inventory.by_owner[].owner` | `string` | yes | yes | The owner, as the frontmatter gives it; `null` for the pages without one. |
| `inventory.by_owner[].pages` | `integer` | yes | no | How many pages. |
| `inventory.overdue` | `Capped_Entry` | yes | no | The pages whose recheck date has passed (`recheck-overdue`). |
| `inventory.orphans` | `Capped_Entry` | yes | no | The pages nothing links to (`page-orphan`). |
| `inventory.unused` | `Capped_Entry` | yes | no | The fragments, phrases, features, glossary terms, and images nothing uses. |
| `inventory.apis` | `object[]` | no | no | How many of each declared API's operations the pages cover, for each API whose spec could be read, in the content model's order. Absent when the project declares no API. |
| `inventory.apis[].api` | `string` | yes | no | The API's name, its key in `[apis]`. |
| `inventory.apis[].operations` | `integer` | yes | no | How many operations its spec has. |
| `inventory.apis[].linked` | `integer` | yes | no | How many of them a link names, by `operationId` or by method and path. |
| `inventory.apis[].examples` | `integer` | yes | no | How many of them a checked example claims as its request or response. |
| `inventory.examples` | `object[]` | no | no | The tested examples the snippets show, for each source in the same repository with an `ascribe.examples.toml`, in the content model's order. Absent when no source has one. |
| `inventory.examples[].source` | `string` | yes | no | The source's name. |
| `inventory.examples[].snippets` | `integer` | yes | no | How many `@snippet` lines, in pages and fragments, show one of its examples. |
| `inventory.examples[].current` | `integer` | yes | no | How many of those show an example an observation is current for. |
| `inventory.examples[].stale` | `integer` | yes | no | How many show one that changed after every observation listing it. |
| `inventory.examples[].unobserved` | `integer` | yes | no | How many show one no observation lists. |
| `links` | `object` | no | no | `links`, when asked for. |
| `links.not_run` | `NotRunJson` | yes | yes | Why it couldn't run, or `null` when it ran. When it isn't `null`, the rest is empty. |
| `links.checked` | `integer` | yes | no | How many different addresses were checked: `asked` and `reused`. |
| `links.asked` | `integer` | yes | no | How many different addresses the link checker was asked about in this run. |
| `links.reused` | `integer` | yes | no | How many different addresses' answers were reused from an earlier run, within `[checks.links] reuse-days`. None of `diagnostics` is from a reused answer: only an answer with nothing to report is kept. |
| `links.oldest_answer` | `string` | yes | yes | The day of the oldest answer reused, `YYYY-MM-DD`, or `null` when none was. |
| `links.ignored` | `integer` | yes | no | How many `[checks.links] ignore` left out. |
| `links.diagnostics` | `Capped_Entry` | yes | no | A diagnostic for each link that moved or is broken, at its place, and for each acknowledgement of a broken link that covers nothing. |
| `links.acknowledged` | `Capped_AcknowledgedEntry` | yes | no | The broken links acknowledged as intended, confirmed or not. |
| `links.previous_run` | `string` | yes | yes | The day of the run the link history is from, `YYYY-MM-DD`, or `null` on a first run, or with `--no-history`. |
| `links.new` | `integer` | yes | yes | How many of `diagnostics` the previous run didn't report, or `null` when `previous_run` is. |
| `links.resolved` | `integer` | yes | yes | How many addresses the previous run reported that this one doesn't, fixed or no longer linked to, or `null` when `previous_run` is. |
| `links.unconfirmed` | `object` | yes | no | The broken links that answered before and aren't reported until they've kept failing for `[checks.links] confirm-days`. They aren't in `diagnostics` or the summary, and don't fail `--exit-code`. |
| `links.unconfirmed.items` | `object[]` | yes | no | The items, at most `--limit` of them. |
| `links.unconfirmed.items[].file` | `string` | yes | no | The file, as a diagnostic's `file` is. |
| `links.unconfirmed.items[].range` | `Range` | yes | no | Where in the file: the link. |
| `links.unconfirmed.items[].url` | `string` | yes | no | The link's address. |
| `links.unconfirmed.items[].status` | `string` | yes | no | What the link checker said: a status, such as `503 Service Unavailable`, or why it didn't answer. |
| `links.unconfirmed.items[].last_answered` | `string` | yes | no | The last day it answered, `YYYY-MM-DD`. |
| `links.unconfirmed.items[].failing_since` | `string` | yes | no | The day it started failing, `YYYY-MM-DD`. |
| `links.unconfirmed.items[].reported_from` | `string` | yes | no | The day it's reported from if it's still failing, `YYYY-MM-DD`. |
| `links.unconfirmed.shown` | `integer` | yes | no | How many are listed. |
| `links.unconfirmed.total` | `integer` | yes | no | How many there are. |
| `links.unconfirmed.truncated` | `boolean` | yes | no | Whether some are left out: `shown` is less than `total`. The report's `next_command` lists them all. |
| `pages` | `object[]` | no | no | `pages`, when asked for: one for each build checked. |
| `pages[].build` | `string` | yes | no | The build. |
| `pages[].published` | `integer` | yes | no | How many pages the build publishes. |
| `pages[].not_listed` | `integer` | yes | no | How many of them aren't `listed`: `unquoted` or `unlisted`. |
| `pages[].records` | `object` | yes | no | The records, sorted by `route`: every page's, or with `--not-listed` those that aren't `listed`. Each has the fields of its entry in the build's `_ascribe/pages.json` (output-layout contract §1.3), by the same names, with `route`, the key it's under there. |
| `pages[].records.items` | `object[]` | yes | no | The items, at most `--limit` of them. |
| `pages[].records.items[].route` | `string` | yes | no | The page's address, from the host's root, as the site output's links write it. |
| `pages[].records.items[].source` | `string` | yes | no | The page's source path, from the content root. |
| `pages[].records.items[].listing` | `string` | yes | no | `listed`, `unquoted`, or `unlisted`. |
| `pages[].records.items[].listing_from` | `object` | yes | no | Where `listing` comes from. |
| `pages[].records.items[].listing_from.level` | `string` | yes | no | `default` when nothing sets it, `page` for the page's own `listing`, or the level of the rule of `[listing]` it's under: `site`, `older-lines`, `folder`, or `type`. |
| `pages[].records.items[].listing_from.folder` | `string` | no | no | For a folder's rule, the folder, as `[listing.folders]` names it. |
| `pages[].records.items[].listing_from.line` | `integer` | no | no | For a rule, its line in `ascribe.toml`. |
| `pages[].records.items[].listing_from.rule` | `string` | no | no | For a rule, the rule as written in `ascribe.toml`. |
| `pages[].records.items[].listing_from.type` | `string` | no | no | For a type's rule, the type. |
| `pages[].records.items[].breadcrumbs` | `object[]` | yes | no | Its place in the site, from the top down to the page itself. |
| `pages[].records.items[].breadcrumbs[].route` | `string` | yes | no | The step's page's address, as `route` is. |
| `pages[].records.items[].breadcrumbs[].title` | `string` | yes | no | The step's page's title, as plain text. |
| `pages[].records.items[].canonical` | `string` | no | no | Its own address, absolute; absent without `[consumer] site`. |
| `pages[].records.items[].description` | `string` | no | no | Its description, as plain text on one line; absent when it has none. |
| `pages[].records.items[].markdown` | `string` | no | no | Its Markdown version's address, from the host's root; absent when the build publishes none. |
| `pages[].records.items[].title` | `string` | no | no | Its title, as plain text; absent when it has none. |
| `pages[].records.shown` | `integer` | yes | no | How many are listed. |
| `pages[].records.total` | `integer` | yes | no | How many there are. |
| `pages[].records.truncated` | `boolean` | yes | no | Whether some are left out: `shown` is less than `total`. The report's `next_command` lists them all. |
| `pages[].site` | `string` | no | no | The project's address, `[consumer] site`, as the build's `_ascribe/pages.json` has it; absent when it isn't set. |
| `problems` | `object` | no | no | `problems`, when asked for. |
| `problems.summary` | `Summary` | yes | no | How many errors, warnings, and advice, and of each kind of next step. |
| `problems.by_check` | `object` | yes | no | How many diagnostics each check reports, most first. |
| `problems.by_check.items` | `object[]` | yes | no | The items, at most `--limit` of them. |
| `problems.by_check.items[].code` | `string` | yes | no | The code, such as `ASC036`. |
| `problems.by_check.items[].slug` | `string` | yes | no | The check's name, such as `link-target-missing`. |
| `problems.by_check.items[].severity` | `string` | yes | no | `error`, `warning`, or `advice`. |
| `problems.by_check.items[].next` | `string` | yes | no | Its kind of next step, as a diagnostic's `next` is. |
| `problems.by_check.items[].count` | `integer` | yes | no | How many. |
| `problems.by_check.shown` | `integer` | yes | no | How many are listed. |
| `problems.by_check.total` | `integer` | yes | no | How many there are. |
| `problems.by_check.truncated` | `boolean` | yes | no | Whether some are left out: `shown` is less than `total`. The report's `next_command` lists them all. |
| `problems.by_file` | `object` | yes | no | How many diagnostics each file has, most first. |
| `problems.by_file.items` | `object[]` | yes | no | The items, at most `--limit` of them. |
| `problems.by_file.items[].file` | `string` | yes | no | The file, relative to the project root, as a diagnostic's `file` is. |
| `problems.by_file.items[].errors` | `integer` | yes | no | How many errors. |
| `problems.by_file.items[].warnings` | `integer` | yes | no | How many warnings. |
| `problems.by_file.items[].advice` | `integer` | yes | no | How many advice. |
| `problems.by_file.shown` | `integer` | yes | no | How many are listed. |
| `problems.by_file.total` | `integer` | yes | no | How many there are. |
| `problems.by_file.truncated` | `boolean` | yes | no | Whether some are left out: `shown` is less than `total`. The report's `next_command` lists them all. |
| `problems.acknowledged` | `Capped_AcknowledgedEntry` | yes | no | The problems acknowledged as intended, with their reasons, in file order. |
| `problems.list_command` | `string` | yes | no | The command that lists each problem: `ascribe check`, as JSON. |
| `problems.previous_run` | `string` | yes | yes | The day of the last report, `YYYY-MM-DD`, or `null` on a first run, with `--no-history`, or when the last report checked other builds. |
| `problems.new` | `integer` | yes | yes | How many problems the last report didn't list, or `null` when `previous_run` is. A problem is told from another by its check, its file, the text it's about, and its order among those alike. |
| `problems.resolved` | `integer` | yes | yes | How many problems the last report listed that this one doesn't, fixed or with the text they're about edited, or `null` when `previous_run` is. |
| `problems.new_items` | `Capped_Entry` | yes | no | The problems the last report didn't list, in the order `ascribe check` reports them; none when `previous_run` is `null`. |
| `problems.baselined` | `integer` | no | no | How many advice are baselined, listed in `ascribe.baseline`, which the counts leave out: the project's backlog. Left out when there are none, and with `--no-baseline`. |

**`Summary`**: How many problems of each severity, and of each kind of next step.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `errors` | `integer` | yes | no | How many errors. |
| `warnings` | `integer` | yes | no | How many warnings. |
| `advice` | `integer` | yes | no | How many advice. |
| `by_next` | `object[]` | yes | no | How many of each kind of next step, in the order `fix`, `choose`, `write`, `review`, `outside`; a kind with none is left out. |
| `by_next[].next` | `string` | yes | no | `fix`, `choose`, `write`, `review`, or `outside`, as a diagnostic's `next` is. |
| `by_next[].count` | `integer` | yes | no | How many. |

**`Range`**: A span of a file. `end` is just past its last character; an empty range (an insertion) has equal positions.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `start` | `Pos` | yes | no | Its first character. |
| `end` | `Pos` | yes | no | Just past its last character. |

**`NotRunJson`**: Why a section couldn't run, and what would let it.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `reason` | `string` | yes | no | Why, as a sentence. |
| `how` | `string` | yes | no | What to do so that it runs, as a sentence. |

**`Capped_Entry`**: A list of at most `--limit` items.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `items` | `object[]` | yes | no | The items, at most `--limit` of them. |
| `items[].code` | `string` | yes | no | The code, such as `ASC036`. |
| `items[].slug` | `string` | yes | no | The diagnostic's name, such as `link-target-missing`. |
| `items[].severity` | `string` | yes | no | `error`, `warning`, or `advice`. Advice never fails the command. More severities may be added; a reader treats one it doesn't know as it treats advice. |
| `items[].next` | `string` | yes | no | The kind of next step: `fix` when Ascribe can make the edit, `choose` when the writer picks among things Ascribe can list, `write` when it needs writing or judgment, `outside` when nothing in the source can fix it, and `review` when it may be fine as it is. |
| `items[].message` | `string` | yes | no | What's wrong, and what to do about it. |
| `items[].file` | `string` | yes | no | The file, relative to the project root (the folder of `ascribe.toml`), with `/` separators. `ascribe.toml` for a content-model problem. |
| `items[].range` | `Range` | yes | no | Where in the file. |
| `items[].related` | `object[]` | yes | no | Other places that explain it. |
| `items[].related[].file` | `string` | yes | no | The file, as a diagnostic's `file` is. |
| `items[].related[].range` | `Range` | yes | no | Where in the file. |
| `items[].related[].message` | `string` | yes | no | What it has to do with the diagnostic. |
| `items[].fixes` | `object[]` | yes | no | Edits that would fix it. |
| `items[].fixes[].title` | `string` | yes | no | What the fix does. |
| `items[].fixes[].file` | `string` | yes | no | The file the edits are in, as a diagnostic's `file` is. |
| `items[].fixes[].edits` | `object[]` | yes | no | The edits, each replacing the text of its range. |
| `items[].fixes[].edits[].range` | `Range` | yes | no | The text it replaces. |
| `items[].fixes[].edits[].new_text` | `string` | yes | no | The text that replaces it. |
| `items[].fixes[].applicability` | `string` | yes | no | `safe` when applying the edits as they are can't change what the page says and leaves nothing to decide; `unsafe` otherwise. |
| `items[].builds` | `string[]` | yes | no | The builds a page-level diagnostic appears in, in `ascribe.toml`'s order. Empty for a file-level diagnostic, and for one in content no build publishes. With `--build`, only that build. |
| `items[].unpublished` | `boolean` | yes | no | Whether it's in content that no build publishes. |
| `items[].help` | `string` | yes | no | How to fix it, in general: the diagnostics reference's advice for its code, in Markdown, whose links are the docs' permalinks, such as `https://ascribed-dev.com/go/22-checks/`. |
| `items[].docs` | `string` | yes | no | The address of its entry in the diagnostics reference: the docs' permalink for its code, such as `https://ascribed-dev.com/go/ASC036/`, which forwards to the entry wherever it is. |
| `items[].repeats` | `integer` | yes | no | For a problem in included content reported because one of its related places is in a path named: at how many other includes it's reported too, collapsed into this one. `0` otherwise. |
| `items[].rule` | `string` | no | no | The rule of the program that found it, such as `Ascribe.Repeated` from Vale, for a `prose` diagnostic. Absent for Ascribe's own. |
| `shown` | `integer` | yes | no | How many are listed. |
| `total` | `integer` | yes | no | How many there are. |
| `truncated` | `boolean` | yes | no | Whether some are left out: `shown` is less than `total`. The report's `next_command` lists them all. |

**`Capped_AcknowledgedEntry`**: A list of at most `--limit` items.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `items` | `object[]` | yes | no | The items, at most `--limit` of them. |
| `items[].code` | `string` | yes | no | The code of the check that found it, such as `ASC036`. |
| `items[].slug` | `string` | yes | no | The check's name. |
| `items[].message` | `string` | yes | no | What the check found. |
| `items[].file` | `string` | yes | no | The file, as a diagnostic's `file` is. |
| `items[].range` | `Range` | yes | no | Where in the file. |
| `items[].builds` | `string[]` | yes | no | The builds it appears in, as a diagnostic's `builds` are. |
| `items[].reason` | `string` | yes | no | Why it's intended: the acknowledgement's reason. |
| `items[].at` | `object` | yes | no | Where the acknowledgement is written. |
| `items[].at.file` | `string` | yes | no | The file, as a diagnostic's `file` is. |
| `items[].at.range` | `Range` | yes | no | Where in the file. |
| `shown` | `integer` | yes | no | How many are listed. |
| `total` | `integer` | yes | no | How many there are. |
| `truncated` | `boolean` | yes | no | Whether some are left out: `shown` is less than `total`. The report's `next_command` lists them all. |

**`Pos`**: A position in a file.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `line` | `integer` | yes | no | The line, from 1. |
| `column` | `integer` | yes | no | The column, from 1, in Unicode characters (not bytes or UTF-16 units). |
| `offset` | `integer` | yes | no | The byte offset from the start of the file. |
