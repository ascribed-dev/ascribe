@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/check.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
The fields of [`schemas/check.schema.json`]({repo}/blob/v{version}/schemas/check.schema.json):

**`CheckReport`**: What `ascribe check --format json` and `ascribe build --format json` write: one document, whatever the outcome. Fields can be added without a new `schema_version`, so a reader ignores fields it doesn't know.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `schema_version` | `integer` | yes | no | The version of this schema. It changes only when a field is removed or changes meaning. |
| `ascribe_version` | `string` | yes | no | The release of Ascribe that wrote it, as `0.4.5`. |
| `error` | `string` | yes | yes | Why the project couldn't be checked (exit code 2), or `null`. When it isn't `null`, `diagnostics` holds what was found first: the content model's problems. |
| `files_checked` | `integer` | yes | no | How many source files were checked: the project's. |
| `files_reported` | `integer` | yes | no | How many of them the report covers: the source files in the paths named, or every one when no path was. |
| `builds_checked` | `string[]` | yes | no | The builds whose page-level checks ran, in `ascribe.toml`'s order: every build, the ones named with `--build`, or the editor's with `--editor-build`. Empty when the project couldn't be checked. |
| `diagnostics` | `object[]` | yes | no | Every diagnostic, in file order, and in source order within a file; advice after the errors and warnings, in the same order. With paths, only those that count for them. With `--summary`, none: see `truncated`. |
| `diagnostics[].code` | `string` | yes | no | The code, such as `ASC036`. |
| `diagnostics[].slug` | `string` | yes | no | The diagnostic's name, such as `link-target-missing`. |
| `diagnostics[].severity` | `string` | yes | no | `error`, `warning`, or `advice`. Advice never fails the command. More severities may be added; a reader treats one it doesn't know as it treats advice. |
| `diagnostics[].next` | `string` | yes | no | The kind of next step: `fix` when Ascribe can make the edit, `choose` when the author picks among things Ascribe can list, `write` when it needs writing or judgment, `outside` when nothing in the source can fix it, and `review` when it may be fine as it is. |
| `diagnostics[].message` | `string` | yes | no | What's wrong, and what to do about it. |
| `diagnostics[].file` | `string` | yes | no | The file, relative to the project root (the directory of `ascribe.toml`), with `/` separators. `ascribe.toml` for a content-model problem. |
| `diagnostics[].range` | `Range` | yes | no | Where in the file. |
| `diagnostics[].related` | `object[]` | yes | no | Other places that explain it. |
| `diagnostics[].related[].file` | `string` | yes | no | The file, as a diagnostic's `file` is. |
| `diagnostics[].related[].range` | `Range` | yes | no | Where in the file. |
| `diagnostics[].related[].message` | `string` | yes | no | What it has to do with the diagnostic. |
| `diagnostics[].fixes` | `object[]` | yes | no | Edits that would fix it. |
| `diagnostics[].fixes[].title` | `string` | yes | no | What the fix does. |
| `diagnostics[].fixes[].file` | `string` | yes | no | The file the edits are in, as a diagnostic's `file` is. |
| `diagnostics[].fixes[].edits` | `object[]` | yes | no | The edits, each replacing the text of its range. |
| `diagnostics[].fixes[].edits[].range` | `Range` | yes | no | The text it replaces. |
| `diagnostics[].fixes[].edits[].new_text` | `string` | yes | no | The text that replaces it. |
| `diagnostics[].fixes[].applicability` | `string` | yes | no | `safe` when applying the edits as they are can't change what the page says and leaves nothing to decide; `unsafe` otherwise. |
| `diagnostics[].builds` | `string[]` | yes | no | The builds a page-level diagnostic appears in, in `ascribe.toml`'s order. Empty for a file-level diagnostic, and for one in content no build publishes. With `--build`, only that build. |
| `diagnostics[].unpublished` | `boolean` | yes | no | Whether it's in content that no build publishes. |
| `diagnostics[].help` | `string` | yes | no | How to fix it, in general: the diagnostics reference's advice for its code, in Markdown, whose links are the docs' permalinks, such as `https://ascribed-dev.com/go/22-checks/`. |
| `diagnostics[].docs` | `string` | yes | no | The address of its entry in the diagnostics reference: the docs' permalink for its code, such as `https://ascribed-dev.com/go/ASC036/`, which forwards to the entry wherever it is. |
| `diagnostics[].repeats` | `integer` | yes | no | For a problem in included content reported because one of its related places is in a path named: at how many other includes it's reported too, collapsed into this one. `0` otherwise. |
| `diagnostics[].rule` | `string` | no | no | The rule of the program that found it, such as `Ascribe.Repeated` from Vale, for a `prose` diagnostic. Absent for Ascribe's own. |
| `truncated` | `boolean` | yes | no | Whether `diagnostics` leaves some out. It does with `--summary`, which lists none. |
| `shown` | `integer` | yes | no | How many diagnostics `diagnostics` lists. |
| `total` | `integer` | yes | no | How many diagnostics there are. |
| `next_command` | `string` | yes | yes | The command that lists the ones left out, when `truncated`; `null` otherwise. |
| `summary` | `object` | yes | no | How many errors, warnings, and advice. |
| `summary.errors` | `integer` | yes | no | How many errors. |
| `summary.warnings` | `integer` | yes | no | How many warnings. |
| `summary.advice` | `integer` | yes | no | How many advice. |
| `summary.acknowledged` | `integer` | no | no | How many problems are acknowledged as intended. Left out when there are none. |
| `summary.baselined` | `integer` | no | no | How many problems `ascribe.baseline` holds back, which `diagnostics` leaves out and which are advice. Left out when there are none, and with `--no-baseline`. |
| `summary.by_code` | `object[]` | no | no | With `--summary`: how many diagnostics have each code, most first. |
| `summary.by_code[].code` | `string` | yes | no | The code, such as `ASC036`. |
| `summary.by_code[].slug` | `string` | yes | no | The diagnostic's name, such as `link-target-missing`. |
| `summary.by_code[].severity` | `string` | yes | no | `error`, `warning`, or `advice`. |
| `summary.by_code[].count` | `integer` | yes | no | How many. |
| `summary.by_file` | `object[]` | no | no | With `--summary`: how many diagnostics are in each file, most first. |
| `summary.by_file[].file` | `string` | yes | no | The file, as a diagnostic's `file` is. |
| `summary.by_file[].errors` | `integer` | yes | no | How many errors. |
| `summary.by_file[].warnings` | `integer` | yes | no | How many warnings. |
| `summary.by_file[].advice` | `integer` | yes | no | How many advice. |
| `acknowledged` | `object[]` | no | no | The problems acknowledged as intended, which `diagnostics` leaves out and which don't fail the command, in file order. Left out when there are none, and with `--summary`. |
| `acknowledged[].code` | `string` | yes | no | The code of the check that found it, such as `ASC036`. |
| `acknowledged[].slug` | `string` | yes | no | The check's name. |
| `acknowledged[].message` | `string` | yes | no | What the check found. |
| `acknowledged[].file` | `string` | yes | no | The file, as a diagnostic's `file` is. |
| `acknowledged[].range` | `Range` | yes | no | Where in the file. |
| `acknowledged[].builds` | `string[]` | yes | no | The builds it appears in, as a diagnostic's `builds` are. |
| `acknowledged[].reason` | `string` | yes | no | Why it's intended: the acknowledgement's reason. |
| `acknowledged[].at` | `object` | yes | no | Where the acknowledgement is written. |
| `acknowledged[].at.file` | `string` | yes | no | The file, as a diagnostic's `file` is. |
| `acknowledged[].at.range` | `Range` | yes | no | Where in the file. |
| `fixed` | `object[]` | no | no | With `--fix`, the safe fixes it applied, in file order and, within a file, in the order of their diagnostics; the rest of the report is the check after them. With `--dry-run`, the fixes it would apply, none of which was, and the check is of the files as they are. Left out when there's none. |
| `fixed[].code` | `string` | yes | no | The code of the diagnostic it fixed, such as `ASC036`. |
| `fixed[].slug` | `string` | yes | no | The diagnostic's name. |
| `fixed[].title` | `string` | yes | no | What the fix did. |
| `fixed[].file` | `string` | yes | no | The file it changed, as a diagnostic's `file` is. |

**`Range`**: A span of a file. `end` is just past its last character; an empty range (an insertion) has equal positions.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `start` | `Pos` | yes | no | Its first character. |
| `end` | `Pos` | yes | no | Just past its last character. |

**`Pos`**: A position in a file.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `line` | `integer` | yes | no | The line, from 1. |
| `column` | `integer` | yes | no | The column, from 1, in Unicode characters (not bytes or UTF-16 units). |
| `offset` | `integer` | yes | no | The byte offset from the start of the file. |
