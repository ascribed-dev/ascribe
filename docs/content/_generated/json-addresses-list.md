@memo: Generated from the Rust types by crates/ascribe-cli/src/shapes.rs, with schemas/addresses-list.schema.json; change the types, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes` to rewrite it.
The fields of [`schemas/addresses-list.schema.json`]({repo}/blob/v{version}/schemas/addresses-list.schema.json):

**`AddressesListReport`**: What `ascribe addresses list --format json` writes.

| Field | Type | Required | Nullable | Description |
| --- | --- | --- | --- | --- |
| `schema_version` | `integer` | yes | no | The version of this schema. It changes only when a field is removed or changes meaning. |
| `ascribe_version` | `string` | yes | no | The release of Ascribe that wrote it, as `0.4.5`. |
| `has_file` | `boolean` | yes | no | Whether the project has `ascribe.addresses`. Without it, every build's lists are empty. |
| `builds` | `object[]` | yes | no | Each build's redirects and retired addresses, as its `_ascribe/redirects.json` has them, in the content model's order. |
| `builds[].version` | `integer` | yes | no | The format's version. |
| `builds[].build` | `string` | yes | no | The build's name. |
| `builds[].redirects` | `object[]` | yes | no | Each pair, sorted by `from`. |
| `builds[].redirects[].from` | `string` | yes | no | The old address, from the host's root, in the published case and slash form. |
| `builds[].redirects[].to` | `string` | yes | no | Where it leads now, from the host's root. |
| `builds[].redirects[].status` | `integer` | yes | no | The HTTP status: 301. |
| `builds[].redirects[].entry` | `string` | yes | no | The `ascribe.addresses` key the pair comes from; or, for a pair a `[former]` setting makes, `former.` and the setting: `former.base-path`, `former.permalink-path`, or `former.agents`. |
| `builds[].gone` | `object[]` | yes | no | Each retired address, sorted by `path`. |
| `builds[].gone[].path` | `string` | yes | no | The address, from the host's root. |
| `builds[].gone[].reason` | `string` | yes | no | Why it was retired. |
| `builds[].gone[].entry` | `string` | yes | no | The `ascribe.addresses` key it comes from. |
| `builds[].gone[].page` | `string` | yes | no | Where the page saying so is, from the host's root: the base path, `GONE_FOLDER`, and the key's address from the base path, its page's for a Markdown version. |
