@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/model.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
The fields of [`schemas/model.schema.json`]({repo}/blob/v{version}/schemas/model.schema.json):

**`ModelReport`**: What `ascribe model --format json` answers: the content model's sections, or the one asked for.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `schema_version` | `integer` | yes | no | The version of this schema. It changes only when a field is removed or changes meaning. |
| `ascribe_version` | `string` | yes | no | The release of Ascribe that wrote it, as `0.4.5`. |
| `apis` | `object[]` | no | no | APIs, in declaration order. |
| `apis[].name` | `string` | yes | no | Its name, as `@operation` and `api:` addresses write it. |
| `apis[].spec` | `string` | yes | no | The spec's file, from the folder `ascribe.toml` is in. |
| `apis[].reference` | `string` | yes | no | Where its reference is: the renderer's page, or an address pattern, as written. |
| `apis[].renderer` | `string` | no | no | The renderer its reference is published with, `redoc`; absent when `reference` is an address pattern. |
| `builds` | `object[]` | no | no | Builds, in declaration order, a versioned build as one for each live line, the current line first. |
| `builds[].name` | `string` | yes | no | Its name. |
| `builds[].variants` | `string` | yes | no | Which variant arms it keeps: `switch` for all of them, or the selection, such as `deployment=cloud`. |
| `builds[].availability` | `string` | yes | no | How it treats availability: `badge`, or the target it filters for, such as `filter self-managed 3.3`. |
| `builds[].editor` | `boolean` | yes | no | Whether it's the build the editor checks (`[editor] build`). |
| `builds[].line` | `string` | no | no | For a line of a versioned build, the line, as `[releases.<target>]` writes it. |
| `builds[].versions` | `string` | no | no | For a line of a versioned build, the target it's versioned by. |
| `checks` | `ModelSetting[]` | no | no | `[checks]`: each key of `[checks.vale]` and `[checks.links]`, with its value in effect, and each check the file sets, by its slug. |
| `consumer` | `ModelSetting[]` | no | no | `[consumer]`: each key, with its value in effect. |
| `crawlers` | `ModelSetting[]` | no | no | `[crawlers]`: each key, with its value in effect. |
| `dimensions` | `object[]` | no | no | Dimensions, in declaration order. |
| `dimensions[].name` | `string` | yes | no | Its name. |
| `dimensions[].label` | `string` | yes | no | Its label. |
| `dimensions[].values` | `object[]` | yes | no | Its values, in display order. |
| `dimensions[].values[].value` | `string` | yes | no | The value. |
| `dimensions[].values[].label` | `string` | yes | no | Its label. |
| `dimensions[].values[].versionless` | `boolean` | yes | no | Whether it has no versions, so an availability spec gives it a state but no version. |
| `editor` | `ModelSetting[]` | no | no | `[editor]`: each key, with its value in effect. |
| `features` | `object[]` | no | no | Features, in declaration order. |
| `features[].key` | `string` | yes | no | Its key, which an availability spec can name. |
| `features[].name` | `string` | yes | no | Its name. |
| `features[].availability` | `string` | yes | no | Its availability spec, as written. |
| `fragments` | `object` | no | no | `[fragments]`: the files that are fragments, and the fields a fragment's frontmatter may give. |
| `fragments.patterns` | `string[]` | yes | no | The patterns of the files that are fragments besides those whose path has a file or folder name starting with `_`, from the content root. |
| `fragments.fields` | `ModelField[]` | yes | no | The fields a fragment's frontmatter may give, in declaration order. |
| `from` | `object[]` | no | no | `[from]`: each file the project takes from, in the order `take` names them, with what it takes. |
| `from[].file` | `string` | yes | no | The file, from the folder `ascribe.toml` is in. |
| `from[].take` | `string[]` | yes | no | What it takes, as `take` names it: `releases`, `consumer.site`. |
| `frontmatter` | `ModelField[]` | no | no | `[frontmatter]`: the fields every page type has, as written, in declaration order. Each type's `fields` has them already. |
| `glossary` | `object[]` | no | no | Glossary terms, in the order of `[glossary] pages`, and on each page in document order. |
| `glossary[].id` | `string` | yes | no | Its id: its heading's id. |
| `glossary[].term` | `string` | yes | no | The term: its heading's text. |
| `glossary[].aliases` | `string[]` | yes | no | Other ways it's written. |
| `glossary[].match` | `string` | yes | no | Which occurrences are linked: `first` on each page, `every` one, or only those a writer links (`marked`). |
| `glossary[].case_sensitive` | `boolean` | yes | no | Whether an occurrence must match its case. |
| `glossary[].page` | `string` | yes | no | The file its heading is written in, from the content root: its glossary page, or a fragment the page includes. |
| `glossary[].line` | `integer` | yes | no | The heading's line in that file, from 1. |
| `images` | `ModelAttribute[]` | no | no | `[images.attributes]`: the keys an image's attribute block may give, in declaration order. |
| `intended` | `object[]` | no | no | `[[intended]]`: the acknowledged problems, in declaration order. |
| `intended[].check` | `string` | yes | no | The check, by its slug. |
| `intended[].kind` | `string` | yes | no | What it's about: `phrase`, `feature`, or `image`. |
| `intended[].name` | `string` | yes | no | Its key, or the image's path. |
| `intended[].reason` | `string` | yes | no | Why the problem is intended. |
| `lifecycle` | `object[]` | no | no | Lifecycle states: the built-in ones, as the project adjusts them, then the ones it declares. |
| `lifecycle[].name` | `string` | yes | no | Its name, which an availability spec gives. |
| `lifecycle[].label` | `string` | yes | no | Its label. |
| `lifecycle[].available` | `boolean` | yes | no | Whether content in it counts as available. |
| `lifecycle[].builtin` | `boolean` | yes | no | Whether it's built in, rather than declared by the project. |
| `listing` | `ModelSetting[]` | no | no | `[listing]`: each key, with its value in effect. |
| `navigation` | `ModelNavSection` | no | no | The project's navigation, its root section: every page that isn't `unlisted`, in order, as each build's `_ascribe/nav.json` has its share of it. In the whole report, and with `--section navigation`. |
| `notes` | `object[]` | no | no | Note types, the values of `@note`'s `type`: the built-in ones, as the project labels them, then the ones it declares. |
| `notes[].name` | `string` | yes | no | Its name, which `@note`'s `type` gives. |
| `notes[].label` | `string` | yes | no | Its label. |
| `notes[].builtin` | `boolean` | yes | no | Whether it's built in, rather than declared by the project. |
| `phrases` | `object[]` | no | no | Phrases, in declaration order. |
| `phrases[].key` | `string` | yes | no | Its key, written `{key}` in a page. |
| `phrases[].value` | `string` | yes | no | Its value. |
| `project` | `ModelSetting[]` | no | no | `[project]`: each key, with its value in effect. |
| `releases` | `object[]` | no | no | `[releases.<target>]`, in declaration order: each target's lines, with their status. |
| `releases[].target` | `string` | yes | no | The target: a versioned dimension value. |
| `releases[].lines` | `object[]` | yes | no | Its lines, newest first: planned, live, then archived. |
| `releases[].lines[].line` | `string` | yes | no | The line, as the list writes it. |
| `releases[].lines[].status` | `string` | yes | no | `current`, `live`, `planned`, or `archived`. |
| `releases[].lines[].builds` | `string[]` | yes | no | The builds that are this line, one for each build versioned by the target: `site@3.4`, or `site` for the current line. |
| `sources` | `object[]` | no | no | Sources, in declaration order. |
| `sources[].name` | `string` | yes | no | Its name, as addresses write it: `code:` for a source named `code`. |
| `sources[].path` | `string` | yes | no | Its folder, relative to the project's folder: the `path` written, or, for a source in another repository, the folder of its copies. |
| `sources[].include` | `string[]` | yes | no | The patterns of the files it makes readable, relative to its folder; empty for every file. |
| `sources[].ignore` | `string[]` | yes | no | The patterns of the files it doesn't make readable, though `include` matches them. |
| `sources[].git` | `object` | yes | yes | The repository it's in, for a source in another repository. |
| `sources[].git.url` | `string` | yes | no | Its URL. |
| `sources[].git.branch` | `string` | yes | yes | The branch an update follows; `null` for the repository's default. |
| `sources[].examples` | `object` | no | no | Its tested examples, for a source with an `ascribe.examples.toml`: absent for any other. For a source in another repository, they're read from its copies. |
| `sources[].examples.valid` | `boolean` | yes | no | Whether its `ascribe.examples.toml` can be read. One that can't makes no file an example. |
| `sources[].examples.files` | `integer` | yes | no | How many of the files the source makes readable its config makes examples. |
| `sources[].examples.observations` | `object[]` | yes | no | Its observations, in name order: those that can't be read left out. |
| `sources[].examples.observations[].name` | `string` | yes | no | Its name: `default`, or the name `ascribe observe --name` gave it. |
| `sources[].examples.observations[].environment` | `{ [name]: string }` | yes | no | Its environment as it has it, by name: `os`, `arch`, and each entry's kept line. |
| `sources[].examples.observations[].tested` | `string` | yes | no | What a page says it was tested with, by this observation alone. |
| `sources[].examples.observations[].listed` | `integer` | yes | no | How many files it lists. For a source in another repository, only those it has copies of, since the others weren't copied. |
| `sources[].examples.observations[].current` | `integer` | yes | no | How many of those are as it found them: the rest changed since, or are gone. |
| `types` | `object[]` | no | no | Page types, in declaration order: the implicit `page` type when none is declared. |
| `types[].name` | `string` | yes | no | Its name, which each page of the type gives as its `type`; the implicit `page` type's pages give none. |
| `types[].fields` | `ModelField[]` | yes | no | Its frontmatter fields, in declaration order. |
| `types[].guide` | `string[]` | no | no | What to read before writing a page of the type, in the order to read it, from the folder `ascribe.toml` is in, from the type's `guide` or the lock, as `template` is; absent when it has none. |
| `types[].purpose` | `string` | no | no | What a page of the type is for, as the content model says; absent when it says nothing. |
| `types[].template` | `string` | no | no | The page a page of the type starts from, from the folder `ascribe.toml` is in: the type's `template`, or the one `templates/templates.lock` gives a type that names neither a template nor a guide; absent when it has none. |
| `versions` | `ModelSetting[]` | no | no | `[versions]`: each key, with its value in effect. |
| `widgets` | `object[]` | no | no | Project widgets, in declaration order. |
| `widgets[].name` | `string` | yes | no | Its name, written `@name`. |
| `widgets[].description` | `string` | yes | yes | What it's for, as the content model describes it. |
| `widgets[].line` | `boolean` | yes | no | Whether it may be one line. |
| `widgets[].container` | `boolean` | yes | no | Whether it may be a container, closed by `@end`. |
| `widgets[].attributes` | `ModelAttribute[]` | yes | no | Its attributes, in canonical order. |

