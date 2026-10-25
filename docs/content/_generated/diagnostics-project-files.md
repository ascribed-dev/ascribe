## Addresses file
@memo: Generated from tests/conformance/diagnostics.toml by tests/conformance/tests/docs.rs; edit the registry, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-conformance --test docs` to rewrite it.

### ASC254 `addresses-file-invalid`
@id {permalink=true}: ASC254
@summary: `ascribe.addresses` isn't valid TOML, or doesn't have the shape §7.10 gives it.

Error · file level · next step: write · [SPEC §7.10]({repo}/blob/main/SPEC.md#710-the-addresses-file)

**When:** `ascribe.addresses` isn't valid TOML, or doesn't have the shape §7.10 gives it.

**Message:** ascribe.addresses can't be read: \{detail}

**Fix:** ascribe.addresses is written by `ascribe rename`, the editor's moves, and `ascribe addresses`. Undo the change made to it by hand, other than to a retired address's reason, or, for a file from a newer version of Ascribe, update Ascribe. Don't delete the file to quiet this: it's what keeps the addresses the docs have published working, and the build writes no redirects without it.

**Read more:** [`ascribe.addresses`](../publishing/redirects.md#ascribeaddresses) in the guide to keeping addresses.

### ASC255 `address-lost`
@id {permalink=true}: ASC255
@summary: A page's address, or its Markdown version's, that the base published and nothing accounts for.

Warning · file level · next step: write · [SPEC §7.10]({repo}/blob/main/SPEC.md#710-the-addresses-file)

**When:** A page's address, or its Markdown version's, that the base published and nothing accounts for.

**Message:** \{source} was published at \{addresses}, and nothing publishes them now\{builds}

**Fix:** Decide what a reader who follows an old link should get. When the content is somewhere else now, forward it: `ascribe addresses move <old> <new>` records where it went, and the build writes the redirect. When it's gone on purpose, retire it: `ascribe addresses retire <old> --reason "…"`. When it was deleted by mistake, restore the file. Moving a page with `ascribe rename`, or in the editor, records the move for you. The base is the default branch, or `--base`; see [Keep the addresses you've published](../publishing/redirects.md).

**Read more:** [What the check reports](../publishing/redirects.md#what-the-check-reports) in the guide to keeping addresses.

### ASC256 `anchor-lost`
@id {permalink=true}: ASC256
@summary: A heading's explicit id that the base published on a page still published, and nothing accounts for.

Advice · file level · next step: write · [SPEC §7.10]({repo}/blob/main/SPEC.md#710-the-addresses-file)

**When:** A heading's explicit id that the base published on a page still published, and nothing accounts for.

**Message:** the id `{id}` in \{path} was published at \{addresses}, and the page doesn't have it now\{builds}

**Fix:** Put the `@id` back on the heading that covers it now, so links to it keep working, or record where it went: `ascribe addresses move <file>#<old> <file>#<new>`, or `ascribe addresses retire <file>#<old> --reason "…"`. A link's fragment never reaches the host, so no redirect can follow it yet; the entry is kept for when one can.

**Read more:** [What the check reports](../publishing/redirects.md#what-the-check-reports) in the guide to keeping addresses.

### ASC258 `addresses-build-gone`
@id {permalink=true}: ASC258
@summary: A build the base has and the working tree doesn't.

Advice · file level · next step: outside · [SPEC §7.10]({repo}/blob/main/SPEC.md#710-the-addresses-file)

**When:** A build the base has and the working tree doesn't.

**Message:** the base has the build `{build}`, and ascribe.toml doesn't now, so its \{count} addresses aren't compared

**Fix:** Nothing to do when the build was retired on purpose, as an older line is when it leaves the window: its addresses aren't compared, since builds are matched by name. When the build was renamed, its addresses are compared under its new name once the base has it.

**Read more:** [What the check reports](../publishing/redirects.md#what-the-check-reports) in the guide to keeping addresses.

## APIs

### ASC261 `api-spec-unreadable`
@id {permalink=true}: ASC261
@summary: A declared spec that doesn't exist, can't be read, or isn't YAML or JSON.

Error · file level · next step: write · [SPEC §7.11]({repo}/blob/main/SPEC.md#711-apis)

**When:** A declared spec that doesn't exist, can't be read, or isn't YAML or JSON.

**Message:** the spec of API `{api}`, `{path}`, can't be read: \{reason}

**Fix:** Fix `spec`, which is relative to the folder ascribe.toml is in, or fix the file. A spec generated from code has to be on disk when Ascribe runs, as it does for the renderer: generate it first. Ascribe reads one file, in YAML or JSON, as the renderer is given it: for a spec split across files, point `spec` at the bundled one.

**Read more:** [`[apis.<name>]`](../content-model/checks.md#23-apisname) in the `ascribe.toml` reference.

### ASC262 `api-spec-empty`
@id {permalink=true}: ASC262
@summary: A declared spec with no operations.

Advice · file level · next step: outside · [SPEC §7.11]({repo}/blob/main/SPEC.md#711-apis)

**When:** A declared spec with no operations.

**Message:** the spec of API `{api}`, `{path}`, has no operations, so no link can name one

**Fix:** Check that `spec` names the spec you mean, and that it has a `paths` object with operations in it. A path item that's a `$ref` isn't followed: point `spec` at the bundled spec your renderer is given.

**Read more:** [`[apis.<name>]`](../content-model/checks.md#23-apisname) in the `ascribe.toml` reference.
