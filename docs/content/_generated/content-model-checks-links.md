@memo: Generated from crates/ascribe-model/settings.toml by tests/conformance/tests/docs.rs; edit the registry, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-conformance --test docs` to rewrite it.
| Key | Type | Default | Description |
|---|---|---|---|
| `command` | string | `"lychee"` | The command that runs lychee. A path with a separator is from the project root; a bare name is looked up as the shell would. |
| `ignore` | array of strings | `[]` | The hosts whose links aren't checked: a host name, such as `example.com`, or `*.` and a host name, which covers every host under it but not the name itself. Case doesn't matter. |
| `reuse-days` | whole number | `14` | How many days an address that answered isn't asked about again. Each address's own limit falls between half of this and all of it, so addresses first checked together don't all come due on one day. `0` asks about every address on every run. |
| `confirm-days` | whole number | `3` | How many days a link that answered before and fails now is listed as unconfirmed, and not reported, before it's reported. A 404 or a 410 is reported at once. `0` reports every failure at once. |
