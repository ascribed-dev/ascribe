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

## H4: `ascribe fmt` doesn't reach a fixed point on a block-quoted directive with control bytes (fixed on `main`)

- **Where:** `crates/ascribe-fmt`; found by the `format` fuzz target's idempotence assertion on a 322-byte page with `@note` inside a block quote and control and NUL bytes in its text. The command itself returns 0 and doesn't hang; a second `fmt` changes the file again.
- **What it costs:** a correctness bug, not a security one: CI's "fmt is clean" check can't settle on such a file. Not exploitable under decision 2.
- **Recommendation:** make `format` reach its fixed point in one pass there, with the input as a case.
- **Size:** medium.
- **Re-checked 29 October:** fixed on `main` by the quoted list item's indent rule (`changes/fixes/format-quoted-list-item.md`, `crates/ascribe-fmt/src/indent.rs`): the `format` fuzz target passes the input, and a second and third `fmt` change nothing.

## Checked, nothing found

- **Panics in the loaders and readers.** The `model` target over arbitrary TOML (the parser bounds its nesting), the new `frontmatter` target (`index_file` and `frontmatter_lines`; the YAML reader bounds depth to 128 and alias fan-out), the new `openapi` target (`read_spec` as YAML and JSON, then the schema translation both ways; `$ref` following keeps a `seen` set, so a cycle ends), and the new `example` target (`json_schema::check` against an arbitrary schema and example in both dialects; pattern removal is bounded, `$ref` cycles guarded): about sixteen minutes each, no crash.
- **A YAML alias bomb** in frontmatter and in a spec: rejected by the reader's repetition limit in about 70 milliseconds, no memory growth.
- **Deep block nesting and long lines.** 20,000 unbalanced `[` and `*` parse with an ordinary diagnostic. The review's first write-up said 50,000 nested block quotes parse too; **that was wrong**, found on the re-check of 29 October: nested block quotes overflow the stack in the parser's conversion as nested inlines do, so the private finding covers both, with the block case reaching the language server at a lower depth.
- **Regexes the content model supplies.** A field's pattern is the model's own glob matcher (`crates/ascribe-model/src/pattern.rs`), no backtracking engine; a spec's JSON Schema `pattern` goes to `boon` on the `regex` crate, and an unreadable one is dropped. No backtracking engine is fed project input.
- **Paths as written.** `..`, absolute, root-relative, percent-encoded, and backslash forms in a link, image, or include are refused as missing or outside; `RelPath::parse` keeps `..` only at the start and the layout's allow rule rejects what climbs out. A case difference is `CaseMismatch`, reported as missing. A snippet address refuses `..`, `.`, a leading `/`, `\`, and NUL, and `links_stay_in` checks the real path. A memo's `about` is a target name, not a path, read only from `memos/` and `.ascribe/memos/` through `FileSystem`.
- **Writes.** `ascribe addresses` and `permalinks sync` write only fixed files beside `ascribe.toml`; a key changes the file's content, not its path. `rename` refuses a move unless both ends are inside the project. (`templates eject`'s target folder and the servers' edits are phase 3's.)
- **What the site output writes.** Every value Ascribe puts into an attribute or element goes through the site emitter's `escape` or the renderer's `encode`, both escaping `& < > "`: anchor comments, `<ascribe-operation>`'s attributes and link text from a spec, availability targets, tab labels, a glossary term's `title`, a heading's `@id`, image markers. The JSON and plain outputs are serialized, never read as HTML. The HTML report's own text and embedded JSON are escaped, and its base label is a commit hash, not a message. Snippet code is a fenced block with escaped attributes.
- **One bad file in the language server.** A handler's panic is caught and costs only its own diagnostics; a compile-time check holds `panic = "unwind"`. The exception is the private finding, which isn't a panic.

## Re-checked on 29 October 2026, against `main` at `85c4c85c`

Two hundred commits later: H1, H2, and H3 hold unchanged (H2 slightly better in the servers, which now stamp each code file and re-read it only on change); H4 is fixed. The fuzz crate builds against the moved crates, and one more minute of each new target found nothing. One "nothing found" claim was wrong, above. What the 200 commits added to this phase's surface, each looked at:

- **The git guard** (`crates/ascribe-cli/src/guard.rs`, decision #965) runs before every writing command and the MCP tools that write. It's a restorability check (is each existing file tracked and clean), not a boundary: a file that doesn't exist yet is allowed anywhere, and git is asked about a link's own name, not its target. The readers' link rule is still the boundary.
- **`ascribe new`** (`crates/ascribe-cli/src/commands/new.rs`) writes a page at a path the user names, `<slug>.md` only, never overwriting, and refuses a path outside the content root; it doesn't refuse a linked folder on the way, so `ascribe new page docs/<link>` writes the file where the link points. The user named the folder, and the name is a slug, so a hardening note: **H5** below.
- **`ascribe-workspace`'s listing** (`crates/ascribe-workspace/src/listing.rs`) uses `DirEntry::file_type`, so a linked folder isn't descended and a linked file is stamped only; `ascribe-edit`'s one read outside `FileSystem` is the case-only rename's probe; `ascribe-report`'s link cache and histories moved into the CLI's `DiskStore`, which writes a temporary file and renames and refuses a linked folder on the way; `sources fetch`'s copies refuse `..`, a path outside, and a linked folder on the way.
- **`ascribe check --format github`** escapes `%`, line breaks, `:` and `,` as GitHub does, so a message can't end a workflow command. The new fixes `check --fix` writes (GitHub alerts, a title from the heading, written with the YAML serializer) settle in one pass on awkward text.
- **Snippets from named files** (`Dockerfile`, `.env`, and others): a committed `.env` under a `[sources]` folder can now be a snippet, by the author's own address; worth a line in the snippets docs, not a boundary.
- **The hook's check server** (`crates/ascribe-cli/src/agents/checker.rs`) is a third long-running process the review never drove: it checks on a worker thread with no `catch_unwind`, so a panic ends the worker and later answers are silently lost, and it's one more sink of the private crash finding. **H6** below.

### H5: `ascribe new` writes through a linked folder under the content root

- **Where:** `crates/ascribe-cli/src/commands/new.rs`, `write_page`: `create_dir_all` and `create_new` on the user's path, with no link check on the folders on the way, unlike `DiskStore::write` and the sources' copies.
- **What it costs:** a committed link under the content root makes a page the user asks for land outside the project, under the slug the title gives. The user named the folder and the file is new, so a surprise, not a breach.
- **Recommendation:** the `link_on_the_way` check the report's store uses, before `create_dir_all`.
- **Size:** small.

### H6: the hook's check server catches no panic

- **Where:** `crates/ascribe-cli/src/agents/checker.rs`, the worker thread that checks the project (`std::thread::spawn`, the default stack), whose send of the answer is `let _ =`.
- **What it costs:** a panic in a check ends the worker, and every later hook call gets no answer, silently, until the server is restarted; the language server and the MCP server each guard their handlers with `catch_unwind`.
- **Recommendation:** the same guard around the worker's check, and a logged restart of the worker.
- **Size:** small.
