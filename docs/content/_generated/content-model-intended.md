@memo: Generated from crates/ascribe-model/settings.toml by tests/conformance/tests/docs.rs; edit the registry, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-conformance --test docs` to rewrite it.
| Key | Type | Default | Description |
|---|---|---|---|
| `check` | string | **required** | The check, by its name. It must be a review check that reports its problems at an entry, as the [diagnostics reference](../checks/diagnostics.md) says. |
| `reason` | string | **required** | Why the problem is intended. Not empty. |
| `phrase` | string | required: exactly one of `phrase`, `feature`, and `image` | The entry, when it's a phrase: its key. |
| `feature` | string | required: exactly one of `phrase`, `feature`, and `image` | The entry, when it's a feature: its key. |
| `image` | string | required: exactly one of `phrase`, `feature`, and `image` | The entry, when it's an image: its path under the content root. |
