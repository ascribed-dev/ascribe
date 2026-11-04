@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/search.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
The fields of [`schemas/search.schema.json`]({repo}/blob/v{version}/schemas/search.schema.json):

**`SearchReport`**: What `ascribe search <QUERY>` answers: what `--format json` writes.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `schema_version` | `integer` | yes | no | The version of this schema. It changes only when a field is removed or changes meaning. |
| `ascribe_version` | `string` | yes | no | The release of Ascribe that wrote it, as `0.4.5`. |
| `query` | `string` | yes | no | The query, as given. |
| `builds` | `string[]` | yes | no | The builds searched, in the content model's order. |
| `places` | `object[]` | yes | no | The places, at most `shown` of them: titles, then headings, then the rest; in each, the places written in pages, then those in fragments and code files, each in path order and in order within a file. Empty with `--summary`. |
| `places[].file` | `string` | yes | no | The file, from the project root, as `ascribe check` reports it: a page, a fragment, a snippet's code file, or `ascribe.toml` for a phrase. |
| `places[].line` | `integer` | yes | no | The line, from 1. |
| `places[].column` | `integer` | yes | no | The column of the match's first character, from 1, in Unicode characters. |
| `places[].end_line` | `integer` | yes | yes | For a snippet's code that can't be lined up with its file, the last line the snippet shows: the place is its lines, from `line`, and not the line the match is on. `null` for every other place. |
| `places[].in` | `string` | yes | no | What it is: `title`, `description`, `heading`, `text`, `table`, `code`, or `phrase`, a phrase's value at its line in `ascribe.toml`. |
| `places[].text` | `string` | yes | no | The block's text as a reader reads it, or for a phrase, its value: whole when it's at most 240 characters, and otherwise cut to about that many around the match's first character, at words, with `…` where it's cut. `line` and `column` say where to read the rest. |
| `places[].match` | `integer[]` | yes | no | Where the match is in `text`: its first character and the one after its last, in Unicode characters; when `text` is cut before the match's last character, the end ends where it's cut, before the `…`. |
| `places[].through` | `SearchThrough[]` | yes | no | What the match reads through that the line doesn't spell, in order. |
| `places[].arm` | `string` | yes | yes | The arm of a group the block is written in, as its attributes are written (`deployment=cloud`), or else its title. |
| `places[].pages` | `SearchPage[]` | yes | no | The pages that show it, in path order, at most three. |
| `places[].pages_total` | `integer` | yes | no | How many pages show it; for a phrase, how many places in the source use it. |
| `places[].refs_command` | `string` | yes | yes | The `ascribe refs` command that lists where it's used, for a phrase, and for a fragment's place when `pages` was cut. |
| `pages` | `object[]` | yes | yes | With `--summary`, in place of the places: each page that shows a place and how many it shows, most first, then in path order; at most `shown` of them. `null` without it. |
| `pages[].page` | `string` | yes | no | The page, from the project root. |
| `pages[].places` | `integer` | yes | no | How many places it shows. |
| `pages_matched` | `integer` | yes | no | How many pages show a place. |
| `total` | `integer` | yes | no | How many places match. |
| `shown` | `integer` | yes | no | How many places are listed, or with `--summary`, pages. |
| `truncated` | `boolean` | yes | no | Whether the list was cut: `shown` is less than `total`, or with `--summary`, than `pages_matched`. |
| `next_command` | `string` | yes | yes | The command that lists them all, when the list was cut. |
| `source_places` | `object[]` | yes | no | When nothing matched, the first places the source writes the words as given, at most three, though no build searched publishes them there; empty when something matched, or when the words aren't written. |
| `source_places[].file` | `string` | yes | no | The file, from the project root. |
| `source_places[].line` | `integer` | yes | no | The line, from 1. |
| `not_searched_builds` | `string[]` | yes | no | When nothing matched, the builds that weren't searched that publish the words, in the content model's order; empty otherwise. |
| `stop_words` | `string[]` | yes | no | The query's stop words, as written: common English words, such as `how`, `do`, and `the`, that a place doesn't need to have. Empty when every word is one, since then a place needs them all. |
| `partial_places` | `object[]` | yes | no | When nothing matched and a place needs two or more words or phrases, the places that have some of them, at most 10: those that have the most first, then those with the most stop words, then in the order of `places`. Each is a place as in `places`, its `match` covering the words it has, with how many it `has` and what it `lacks`. Empty when something matched. |
| `partial_places[].file` | `string` | yes | no | The file, from the project root, as `ascribe check` reports it: a page, a fragment, a snippet's code file, or `ascribe.toml` for a phrase. |
| `partial_places[].line` | `integer` | yes | no | The line, from 1. |
| `partial_places[].column` | `integer` | yes | no | The column of the match's first character, from 1, in Unicode characters. |
| `partial_places[].end_line` | `integer` | yes | yes | For a snippet's code that can't be lined up with its file, the last line the snippet shows: the place is its lines, from `line`, and not the line the match is on. `null` for every other place. |
| `partial_places[].in` | `string` | yes | no | What it is: `title`, `description`, `heading`, `text`, `table`, `code`, or `phrase`, a phrase's value at its line in `ascribe.toml`. |
| `partial_places[].text` | `string` | yes | no | The block's text as a reader reads it, or for a phrase, its value: whole when it's at most 240 characters, and otherwise cut to about that many around the match's first character, at words, with `…` where it's cut. `line` and `column` say where to read the rest. |
| `partial_places[].match` | `integer[]` | yes | no | Where the match is in `text`: its first character and the one after its last, in Unicode characters; when `text` is cut before the match's last character, the end ends where it's cut, before the `…`. |
| `partial_places[].through` | `SearchThrough[]` | yes | no | What the match reads through that the line doesn't spell, in order. |
| `partial_places[].arm` | `string` | yes | yes | The arm of a group the block is written in, as its attributes are written (`deployment=cloud`), or else its title. |
| `partial_places[].pages` | `SearchPage[]` | yes | no | The pages that show it, in path order, at most three. |
| `partial_places[].pages_total` | `integer` | yes | no | How many pages show it; for a phrase, how many places in the source use it. |
| `partial_places[].refs_command` | `string` | yes | yes | The `ascribe refs` command that lists where it's used, for a phrase, and for a fragment's place when `pages` was cut. |
| `partial_places[].has` | `integer` | yes | no | How many of the words and phrases a place needs it has. |
| `partial_places[].lacks` | `string[]` | yes | no | The words and phrases it lacks, as the query writes them, a phrase in its quotes, in the query's order. |

**`SearchThrough`**: What a match reads through that its line doesn't spell. One of these options:

**`SearchThrough`** with `phrase`: A phrase's value, by its key.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `phrase` | `string` | yes | no |  |

**`SearchThrough`** with `title`: The title of a page a link with no text takes, or the text of the heading it names: `page.md` or `page.md#id`.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `title` | `string` | yes | no |  |

**`SearchThrough`** with `code`: A snippet's code: its address, as written (`code:app.py#main`).

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `code` | `string` | yes | no |  |

**`SearchPage`**: A page that shows a place.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `page` | `string` | yes | no | The page's content path. |
| `title` | `string` | yes | yes | Its title, as a reader reads it in the first build searched that publishes the page. |
| `heading` | `string` | yes | yes | The heading the place is under there, as a reader reads it; for a heading, the heading itself. |
| `id` | `string` | yes | yes | That heading's id. |
| `builds` | `string[]` | yes | no | The builds that show it there, in the content model's order. |
