## Addresses file
@memo: Generated from tests/conformance/diagnostics.toml by tests/conformance/tests/docs.rs; edit the registry, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-conformance --test docs` to rewrite it.

### ASC254 `addresses-file-invalid`
@id {permalink=true}: ASC254
@summary: `ascribe.addresses` isn't valid TOML, or doesn't have the shape §7.10 gives it.

[Error](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [SPEC §7.10]({repo}/blob/main/SPEC.md#710-the-addresses-file)

**When:** `ascribe.addresses` isn't valid TOML, or doesn't have the shape §7.10 gives it.

**Message:** ascribe.addresses can't be read: \{detail}

**Fix:** ascribe.addresses is written by `ascribe rename`, the editor's moves, and `ascribe addresses`. Undo the change made to it by hand, other than to a retired address's reason, or, for a file from a newer version of Ascribe, update Ascribe. Don't delete the file to quiet this: it's what keeps the addresses the docs have published working, and the build writes no redirects without it.

**Read more:** [`ascribe.addresses`](../publishing/redirects.md#ascribeaddresses) in the guide to keeping addresses.

### ASC255 `address-lost`
@id {permalink=true}: ASC255
@summary: A page's address, or its Markdown version's, that the base published and nothing accounts for.

[Warning](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [SPEC §7.10]({repo}/blob/main/SPEC.md#710-the-addresses-file)

**When:** A page's address, or its Markdown version's, that the base published and nothing accounts for.

**Message:** \{source} was published at \{addresses}, and nothing publishes them now\{builds}

**Fix:** Decide what a reader who follows an old link should get. When the content is somewhere else now, forward it: `ascribe addresses move <old> <new>` records where it went, and the build writes the redirect. When it's gone on purpose, retire it: `ascribe addresses retire <old> --reason "…"`. When it was deleted by mistake, restore the file. Moving a page with `ascribe rename`, or in the editor, records the move for you. When a routing setting moved them, as `base-path`, `permalink-path`, `agents`, or a build's name, `ascribe check --fix` records the old value in `[former]`, and the build forwards every address it moved. The base is the default branch, or `--base`; see [Keep the addresses you've published](../publishing/redirects.md).

**Read more:** [What the check reports](../publishing/redirects.md#what-the-check-reports) in the guide to keeping addresses.

### ASC256 `anchor-lost`
@id {permalink=true}: ASC256
@summary: A heading's explicit id that the base published on a page still published, and nothing accounts for.

[Advice](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [SPEC §7.10]({repo}/blob/main/SPEC.md#710-the-addresses-file)

**When:** A heading's explicit id that the base published on a page still published, and nothing accounts for.

**Message:** the id `{id}` in \{path} was published at \{addresses}, and the page doesn't have it now\{builds}

**Fix:** Put the `@id` back on the heading that covers it now, so links to it keep working, or record where it went: `ascribe addresses move <file>#<old> <file>#<new>`, or `ascribe addresses retire <file>#<old> --reason "…"`. A link's fragment never reaches the host, so no redirect can follow it yet; the entry is kept for when one can.

