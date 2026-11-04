@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/explain.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
The fields of [`schemas/explain.schema.json`]({repo}/blob/v{version}/schemas/explain.schema.json):

**`ExplainReport`**: What `ascribe explain <CODE>` answers: one diagnostic.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `schema_version` | `integer` | yes | no | The version of this schema. It changes only when a field is removed or changes meaning. |
| `ascribe_version` | `string` | yes | no | The release of Ascribe that wrote it, as `0.4.5`. |
| `code` | `string` | yes | no | The code, such as `ASC036`. |
| `slug` | `string` | yes | no | The diagnostic's name, such as `link-target-missing`. |
| `severity` | `string` | yes | no | `error`, `warning`, or `advice`. |
| `next` | `string` | yes | yes | The kind of next step: `fix`, `choose`, `write`, `outside`, or `review`. Every diagnostic has one. |
| `level` | `string` | yes | no | `file` for a problem found in each file on its own; `page` for one found in each page as a build resolves it. |
| `message` | `string` | yes | no | The message, with its placeholders (`{path}`) as written. |
| `variants` | `object[]` | yes | no | The other ways the message is worded, by the case they're for, in name order. |
| `variants[].name` | `string` | yes | no | The case it's for. |
| `variants[].message` | `string` | yes | no | The message, with its placeholders as written. |
| `fix` | `string` | yes | yes | How to fix it, in general. Every diagnostic has one. |
| `docs` | `string` | yes | no | The address of its entry in the diagnostics reference: the docs' permalink for its code, such as `https://ascribed-dev.com/go/ASC036/`, which forwards to the entry wherever it is. |
| `example` | `object` | yes | yes | A page that has the problem and the same page fixed; `null` when the diagnostic has no example. |
| `example.wrong` | `string` | yes | no | The page, `page.md`, with the problem. |
| `example.right` | `string` | yes | no | The same page, fixed. |
| `example.model` | `string` | yes | yes | The content model the example assumes, when the problem depends on one: the example's own fragment, or else the explain model. `null` when it's a problem under any content model. |
| `example.files` | `object[]` | yes | no | The project's other files the example assumes, in path order. |
| `example.files[].path` | `string` | yes | no | Its content path. |
| `example.files[].text` | `string` | yes | no | Its text. |
