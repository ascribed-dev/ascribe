---
section: The language
---

**A JSON block names the operation it's an example of with an attribute in its info string, `request="<link>"` or `response="<link> <status>"`, whose link is an operation link (decision [#917b](917b-operation-links.md)), and its JSON is checked against that body's schema, each disagreement at its place in the block; the validator is the `boon` crate.**

````markdown
```json request="../api/openapi.yaml#createUser"
{ "name": "Ada" }
```

```json response="../api/openapi.yaml#createUser 201"
{ "id": "u_1", "name": "Ada" }
```
````

**The marker.** A fenced block whose language is `json` takes one of `request` or `response`, in its info string beside `title` and `tested`. The value starts with a link to an operation, in either form phase 2 allows, so it's resolved, checked, completed, hovered, and renamed as a link is. A response names its status after a space, as the spec's key is written (`201`, `2XX`, `default`). A media type follows after another space when the body has more than one JSON media type; otherwise the body's one JSON media type is used, `application/json` or any `…+json`. A directive around the block was the alternative, and isn't taken: it adds a container for one attribute, and the info string keeps the block a plain fenced block every Markdown viewer still shows.

**What's checked.** The block's JSON against the schema of the operation's request body, or of the response with that status, for its media type. The schema's `$ref`s into the same file are followed (`#/components/…` in OpenAPI 3, `#/definitions/…` in Swagger 2.0), with each reference's fragment percent-encoded first, since names hold `\` and spaces. OpenAPI 3.1's schemas are JSON Schema 2020-12. OpenAPI 3.0's and Swagger 2.0's are read as draft 4, which they're based on, after a translation: `nullable: true` and `x-nullable: true` add `null` to the type and to any `enum`, or, in a schema with no type, wrap it in `anyOf` with `{ "type": "null" }`; `type: file` is dropped; and a `$ref`'s siblings are ignored, as both specs and draft 4 say. In a request, a `readOnly` property isn't required, and in a response a `writeOnly` one isn't. JSON Schema's formats (`date-time`, `email`, `uri`, `uuid`, …) are asserted; OpenAPI's own (`int32`, `int64`, `float`, `double`, `byte`, `binary`, `password`) aren't. A `pattern` the validator's regular expressions can't read is left out, and said (below), so the rest of the schema is still checked. A schema with a `$ref` into another file isn't checked, and is said too: a multi-file spec is left out of the plan.

**Where a disagreement is.** `serde_json` keeps no positions, so the check builds a span map beside it: one pass over the block's text, each value's JSON pointer to its range and each member's key to its range. A validator's instance location is a JSON pointer, so a disagreement is put at its value; a property that isn't allowed is put at its key, and a missing required property at its object's opening brace. A block that isn't JSON is put where `serde_json` stopped, which it reports as a line and a column.

**The diagnostics,** each at its place, in area `examples`:

| Slug | Severity | When |
|---|---|---|
| `example-schema-mismatch` | error | The JSON disagrees with the schema, one per disagreement |
| `example-not-json` | error | A marked block isn't JSON, or isn't a `json` block |
| `example-body-missing` | error | The operation has no request body, no response with that status, no body of that media type, or more than one JSON media type and the marker names none |
| `example-schema-unsupported` | advice | Part of the schema can't be checked: a `pattern` the validator can't read, or a `$ref` into another file |

An operation the spec doesn't have is `link-operation-missing`, since the value is a link. Phase 5 gives them codes, messages, and fixes, and records them as SPEC §8.2's rows.

**The dependency** is `boon` 0.6, MIT or Apache-2.0, with no network or file loader: the spec is added as a resource named by a URN, and each body's schema is compiled from its JSON pointer in it (`urn:…#/paths/~1users/post/requestBody/content/application~1json/schema`), so the file's own references resolve with no wrapper. `jsonschema` 0.60, with its default features off, was measured first, and costs more than twice as much for the same answers. Phase 5 adds MIT-0 to `.cargo/deny.toml`'s list with it (below). If `boon` stops being maintained, Ascribe vendors it, as it carries `comrak-ascribe`.

**The validator is in the binary, behind one narrow seam.** A schema and JSON go in, positioned disagreements come out, and nothing else of Ascribe's is imported into it, whether it's a module of `ascribe-resolve` or a small crate of its own. It isn't an optional module: the rest of OpenAPI is a reader of `paths` on `serde_yaml_ng` and `serde_json`, already in the tree, and a project with no `[apis]` pays for the validator only in download size, never at run time. A Cargo feature would help only a source build, since one binary ships per platform, and a diagnostic behind a feature means the conformance suite runs in two configurations or the one that's off rots. A sidecar binary, as Vale runs through `ascribe-check`'s tool runner, costs four more platform packages per release, a protocol between two processes, and a child the language server keeps alive for completion, hover, and the checks as you type; Vale earns that boundary as Go with its own ecosystem, and a Rust crate Ascribe owns doesn't, not for 2 MB. The seam keeps lifting it into a sidecar mechanical, if the binary ever crosses a size that matters or a second heavy dependency arrives.