**Read more:** [What the check reports](../publishing/redirects.md#what-the-check-reports) in the guide to keeping addresses.

### ASC258 `addresses-build-gone`
@id {permalink=true}: ASC258
@summary: A build the base has and the working tree doesn't, other than a line archived now.

[Advice](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [outside](../checks/diagnostics.md#what-to-do-next) · [SPEC §7.10]({repo}/blob/main/SPEC.md#710-the-addresses-file)

**When:** A build the base has and the working tree doesn't, other than a line archived now.

**Message:** the base has the build `{build}`, and ascribe.toml doesn't now, so its \{count} addresses aren't compared

**Fix:** Nothing to do when the build was removed on purpose: its addresses aren't compared, since builds are matched by name. A line `[releases]` archives isn't reported: `ascribe versions retire` records where its addresses go. When the build was renamed, its addresses are compared under its new name once the base has it.

**Read more:** [What the check reports](../publishing/redirects.md#what-the-check-reports) in the guide to keeping addresses.

### ASC265 `addresses-entry-not-a-page`
@id {permalink=true}: ASC265
@summary: A `[moved]` value that isn't a page, or a heading on one, in every build that published its key, or, for a permalink, a permalink the content declares.

[Error](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [choose](../checks/diagnostics.md#what-to-do-next) · [SPEC §7.10]({repo}/blob/main/SPEC.md#710-the-addresses-file)

**When:** A `[moved]` value that isn't a page, or a heading on one, in every build that published its key, or, for a permalink, a permalink the content declares.

**Message:** `{key}` is forwarded to `{value}`, which isn't a page of the project, so no build writes its redirect

**Fix:** Forward it to a page every build that published it publishes: `ascribe addresses move <old> <new>` replaces the entry. A permalink, `go:<name>`, can also be forwarded to another permalink, `go:<other>`, that a heading's `@id {permalink=true}` or a page's `permalink` declares. When nothing replaces it, retire it instead: `ascribe addresses retire <old> --reason "…"`. A fragment, a path outside the content root, or a page a build leaves out would send a reader to nothing.

**Read more:** [The file's checks](../publishing/redirects.md#the-files-checks) in the guide to keeping addresses.

### ASC266 `addresses-entry-live`
@id {permalink=true}: ASC266
@summary: A key of `[moved]` or `[retired]` whose source is back: a page, a heading's `@id`, or a permalink the content declares.

[Advice](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [fix](../checks/diagnostics.md#what-to-do-next) · [SPEC §7.10]({repo}/blob/main/SPEC.md#710-the-addresses-file)

**When:** A key of `[moved]` or `[retired]` whose source is back: a page, a heading's `@id`, or a permalink the content declares.

**Message:** `{key}` is a source of the project again, so its entry in [\{table}] does nothing; remove it

**Fix:** Remove the entry: the page, the heading's `@id`, or the permalink is back where its addresses lead, as after restoring a deleted page or a merge, and what the content declares wins. The quick fix and `ascribe check --fix` remove it. When the source came back by mistake, move it away again: for a permalink, remove the `@id {permalink=true}` or the page's `permalink` that declares it.

**Read more:** [The file's checks](../publishing/redirects.md#the-files-checks) in the guide to keeping addresses.

### ASC267 `addresses-entry-taken`
@id {permalink=true}: ASC267
@summary: A key of `[moved]` or `[retired]` whose address another source publishes now.

[Error](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [SPEC §7.10]({repo}/blob/main/SPEC.md#710-the-addresses-file)

**When:** A key of `[moved]` or `[retired]` whose address another source publishes now.

**Message:** \{address}, an address of `{key}`, is published by \{source} now, so a reader who follows an old link to `{key}` lands on another page

**Fix:** Give the page that took the address another path, so old links keep leading where `ascribe.addresses` says, or, when that page is what a reader of the old address wants, remove the entry. A case or slug collision, such as `Setup.md` and `setup.md`, takes an address the same way.

**Read more:** [The file's checks](../publishing/redirects.md#the-files-checks) in the guide to keeping addresses.

### ASC268 `addresses-entry-unpublished`
@id {permalink=true}: ASC268
@summary: An entry the base's `ascribe.addresses` doesn't have, whose key the base never published.

[Advice](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [fix](../checks/diagnostics.md#what-to-do-next) · [SPEC §7.10]({repo}/blob/main/SPEC.md#710-the-addresses-file)

**When:** An entry the base's `ascribe.addresses` doesn't have, whose key the base never published.

**Message:** the base never published `{key}`, so its entry in [\{table}] keeps no address; remove it

**Fix:** Remove the entry: it records a move or a removal of something that was never published, as moving a page made on the same branch leaves. The quick fix and `ascribe check --fix` remove it. An entry the base's file has already is never reported.

**Read more:** [The file's checks](../publishing/redirects.md#the-files-checks) in the guide to keeping addresses.

### ASC269 `addresses-key-twice`
@id {permalink=true}: ASC269
@summary: A key written twice in a table, or in both `[moved]` and `[retired]`.

[Error](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [choose](../checks/diagnostics.md#what-to-do-next) · [SPEC §7.10]({repo}/blob/main/SPEC.md#710-the-addresses-file)

**When:** A key written twice in a table, or in both `[moved]` and `[retired]`.

**Message:** `{key}` is in ascribe.addresses twice: \{first}, and \{second}

**Fix:** Keep one entry, and remove the other, as a merge of two branches can leave both. When both places are right for part of what the page had, keep the one most readers want, and forward the other's content's new home with `ascribe addresses move`. Until it's fixed, the file isn't used, and the build writes no redirects.

**Read more:** [The file's checks](../publishing/redirects.md#the-files-checks) in the guide to keeping addresses.

### ASC270 `addresses-chain`
@id {permalink=true}: ASC270
@summary: A `[moved]` value that's another entry's key.

[Advice](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [fix](../checks/diagnostics.md#what-to-do-next) · [SPEC §7.10]({repo}/blob/main/SPEC.md#710-the-addresses-file)

**When:** A `[moved]` value that's another entry's key.

**Message:** `{key}` is forwarded to `{value}`, which is forwarded to `{next}`

**Fix:** Forward it to where the chain ends, so each entry is one step: the quick fix and `ascribe check --fix` rewrite it, retiring it with the same reason when the chain ends at a retired entry. The build writes one redirect for each address either way. A cycle has no end: forward one of its entries to a page that's published.

**Read more:** [The file's checks](../publishing/redirects.md#the-files-checks) in the guide to keeping addresses.

### ASC271 `addresses-anchor-collision`
@id {permalink=true}: ASC271
@summary: An `#id` entry from a page that's gone whose id is live on the page its page's addresses lead to.

[Error](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [SPEC §7.10]({repo}/blob/main/SPEC.md#710-the-addresses-file)

**When:** An `#id` entry from a page that's gone whose id is live on the page its page's addresses lead to.

**Message:** `{key}` is forwarded to `{value}`, and a link to `#{id}` follows its page's redirect to \{page}, which has its own heading with the id `{id}`, so the link lands there

**Fix:** A link's fragment stays the same through a redirect, and a browser can't tell the old heading's id from the live one. Rename one of the two ids with `ascribe rename <file>#<id> <new>`, which records the move, so each id names one heading, or make the live heading the one the old id leads to.

**Read more:** [The file's checks](../publishing/redirects.md#the-files-checks) in the guide to keeping addresses.

### ASC272 `addresses-file-deleted`
@id {permalink=true}: ASC272
@summary: `ascribe.addresses` deleted while the base has it.

[Error](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [SPEC §7.10]({repo}/blob/main/SPEC.md#710-the-addresses-file)

**When:** `ascribe.addresses` deleted while the base has it.

**Message:** ascribe.addresses was deleted, and the base has it, with \{count}: the build no longer writes their redirects, and the check no longer compares the addresses

**Fix:** Restore the file, as `git checkout <base> -- ascribe.addresses` does. To stop keeping the addresses on purpose, delete it in a pull request of its own, whose check compares with its own head (`ascribe check --base HEAD`), so the decision is in the history.

**Read more:** [The file's checks](../publishing/redirects.md#the-files-checks) in the guide to keeping addresses.

### ASC273 `addresses-soft-404`
@id {permalink=true}: ASC273
@summary: Many addresses forwarded to one index page or the home page.

[Advice](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [SPEC §7.10]({repo}/blob/main/SPEC.md#710-the-addresses-file)

**When:** Many addresses forwarded to one index page or the home page.

**Message:** \{count} addresses are forwarded to \{value}, \{what}, so a reader who followed a link to one of them lands on a page without what they were looking for

**Fix:** Forward each to the page that has what it had, with `ascribe addresses move <old> <new>`, or retire the ones nothing replaces, with `ascribe addresses retire <old> --reason "…"`. A redirect to an index or the home page is a not-found page that answers 200, which search engines treat as one.

**Read more:** [The file's checks](../publishing/redirects.md#the-files-checks) in the guide to keeping addresses.

## APIs

### ASC261 `api-spec-unreadable`
@id {permalink=true}: ASC261
@summary: A declared spec that doesn't exist, can't be read, or isn't YAML or JSON.

[Error](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [SPEC §7.11]({repo}/blob/main/SPEC.md#711-apis)

**When:** A declared spec that doesn't exist, can't be read, or isn't YAML or JSON.

**Message:** the spec of API `{api}`, `{path}`, can't be read: \{reason}

**Fix:** Fix `spec`, which is relative to the folder ascribe.toml is in, or fix the file. A spec generated from code has to be on disk when Ascribe runs, as it does for the renderer: generate it first. Ascribe reads one file, in YAML or JSON, as the renderer is given it: for a spec split across files, point `spec` at the bundled one.

**Read more:** [`[apis.<name>]`](../content-model/sources.md#21-apisname) in the `ascribe.toml` reference.

### ASC262 `api-spec-empty`
@id {permalink=true}: ASC262
@summary: A declared spec with no operations.

[Advice](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [outside](../checks/diagnostics.md#what-to-do-next) · [SPEC §7.11]({repo}/blob/main/SPEC.md#711-apis)

**When:** A declared spec with no operations.

**Message:** the spec of API `{api}`, `{path}`, has no operations, so no link can name one

**Fix:** Check that `spec` names the spec you mean, and that it has a `paths` object with operations in it. A path item that's a `$ref` isn't followed: point `spec` at the bundled spec your renderer is given.

**Read more:** [`[apis.<name>]`](../content-model/sources.md#21-apisname) in the `ascribe.toml` reference.

## Examples

### ASC277 `api-example-schema-mismatch`
@id {permalink=true}: ASC277
@summary: A marked example that disagrees with its body's schema.

[Error](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [SPEC §7.11]({repo}/blob/main/SPEC.md#711-apis)

**When:** A marked example that disagrees with its body's schema.

**Message:** \{found}; \{body} says \{expected}

**Fix:** Change the example to agree with the schema, or, when the API changed and the example is right, change the spec, or the code it's generated from. Each disagreement is reported at its place: the value, a property the schema doesn't allow, or the opening brace of an object missing a required property.

**Read more:** [What's checked](../openapi/examples.md#whats-checked) in the guide to checked examples.

### ASC278 `api-example-not-json`
@id {permalink=true}: ASC278
@summary: A marked example that isn't JSON, or isn't a `json` block.

[Error](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [SPEC §7.11]({repo}/blob/main/SPEC.md#711-apis)

**When:** A marked example that isn't JSON, or isn't a `json` block.

**Message:** the example of \{body} isn't JSON: \{reason}

**Fix:** Fix the JSON where reading it stopped: a trailing comma, a comment, a key without quotes, and text after the value are all errors in JSON. An example in another language, YAML or JSON with comments, isn't checked: take its marker off.

**Read more:** [Mark an example](../openapi/examples.md#mark-an-example) in the guide to checked examples.

### ASC279 `api-example-body-missing`
@id {permalink=true}: ASC279
@summary: A marked example whose operation has no body it names.

[Error](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [write](../checks/diagnostics.md#what-to-do-next) · [SPEC §7.11]({repo}/blob/main/SPEC.md#711-apis)

**When:** A marked example whose operation has no body it names.

**Message:** `{operation}` has no request body in `{path}`

**Fix:** Name a body the operation has: for a request, an operation that takes one; for a response, its status as the spec writes it (`201`, `2XX`, `default`). When the body has more than one JSON media type, write the one meant after the status, or after the link for a request: `response="openapi.yaml#getUser 200 application/problem+json"`.

**Read more:** [Mark an example](../openapi/examples.md#mark-an-example) in the guide to checked examples.

### ASC280 `api-example-schema-unsupported`
@id {permalink=true}: ASC280
@summary: A marked example whose schema can't all be checked.

[Advice](../checks/diagnostics.md#severity) · [file level](../checks/diagnostics.md#file-level-and-page-level) · next step: [outside](../checks/diagnostics.md#what-to-do-next) · [SPEC §7.11]({repo}/blob/main/SPEC.md#711-apis)

**When:** A marked example whose schema can't all be checked.

**Message:** the schema of \{body} has a pattern the validator can't read, `{pattern}`, so it's left out, and the rest of the schema is checked

**Fix:** A pattern is read as a Rust regular expression, which has no lookaround or backreferences: write one it can read, or leave it, and the rest of the schema is still checked. A spec split across files is declared by its bundled file, the one its renderer is given.

**Read more:** [What isn't checked](../openapi/examples.md#what-isnt-checked) in the guide to checked examples.