**`ModelSetting`**: A key of a section of settings, such as `[consumer]`, as the registry of `ascribe.toml`'s keys has it, with the value in effect.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `table` | `string` | yes | no | The table it's in, as `ascribe.toml` writes it: `consumer`, `checks.vale`. |
| `key` | `string` | yes | no | The key: in `checks`, a check's slug. |
| `kind` | `string` | yes | no | What its value is: `switch`, `choice`, `number`, `text`, `level`, `list`, or `table`. |
| `set` | `boolean` | yes | no | Whether `ascribe.toml` sets it. |
| `default` | `any` | no | no | The value used when the file leaves it out, when that's one value. |
| `default_text` | `string` | no | no | What's used when the file leaves it out, in the reference's words, when that isn't one value. |
| `from` | `string` | no | no | The file it's taken from, when `[from]` takes it: a path from the folder `ascribe.toml` is in. Its value is that file's, and it's `set`. |
| `value` | `any` | no | no | The value in effect: the file's, else the default. Absent when the file leaves it out and its default isn't one value. |

**`ModelField`**: A frontmatter field.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `name` | `string` | yes | no | Its name. |
| `type` | `string` | yes | no | Its type in short form: `string`, `list(string)`, `enum(a, b)`. |
| `required` | `boolean` | yes | no | Whether every page of the type must give it. |
| `values` | `string[]` | yes | no | The values it allows, for an enumeration or a list of one; empty otherwise. |
| `description` | `string` | yes | yes | What it's for, as the content model describes it. |
| `fields` | `ModelField[]` | yes | no | An object's fields; empty for any other type. |
| `declared` | `string` | no | no | Which table declares a page type's field: `frontmatter` for `[frontmatter]`, which every type shares, `type` for the type's own `frontmatter`, or `implicit` for the required string `title` neither declares. Absent on an object's fields. |

