<!-- Generated from tests/conformance/diagnostics.toml by tests/conformance/tests/docs.rs. Edit the registry, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-conformance --test docs`. -->

## Retired

No longer reported. Their codes and names aren't reused, and their permalinks still lead here.

### ASC039 `link-id-in-fragment`
@id {permalink=true}: ASC039

A link can name a heading from a fragment the target page includes (SPEC §4.2), so this is no longer an error. A heading a build removes is `link-id-removed`.

### ASC051 `content-type-unresolved`
@id {permalink=true}: ASC051

A page names its content type in its frontmatter, with `type` (SPEC §7.2), so it can't match several types, or none. A page that names none is `content-type-missing`, and one that names a type that isn't declared is `content-type-unknown`.

### ASC071 `model-type-multiple-defaults`
@id {permalink=true}: ASC071

A page names its content type in its frontmatter (SPEC §7.2), and a type has no `default`. `default` in `[types.<name>]` is `model-unknown-key`.

### ASC072 `model-type-unreachable`
@id {permalink=true}: ASC072

A page names its content type in its frontmatter (SPEC §7.2), so any declared type can be used, and a type has no `files`. `files` in `[types.<name>]` is `model-unknown-key`.
