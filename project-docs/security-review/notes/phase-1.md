# Phase 1 notes: untrusted content

Hardening notes from [phase 1](../phase-1-content.md), reviewed on 27 October 2026 by one agent and checked by a second, adversarial one, each reproduction rebuilt from the first's description and run against the binary, the language server, and the MCP server. The three fuzz targets the phase adds (`frontmatter`, `openapi`, `example`) are in `fuzz/`, with their commit; each of the six targets ran about sixteen minutes, and the new ones found nothing.

**Two private findings exist, both high** (README decision 1): one is a read outside the project that reaches the outputs, one is a crash of the long-running servers. Each is with the maintainer with its input, and each gets a conformance case with its fix. The notes below are the rest.

## H1: the parser's inline spans stop covering their source on a destination with control bytes

- **Where:** `crates/ascribe-syntax/src/convert.rs`, the image and link inline spans; found by the `parse` fuzz target's span-coverage assertion (`fuzz/fuzz_targets/parse.rs`) on a 55-byte page whose image destination holds a control character and NUL bytes: an inline span overlaps the one before it, a link's span doesn't start at `[`.
- **What it costs:** nothing today: `check`, `fmt`, and `build` handle the input with ordinary diagnostics. Spans drive the language server's tokens, positions, hovers, and edits, and a consumer that sliced the source by a raw span instead of `get` would panic on the mis-sized one. The nightly fuzz job will open an issue for it on its next run that reaches the input.
- **Recommendation:** make the image and link spans cover exactly their source whatever the destination holds, and add the input as a conformance case.
- **Size:** medium.

## H2: a snippet's code file is read whole before the non-text check

- **Where:** `crates/ascribe-resolve/src/snippet/mod.rs`, `read_code`: `read_file` reads the whole file, then the NUL and UTF-8 checks reject it. Decision 79 bounds a page's expansion, not one source file.
- **What it costs:** a committed code file of hundreds of megabytes under a `[sources]` folder or a `sources/<name>/` copy is allocated in full on every read, in the language server too. Bounded by the file's size, with no amplification: memory pressure, not a crash.
- **Recommendation:** a size limit for a snippet's file, checked with `FileSystem::size` before the read and reported as the NUL case is, far above any real code file.
- **Size:** small.

## H3: the HTML report's policy allows `style-src 'unsafe-inline'`

- **Where:** `crates/ascribe-diff/src/html/mod.rs`, the report's policy: `default-src 'none'; img-src data:; style-src 'unsafe-inline'; script-src 'sha256-…'`. Script is held to the embedded hash and images to `data:`, so a page can't run script or reach the network; a page's raw `style=` and `<style>` apply to the whole report.
- **What it costs:** a pull request's page can restyle or visually spoof the report a reviewer opens. No fetch, so no exfiltration through CSS. Phase 4's note on the same policy (no `form-action`, no `base-uri`) goes with this one.
- **Recommendation:** serve the report's own stylesheet under a hash, as the script is, and drop `'unsafe-inline'`.
- **Size:** medium.

## H4: `ascribe fmt` doesn't reach a fixed point on a block-quoted directive with control bytes

- **Where:** `crates/ascribe-fmt`; found by the `format` fuzz target's idempotence assertion on a 322-byte page with `@note` inside a block quote and control and NUL bytes in its text. The command itself returns 0 and doesn't hang; a second `fmt` changes the file again.
- **What it costs:** a correctness bug, not a security one: CI's "fmt is clean" check can't settle on such a file. Not exploitable under decision 2.
- **Recommendation:** make `format` reach its fixed point in one pass there, with the input as a case.
- **Size:** medium.

## Checked, nothing found

- **Panics in the loaders and readers.** The `model` target over arbitrary TOML (the parser bounds its nesting), the new `frontmatter` target (`index_file` and `frontmatter_lines`; the YAML reader bounds depth to 128 and alias fan-out), the new `openapi` target (`read_spec` as YAML and JSON, then the schema translation both ways; `$ref` following keeps a `seen` set, so a cycle ends), and the new `example` target (`json_schema::check` against an arbitrary schema and example in both dialects; pattern removal is bounded, `$ref` cycles guarded): about sixteen minutes each, no crash.
- **A YAML alias bomb** in frontmatter and in a spec: rejected by the reader's repetition limit in about 70 milliseconds, no memory growth.
- **Deep block nesting and long lines.** 50,000 nested block quotes parse (comrak handles blocks iteratively); 20,000 unbalanced `[` parse. The inline case is one of the private findings.
- **Regexes the content model supplies.** A field's pattern is the model's own glob matcher (`crates/ascribe-model/src/pattern.rs`), no backtracking engine; a spec's JSON Schema `pattern` goes to `boon` on the `regex` crate, and an unreadable one is dropped. No backtracking engine is fed project input.
- **Paths as written.** `..`, absolute, root-relative, percent-encoded, and backslash forms in a link, image, or include are refused as missing or outside; `RelPath::parse` keeps `..` only at the start and the layout's allow rule rejects what climbs out. A case difference is `CaseMismatch`, reported as missing. A snippet address refuses `..`, `.`, a leading `/`, `\`, and NUL, and `links_stay_in` checks the real path. A memo's `about` is a target name, not a path, read only from `memos/` and `.ascribe/memos/` through `FileSystem`.
- **Writes.** `ascribe addresses` and `permalinks sync` write only fixed files beside `ascribe.toml`; a key changes the file's content, not its path. `rename` refuses a move unless both ends are inside the project. (`templates eject`'s target folder and the servers' edits are phase 3's.)
- **What the site output writes.** Every value Ascribe puts into an attribute or element goes through the site emitter's `escape` or the renderer's `encode`, both escaping `& < > "`: anchor comments, `<ascribe-operation>`'s attributes and link text from a spec, availability targets, tab labels, a glossary term's `title`, a heading's `@id`, image markers. The JSON and plain outputs are serialized, never read as HTML. The HTML report's own text and embedded JSON are escaped, and its base label is a commit hash, not a message. Snippet code is a fenced block with escaped attributes.
- **One bad file in the language server.** A handler's panic is caught and costs only its own diagnostics; a compile-time check holds `panic = "unwind"`. The exception is the private finding, which isn't a panic.
