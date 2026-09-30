# Findings

Every false positive, language question, and performance problem the corpora
and the benchmarks turned up, at the pinned commits (`src/corpus.rs`). Each has
a minimal reproduction and an owner. Language questions are in
`project-docs/questions.md` (Q191 to Q196); no GitHub issues were opened.

- **R**: recognition, over the unconverted Markdown.
- **F**: found by converting: the language.
- **P**: performance.

## Recognition (unconverted Markdown)

**Result: no false positives.** Across 4,546 pages (Astro 422, Elastic 3,008,
Docker 1,116) the parser found no directive, end line, title line, group, or
image attribute block, and no directive-shaped line with an unknown name. What
it did find is below, and each class has a verdict in `recognition.toml` that
`tests/recognition.rs` enforces (a new class with no verdict fails the test).

The corpora barely exercise the two risks the phase names most. An independent
scan of the raw lines (`oracle` in `tests/recognition.rs`) finds **one** prose
line starting with `@` outside code (`@astrojs/node can be configured…`, not
directive-shaped, so not recognized) and **two** starting with `.` (`.NET agent`,
`.github/scripts/…`), none recognized. Prose `@` mostly sits inside code fences
here, and mid-line (`@timestamp` in a sentence) is never a directive. So the
hand-written reproductions in `tests/edge.rs` carry the weight: they pin
`@astrojs/react`, `@timestamp`, `support@example.com`, `.NET`, `.env`, braces
that aren't keys, and the intended warnings, offline.

### R1: `{name}` in prose is a phrase candidate: intended (SPEC §5.1)

```
Open {namespace}/{name}.
```

A lowercase key in braces is a candidate; with the key undeclared the text is
literal and `phrase-undeclared` warns, as §5.1 says it SHOULD. In the corpora:
Astro 4 (`{children}` in MDX, `{stars}`), Docker 25 (`/containers/{name}`,
`{namespace}`), Elastic 481 (414 of them `{icon}` and 34 `{kbd}`, MyST roles
written `{icon}\`gear\``). Braces that aren't keys (`{}`, `{ "a": 1 }`, `{2,3}`,
`{Object}`, `{a b}`) are not candidates (`tests/edge.rs`). Not a false positive.

### R2: `{{key}}` holds the candidate `{key}`: intended; resolved by Q191

```
Use {{es}} here.
```

40,729 in Elastic prose and 6,607 in link destinations (`[x]({{es-apis}})`).
Intended by §5.1 (braces may sit against punctuation), and harmless while the
key is undeclared. The risk is a *declared* key left in double braces: it
renders `{Elasticsearch}` with no diagnostic. **Resolved (Q191):** `ascribe
check` now warns about it (`phrase-double-braces`, ASC126). On the converted
Elastic sample, it finds 295 `{{key}}` the converter didn't convert (see
"Converter limits").

### R3: another tool's `:::{name}` line: intended

```
:::{note}
Text.
:::
```

12,568 lines in Elastic. The file isn't Ascribe, so the line is a paragraph and
`{note}` a candidate. The converter removes them.

### R4: a repeated YAML key: intended

Two Elastic pages repeat `navigation_title:` in their frontmatter, which isn't
valid YAML. `frontmatter-syntax` is right. (The upstream tooling tolerates it.)

### R5: a known keyword at line start in prose is a directive: intended, watch

```
@include mixin(x)
```

is the directive `@include` with text after its primary (`directive-extra-text`)
per §3.2. None of the corpora has one (Sass documentation would). `\@include`
or code silences it. Recorded in `tests/edge.rs`; no change proposed.

## Converting (the language)

The converters (`src/convert/`) turn each corpus's constructs into Ascribe. What
they produce is realistic enough to expose these. Counts are for the converted
Elastic sample unless noted.

### F1: no availability for an include (Q192): 17 errors

```
@available: cloud
@include: _snippets/cloud-only.md
```

`binding-no-block`: §3.8 says `@include` isn't a block. Elastic wraps an
include in `applies-item` 17 times in this sample. Owner: phases 06 and 12, if
the language changes. Reproduction: `f1_…` in `tests/edge.rs`.

### F2: a title can't start with a dot (Q193): 9 errors (6 Elastic, 3 Docker); resolved

```
..NET
@variant:
text
@end
```

`.NET` above a directive is the title `NET`; `..NET` isn't a title, so the arm
has none (`variant-arm-kind`). **Resolved (Q193):** `.\.NET` is the title
`.NET`, and the converters write it that way, so the 9 errors are gone.
Reproduction: `f2_…`.

