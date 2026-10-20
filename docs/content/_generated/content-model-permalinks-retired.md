@memo: Generated from crates/ascribe-model/settings.toml by tests/conformance/tests/docs.rs; edit the registry, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-conformance --test docs` to rewrite it.
| Key | Type | Default | Description |
|---|---|---|---|
| `reason` | string | required | Why it was retired. A reader whose link leads there sees it, as text. Not empty. |
| `to` | string | | Where its address leads now: a page, by its path from the content root, and `#` and a heading's id, as `guides/security.md#rotate-keys`. Without it, or in a build that doesn't publish the page, the address shows that what was there was removed, the reason, and a link to the docs' home. |