**The spike's numbers,** from a scratch crate outside the workspace, on 25 October 2026, against `main` at `2f20af2`, on a 4-core Linux container. The release profile is `main`'s (`lto = "fat"`, `codegen-units = 1`), and the validator is called from `ascribe`'s `main` on a schema read at run time, so nothing is optimized away:

| | `main` | with `boon` 0.6.1 | with `jsonschema` 0.60.1 |
|---|---|---|---|
| Release binary, stripped | 16.34 MB | 18.50 MB (+2.16 MB, +13%) | 21.51 MB (+5.17 MB, +32%) |
| Release binary, as built | 18.57 MB | 20.98 MB | 24.71 MB |
| Clean release build | 7 m 09 s | 7 m 40 s (+31 s) | 8 m 40 s (+91 s) |
| Crates the binary gains | | 41 | 44 |

One run each, so the times are within about half a minute. Both build for every release target: `cargo check` passes for `aarch64-apple-darwin`, `x86_64-pc-windows-msvc` with `crt-static`, `aarch64-unknown-linux-gnu`, and `x86_64-unknown-linux-gnu`, and neither tree has C code or a build script that links anything, so the zig builds for glibc 2.28 link nothing new. Licences: every new crate's licence is one `.cargo/deny.toml` allows, or offers one as a choice, except MIT-0. `cargo deny check licenses bans` on each tree rejects only MIT-0, `borrow-or-share` under `fluent-uri`, which isn't in `.cargo/deny.toml`'s list, so phase 5 adds it there; it warns of a second `fluent-uri`, beside `lsp-types`' own, and for `jsonschema` a second `syn` (2, beside the workspace's 3, through `strum_macros`).

Over the proposal's seeded APIs.guru sample (300 of 300 downloaded: 124 Swagger 2.0, 174 OpenAPI 3.0, 2 OpenAPI 3.1):

- **Schemas:** 22,642 JSON request and response bodies, 20,031 of them with a `$ref`. After the translation, the encoded references, and 46 patterns left out, `boon` compiles all 22,642, and so does `jsonschema`. Without the encoding, 17 bodies in one spec fail on a reference with `\` in it; without leaving patterns out, `boon` fails 102 in 11 specs, on ECMAScript patterns the `regex` crate can't read (lookbehind, `[[`, `\p{Alnum}`, and one past its size limit), and `jsonschema`, whose `fancy-regex` reads more of them, 8 in one. One spec, an Azure spec split across files, has a `$ref` into another file.
- **The translation:** 11,021 `nullable`, 4 `x-nullable`, and 29 `type: file`. Left as annotations: `example` in 9,510 schemas, `xml` in 2,753, `discriminator` in 135; `readOnly` in 2,400 (156 objects require a `readOnly` property, which the request rule above handles), and a `$ref` with siblings in 1,729.
- **Examples:** the specs' own JSON examples of a body, 2,228 of them. 1,805 agree with their schema. Of the 423 that don't, 269 are JSON written inside a string, 34 are `null`, and 120 are real disagreements: missing required properties, a number where the schema says string, `null` where it isn't allowed, a `date-time` with no time zone. None comes from the translation. The two validators agree on all but one, a `uri-template` format that draft 4 doesn't define.
- **Positions:** every disagreement, 922 of 922, has a value at its instance location in the span map, and the map's range for each of 119,622 values parses to the value `serde_json` read there, pretty-printed and on one line.
- **Time:** with either validator, compiling all 22,642 schemas takes about 2.5 s in all, `jsonschema`'s slowest one 26 ms, and validating the 2,228 examples 12 ms in all. A page's check compiles only the bodies its blocks name.

_Why:_ the info string keeps a marked example a plain code block, and its value is a link so it gets every link feature for free; `boon` gives the same answers as `jsonschema` on all but one of the corpus's examples for less than half of what it adds to the binary and the build, and compiles a schema straight from its place in the spec. `jsonschema` releases weekly and `boon` last released in January 2025, which is the trade: the check sits behind one module, so changing validators later touches nothing else. _From:_ the spike in [#920](https://github.com/ascribed-dev/ascribe/issues/920), [OpenAPI, phase 4](https://github.com/ascribed-dev/ascribe/blob/9162c25a/project-docs/openapi/phase-4-example-decision.md), and the maintainer on [#929](https://github.com/ascribed-dev/ascribe/pull/929), who chose `boon` and kept the validator in the binary behind its seam, 25 October 2026.