### F3: a section can't name a target its page doesn't (Q194): 147 errors

Page `available: stack ga`, a block `@available: serverless ga`:
`available-exceeds-scope`. §4.4 says so; Elastic's `applies_to` doesn't. The
converter doesn't widen the page's spec, so the errors stay visible.

### F4: only headings can be link targets (Q195): 647 unresolved anchors

`$$$anchor$$$` marks any place in Elastic's docs (1,074 in 135 files); a link to
one is `link-id-missing` (647 of the sample's 992). A further 110 use legacy
Asciidoc ids (`_configuration_files_…`), which no slugger of ours produces.

### F5: ids can't contain `_` or `.` (Q196): 2,534 of 13,007 anchors; resolved

```
## Setup
@id: ece_setup
```

`id-invalid`. The converter rewrites the id (`ece-setup`) and the links it
sees (162); links from outside the site can't be found. **Resolved (Q196):**
ids may contain `_` and `.`, so the converter could keep such anchors as they
are; it still rewrites them. Reproduction: `f5_…`.

### Converter limits (not findings about the language)

- **Markdig-only syntax**: definition lists (`term` / `:   text`) whose bodies
  hold a directive indented four spaces are indented code in CommonMark. Those
  are 384 `directive-indented-code` warnings, correct for the converted text.
- **Unclosed source directives**: a few pages leave `::::{tab-set}` open; the
  converter closes it at the end of the file (6 `container-unclosed`).
- **MDX indentation**: content indented inside `<Steps>` is code in Markdown
  (5 `steps-not-ordered-list` in Astro).
- **Routes**: Astro's links are routes (`/en/guides/x/`), so 2,135 `link-route`
  warnings; Docker and Astro headings repeated across tabs or includes give
  `heading-duplicate-without-id` (2,146 in Docker).
- **`{{key}}` in code spans** stays as written (381): phrases never apply in
  code spans (§5.1), so the substitution is lost. That's the spec, not a bug.
- **`{{key}}` left in prose** (295 in Elastic): substitutions the converter
  doesn't reach, which `phrase-double-braces` (Q191) now reports. A converter
  follow-up, not a language one.

## Performance

Numbers are in `RESULTS.md`; the container is a 4-core 2.1 GHz Xeon, slower
than a developer laptop and with slow system calls.

### P1: the text report is quadratic (owner: phase 10, `report/text.rs`); fixed

```
ascribe check          # the default, text
ascribe check --format json
```

`write_diagnostic` calls `FileTable::texts()` (which clones every file's text)
and `ariadne::sources` (which builds line tables for every file) **for every
diagnostic**, so the cost is diagnostics x corpus size. Converted Docker (1,116
files, 8.6 MB, about 4,000 diagnostics): **80 s** in text and **3.2 s** as JSON.
Converted Elastic (21 MB, 2,895): over **100 s** against 7 s. Reproduce with
`cargo bench -p tessera-corpora --bench perf` (the noisy project, 1,000 pages
with one warning each: 1.4 s against 0.25 s; the gap grows with the corpus).
The fix is one cache built once per report. **Fixed:** the report now fills
one source cache as it goes, so each file's text is read and indexed once; on
1,000 generated pages with a warning each, text output went from 6.7 s to
0.54 s (JSON: 0.19 s).

### P2: system calls (owner: phases 12 and 10, suspected)

`strace -c -f ascribe check --format json` on converted Elastic: 23,280
`getdents64`, 24,985 `openat`, 72,804 `write` for 3,008 pages. System time is
2.5 to 4.4 s of a 5 to 8 s run here. The directory listings look like a
case-exact existence probe that lists the parent directory each time (the asset
contract's exact-case rule); the writes are the JSON report going out
unbuffered. Both would be much cheaper on a laptop and are fixable without
touching results.

### P3: page-level checks dominate on real pages (owner: phase 14)

Converted Elastic: file-level checks 1.2 s, all builds 8.7 s in process. The
same 3,000 pages of the synthetic project take 0.7 s: real pages average 7 KB
against 0.5 KB, and phase 14's notes already say `PageChecker` indexes the
project a second time and resolves every page per build. See the miss recorded
in `RESULTS.md`.

### P4: an unchanged `ascribe build` isn't faster than the first (watch)

3,000 pages, plain and JSON: first build 2.7 s, unchanged rebuild 3.6 s (2.3 to
4.0 s across runs). "Unchanged files left alone" costs a read of each, but the
rebuild should not cost more than writing them. Owner: phase 18; measure again
before acting.