**`ModelAttribute`**: An attribute a widget accepts.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `key` | `string` | yes | no | The key. |
| `type` | `string` | yes | no | The value's type: `string`, `number`, `boolean`, `enum`, `set`, or `note-type`. |
| `values` | `string[]` | yes | no | The values it allows, for `enum` and a `set` of named values; empty otherwise. |
| `required` | `boolean` | yes | no | Whether every use must give it. |
| `default` | `string` | yes | yes | The value used when it's left out, as written (a set's members joined by `\|`). |
| `description` | `string` | yes | yes | What it's for. |

**`ModelNavSection`**: A section of the navigation: a folder and its pages, in order.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `folder` | `string` | yes | no | The folder, from the content root, ending in `/`; `""` for the root. |
| `label` | `string` | yes | no | Its label: from `ascribe.nav`, else its own page's title as plain text, else its folder's name with `-` and `_` as spaces and its first letter capitalized, and "Start" for the root. |
| `pages` | `ModelNavPage[]` | yes | no | Its pages in order, its own page not among them. |
| `sections` | `ModelNavSection[]` | yes | no | Its nested sections with a page, in order: each one whose own page is among `pages` right after that page, then the rest. |
| `listed` | `boolean` | yes | no | Whether `ascribe.nav` lists it. |
| `page` | `ModelNavPage` | no | no | Its own page: its `index.md`, or the page beside the folder with its name, which is then one of the parent's `pages` too. |

**`ModelNavPage`**: A page of the navigation.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `source` | `string` | yes | no | Its path from the content root. |
| `label` | `string` | yes | no | Its label as plain text: its `nav-label`, else its title, else its file name without `.md`. |
| `type` | `string` | no | no | Its page type's name, when it has one. |
