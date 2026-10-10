# Extract-and-embed tools: putting code from real source files into documentation (state as of 2026-10-09)

Notes on method: repository status (last push, latest release, license, stars) was read from the GitHub REST API on 2026-10-09; download counts from the npm, PyPI Stats, and NuGet APIs on the same day. Feature claims come from each tool's own documentation or source. Anything not confirmed at a primary source is listed under Gaps, not under Cited Findings.

## 1. Which ways of marking a region exist, and what are the documented failure modes of each?

### Takeaway
Five ways exist: comment tags with a name (the majority), line numbers, text or regex matching, language symbols (Python objects, C# `#region`, brace blocks), and no region at all (whole file only). Tool documentation itself calls line numbers brittle (Microsoft, mdBook); the documented costs of comment tags are tags leaking as content when not behind a comment, false matches on the tag pattern, name collisions, and unpaired tags.

### Cited Findings

**Named comment tags (start/end pairs)**

- Bluehawk: block tags are `:<tag>-start:` / `:<tag>-end:`, line tags are `:<tag>:`, written inside the source language's comments. `snippet` takes a required identifier, unique per file. Attribute lists are JSON objects whose opening brace must be on the tag's line. — [Bluehawk tag reference](https://mongodb-university.github.io/Bluehawk/reference/tags)
- AsciiDoc: `tag::name[]` / `end::name[]`, placed behind a line comment in the included file. Names cannot be empty or contain spaces; the directive must sit on a word boundary and be followed by a space or end of line. For languages without line comments (XML), wrap in a circumfix comment with a space each side: `<!-- tag::name[] -->`. — [Asciidoctor: include tagged regions](https://docs.asciidoctor.org/asciidoc/latest/directives/include-tagged-regions/)
- AsciiDoc selection: `tag=name` or `tags=a;b`; `*` selects all tagged regions, `**` all lines except tag lines, `!name` negates; `foo;!bar` selects `foo` minus nested `bar`. Including a tag pulls in every region with that name anywhere in the file. — [Asciidoctor: include tagged regions](https://docs.asciidoctor.org/asciidoc/latest/directives/include-tagged-regions/)
- mdBook: `{{#include file.rs:anchor}}`; the anchor is a pair of lines matching `ANCHOR:\s*[\w_-]+` and `ANCHOR_END:\s*[\w_-]+`. "Lines containing anchor patterns inside the included anchor are ignored" (so nested anchors work and their marker lines are dropped). — [mdBook: mdBook-specific features](https://rust-lang.github.io/mdBook/format/mdbook.html)
- pymdownx.snippets: section markers `--8<-- [start:name]` / `--8<-- [end:name]` (added in 9.7), may sit inside comments, referenced as `file:name`; marker lines are always omitted. — [PyMdown Snippets](https://facelessuser.github.io/pymdown-extensions/extensions/snippets/)
- Microsoft Learn `:::code`: `id="snippet_Create"` refers to `// <snippet_Create>` ... `// </snippet_Create>` comments. "Use only letters and underscores for the name." Named snippets can nest; inner tags are not rendered. — [Microsoft Learn: How to include code in docs](https://learn.microsoft.com/en-us/contribute/content/code-in-docs)
- DocFX: `[!code-csharp[](Program.cs#region)]` selects a region by name; `[!code-csharp[](Program.cs#L12-L16)]` selects lines. — [DocFX Markdown](https://dotnet.github.io/docfx/docs/markdown.html)
- Doxygen: `\snippet <file> block_id`; the block is delimited by a pair of marker comments such as `//! [Adding a resource]`. "[block_id] markers should appear exactly twice in the source file"; "the lines containing the block markers will not be included." — [Doxygen special commands](https://www.doxygen.nl/manual/commands.html)
- Javadoc (JEP 413, delivered in JDK 18): `// @start region=name` ... `// @end` (or `@end region=name`) in external or inline snippets; `{@snippet class=X region=name}` or `file=... region=...` selects it. `@highlight`, `@replace`, and `@link` can also open a region with a `region` attribute. Regions may overlap. — [JEP 413](https://openjdk.org/jeps/413)
- MarkdownSnippets: `// begin-snippet: Name` ... `// end-snippet`, or C# `#region Name` / `#endregion`; Markdown refers with `snippet: Name`. — [MarkdownSnippets README](https://github.com/SimonCropp/MarkdownSnippets)
- snipsync (Temporal): `// @@@SNIPSTART id` ... `// @@@SNIPEND` in source; `<!--SNIPSTART id-->` ... `<!--SNIPEND-->` in the target Markdown. IDs may contain letters, numbers, hyphens, underscores. — [snipsync README](https://github.com/temporal-community/snipsync)
- AWS SDK code examples: `snippet-start:[tag]` / `snippet-end:[tag]`; the tooling requires matched pairs and reports "duplicate snippet-start tag", "duplicate snippet-end tag", "snippet-end with no matching start", "snippet-start with no matching end". — [aws-doc-sdk-examples-tools README](https://github.com/awsdocs/aws-doc-sdk-examples-tools); [snippets.py](https://github.com/awsdocs/aws-doc-sdk-examples-tools/blob/main/aws_doc_sdk_examples_tools/snippets.py)
- Google: region tags are checked by snippet-bot, which "detects mismatched region tags by regex for changed files and report the status as Github Check" (tag already started, end without start, start without end) and comments a summary of region tag changes on each pull request. — [snippet-bot README](https://github.com/googleapis/repo-automation-bots/tree/main/packages/snippet-bot)

**Line numbers**

- Sphinx `literalinclude` `:lines: 1,3,5-10,20-`; AsciiDoc `lines=5..10`, `lines=7;14..25`, `-1` for last line; mdBook `file.rs:2:10`; pymdownx `file.md:4:6`, `file.md:1:3,5:6`, negative indexes (10.13); Learn `range="2-24,26"`; DocFX `#L12-L16`. — [Sphinx directives](https://www.sphinx-doc.org/en/master/usage/restructuredtext/directives.html); [Asciidoctor include lines](https://docs.asciidoctor.org/asciidoc/latest/directives/include-lines/); [mdBook](https://rust-lang.github.io/mdBook/format/mdbook.html); [PyMdown Snippets](https://facelessuser.github.io/pymdown-extensions/extensions/snippets/); [Microsoft Learn](https://learn.microsoft.com/en-us/contribute/content/code-in-docs); [DocFX Markdown](https://dotnet.github.io/docfx/docs/markdown.html)
- Line numbers are the only region mechanism in: embedme (`path/to/file.ts#L20-L30`), remark-code-import (`file=./say-hi.js#L3-L6`, `#L3-`), docusaurus-theme-github-codeblock (a GitHub blob URL ending `#L105-L108`), markdown-autodocs (`CODE:src=...&lines=22-44`). — [embedme README](https://github.com/zakhenry/embedme); [remark-code-import README](https://github.com/kevin940726/remark-code-import); [docusaurus-theme-github-codeblock README](https://github.com/saucelabs/docusaurus-theme-github-codeblock); [markdown-autodocs README](https://github.com/dineshsonachalam/markdown-autodocs)

**Text or pattern matching**

- Sphinx `:start-after:` / `:end-before:` (0.6) and `:start-at:` / `:end-at:` (1.5) match "the first line containing that string". — [Sphinx directives](https://www.sphinx-doc.org/en/master/usage/restructuredtext/directives.html)
- reStructuredText `include`: `start-after` / `end-before` match the first occurrence of the text; `start-line` / `end-line` are 0-indexed, end exclusive. — [Docutils directives](https://docutils.sourceforge.io/docs/ref/rst/directives.html)
- Doxygen `\dontinclude` with `\line`, `\skip`, `\skipline`, `\until` walks a pointer through a file by pattern. — [Doxygen special commands](https://www.doxygen.nl/manual/commands.html)
- mkdocs-include-markdown-plugin: `start` and `end` delimiter strings (may contain escapes such as `\n`). — [mkdocs-include-markdown-plugin README](https://github.com/mondeja/mkdocs-include-markdown-plugin)
- Javadoc `@highlight`, `@replace`, `@link` target text by `substring=` or `regex=`. — [JEP 413](https://openjdk.org/jeps/413)

**Language symbols**

- Sphinx `:pyobject: Timer.start`: "For Python files, only include the specified class, function or method." — [Sphinx directives](https://www.sphinx-doc.org/en/master/usage/restructuredtext/directives.html)
- mkdocs-codeinclude-plugin: `block:someString` finds a line containing the token and takes the next curly-brace-delimited block; `inside_block:` omits the opening line and closing brace. Every line is searched for the token. — [mkdocs-codeinclude-plugin README](https://github.com/rnorth/mkdocs-codeinclude-plugin)
- C# `#region` is accepted by MarkdownSnippets and DocFX. — [MarkdownSnippets README](https://github.com/SimonCropp/MarkdownSnippets); [DocFX Markdown](https://dotnet.github.io/docfx/docs/markdown.html)
- Javadoc `{@snippet class=HelloWorld}` names a Java source file by class. — [Oracle: Programmer's Guide to Snippets](https://docs.oracle.com/en/java/javase/21/javadoc/snippets.html)

**Whole file only (no region)**

- Docusaurus `!!raw-loader!./file.js` import passed to `<CodeBlock>`; requires installing raw-loader; "This feature is experimental and might be subject to breaking API changes in the future." — [Docusaurus: MDX and React](https://docusaurus.io/docs/markdown-features/react)
- Expressive Code / Starlight: "import any code file as a string" with Vite's `?raw` suffix and pass it to `<Code code={...}>`. The page documents no file include or region selection. — [Expressive Code: Code component](https://expressive-code.com/key-features/code-component/)
- Hugo `os.ReadFile` / `readFile` "returns raw (uninterpreted) content"; the page documents no line or region selection. — [Hugo os.ReadFile](https://gohugo.io/functions/os/readfile/)
- Jekyll `include` (from `_includes`) and `include_relative`; no line or region selection documented. "you cannot use the `../` syntax to specify an include location that refers to a higher-level directory." — [Jekyll includes](https://jekyllrb.com/docs/includes/)

**Documented failure modes**

- Line numbers: "Line number references are brittle because code files inevitably change in ways that make line numbers change. You don't necessarily get notified of such changes. Your article eventually starts showing the wrong lines and you have no clue anything has changed." Microsoft's rule: "Prefer named snippets over hard-coded line numbers." — [Microsoft Learn](https://learn.microsoft.com/en-us/contribute/content/code-in-docs)
- Line numbers: mdBook recommends anchors "To avoid breaking your book when modifying included files." — [mdBook](https://rust-lang.github.io/mdBook/format/mdbook.html)
- Tags as content: an AsciiDoc tag not behind a comment "will be treated as regular content"; without tag filtering, tag lines appear in the output. — [Asciidoctor: include tagged regions](https://docs.asciidoctor.org/asciidoc/latest/directives/include-tagged-regions/)
- False matches: Bluehawk issue "Some constructs might match bluehawk :command: regex", "For example, rST roles and some coincidental colon placement" (open since 2022-07-01). — [Bluehawk #124](https://github.com/mongodb-university/Bluehawk/issues/124)
- Name collisions: Bluehawk "Similar output files can clobber each other": `node/auth.js` and `web/auth.js` with the same snippet name write the same output file "without regard for the difference" (open since 2022-07-01). — [Bluehawk #115](https://github.com/mongodb-university/Bluehawk/issues/115)
- Editor folding: MarkdownSnippets warns that Visual Studio may collapse `#region` blocks, so outlining may need to be disabled. — [MarkdownSnippets README](https://github.com/SimonCropp/MarkdownSnippets)
- Comment syntax limits: in Javadoc, "properties files only support line comments ... And, even in Java source files, you cannot use end-of-line comments within a text block"; the workaround is a markup comment ending in `:` that applies to the next line. Inline snippets "may not contain the character pair `*/`" and braces must be balanced. — [Oracle: Programmer's Guide to Snippets](https://docs.oracle.com/en/java/javase/21/javadoc/snippets.html)
- Build systems: "Some build systems may (incorrectly) treat files in the `snippet-files` directory as part of the enclosing package hierarchy... The local `snippet-files` directory cannot be used in these cases." — [Oracle: Programmer's Guide to Snippets](https://docs.oracle.com/en/java/javase/21/javadoc/snippets.html)
- Token search ambiguity: mkdocs-codeinclude searches every line for the token, so a token appearing on more than one line is a stated concern (the README's note on this was cut off in what I read). — [mkdocs-codeinclude-plugin README](https://github.com/rnorth/mkdocs-codeinclude-plugin)
- Pattern matching: Sphinx and Docutils match only the first occurrence of the text. — [Sphinx directives](https://www.sphinx-doc.org/en/master/usage/restructuredtext/directives.html); [Docutils directives](https://docutils.sourceforge.io/docs/ref/rst/directives.html)

### Inferences
- The tools split by who owns the region's name: tags put the name in the code file (docs refer to it), line numbers and text patterns put the knowledge only in the docs, and symbol selectors reuse a name the language already has. Only the first and third survive edits to the code file without a silent wrong result.
- Nesting is handled three different ways: allowed with inner markers stripped (mdBook, Microsoft Learn, AsciiDoc), allowed and overlapping (Javadoc), or undocumented (pymdownx, Bluehawk's tag page).
- Symbol-based selection is rare and shallow: one language in Sphinx (Python), brace matching rather than parsing in mkdocs-codeinclude. I found no mainstream tool in this family that selects by AST node across languages.

### Gaps
- Google's public page on region tag format (`[START x]` / `[END x]`), naming, and uniqueness: the URL I tried (docs.cloud.google.com/docs/samples/snippet-conventions) returned 404, so the tag format itself is not confirmed here at a primary source. Only snippet-bot's behaviour is.
- DocFX's per-language table of tag forms (the "tag name representation in code snippet source file" section Microsoft Learn links to) was not on the DocFX page I read; the exact comment forms per language are unconfirmed.
- I found no primary source documenting tags breaking code formatters or linters. MongoDB's Grove process runs the language formatter before snipping, but the page does not say tags caused formatter problems.
- Whether Bluehawk tags can nest (for example `state` inside `snippet`) is not stated on its tag reference page.

## 2. Which tools support transformations beyond plain extraction, and how are they expressed?

### Takeaway
Bluehawk and Javadoc `{@snippet}` are the only tools here with a real transformation vocabulary written in the code file (remove, replace, uncomment, states; highlight, replace, link). Most others offer only dedent and highlight, set on the docs side by line number relative to the extracted snippet.

### Cited Findings

**Bluehawk (all tags)**

- `snippet` (block only): marks a range; `bluehawk snip` writes each to its own file. — [Bluehawk tag reference](https://mongodb-university.github.io/Bluehawk/reference/tags)
- `remove` (block or line): removes the range from output. — [Bluehawk tag reference](https://mongodb-university.github.io/Bluehawk/reference/tags)
- `replace` (block only): attribute list with a `terms` dictionary; each key is swapped for its value within the range, in all output. — [Bluehawk tag reference](https://mongodb-university.github.io/Bluehawk/reference/tags)
- `uncomment` (block only): removes one comment marker from the start of each line in the range. "plaintext does not have a comment syntax, so this tag does nothing in plaintext." — [Bluehawk tag reference](https://mongodb-university.github.io/Bluehawk/reference/tags)
- `state` (block only): one or more names; the range is kept only when the name matches `--state`; without the flag, state content is omitted. — [Bluehawk tag reference](https://mongodb-university.github.io/Bluehawk/reference/tags)
- `state-uncomment` (block only): like `state`, and also removes one layer of comments. — [Bluehawk tag reference](https://mongodb-university.github.io/Bluehawk/reference/tags)
- `state-remove` (block): removes the range for the named states and keeps it in all others. — [Bluehawk tag reference](https://mongodb-university.github.io/Bluehawk/reference/tags)
- `emphasize` (block or line): highlights marked lines in formatted output and calculates the line numbers. "The emphasize tag only applies to certain formatted outputs." — [Bluehawk tag reference](https://mongodb-university.github.io/Bluehawk/reference/tags)
- `code-block` was removed as an alias for `snippet` in version 1.0. — [Bluehawk tag reference](https://mongodb-university.github.io/Bluehawk/reference/tags)
- Commands: `snip` writes `<source-file-name>.snippet.<snippet-name>.<extension>` and omits `state` content by default; `copy` writes whole processed files in the original directory structure (binary files copied unprocessed); `check` reports errors without writing files and exits non-zero on any Bluehawk error. Flags: `--output`, `--state`, `--ignore` (gitignore-style; `.gitignore` files applied automatically), `--rename` (copy only, JSON map, no paths), `--format` (`rst`, `md`, `docusaurus`). The `md` format does not support `emphasize`. — [Bluehawk CLI reference](https://mongodb-university.github.io/Bluehawk/reference/cli)
- Stated uses include "Replace 'finished' code with 'todo' code for a branch in a tutorial repo". — [Bluehawk README](https://github.com/mongodb-university/Bluehawk)

**Javadoc `{@snippet}`**

- `@highlight` with `substring=` or `regex=`, `type` of `bold`, `italic`, or `highlighted`; `@replace` with `replacement=` (regex groups as `$1`; an empty replacement deletes text); `@link` with `target=`. Markup comments do not appear in output. Ending a markup comment with `:` applies it to the following line. — [JEP 413](https://openjdk.org/jeps/413)
- "Incidental white space is removed from the content in the same way as with `String.stripIndent`." — [Oracle: Programmer's Guide to Snippets](https://docs.oracle.com/en/java/javase/21/javadoc/snippets.html)
- Hybrid snippets carry the code both inline and in an external file; "the Standard Doclet verifies that the result of processing the snippet tag as an inline snippet is the same as processing it as an external snippet." — [Oracle: Programmer's Guide to Snippets](https://docs.oracle.com/en/java/javase/21/javadoc/snippets.html)

**Sphinx `literalinclude`**

- `:dedent:` (1.3; automatic common-indent removal via `textwrap.dedent()` in 3.5), `:emphasize-lines:`, `:prepend:` / `:append:` (1.0), `:diff:` (1.3, unified diff against another file), `:lineno-match:` (1.3, "only allowed only when the selection consists of contiguous lines"), `:lineno-start:`, `:tab-width:`, `:language:`, `:caption:`, `:force:` (2.1). — [Sphinx directives](https://www.sphinx-doc.org/en/master/usage/restructuredtext/directives.html)
- "the line numbers in emphasize-lines refer to these selected lines, counted consecutively starting from 1." — [Sphinx directives](https://www.sphinx-doc.org/en/master/usage/restructuredtext/directives.html)
- Sphinx warns "non-whitespace stripped by dedent" when dedent cuts into code. — [sphinx/directives/code.py](https://github.com/sphinx-doc/sphinx/blob/master/sphinx/directives/code.py)

**Others**

- reStructuredText `include`: `literal`, `code`, `number-lines`, `tab-width`, `parser`; no dedent or highlight. — [Docutils directives](https://docutils.sourceforge.io/docs/ref/rst/directives.html)
- AsciiDoc: `indent=0` removes leading indentation, `indent=n` re-indents by n columns; "If any line in the verbatim content is not indented, the attribute is effectively ignored" (as summarized from the page). It is an attribute of the verbatim block. — [Asciidoctor: include with indent](https://docs.asciidoctor.org/asciidoc/latest/directives/include-with-indent/)
- mdBook `{{#rustdoc_include}}`: shows the chosen lines or anchor and includes the rest of the file prefixed with `#` so it is hidden, letting "a reader can expand the snippet to see the complete example"; rustdoc uses the full example under `mdbook test`. Hidden-line prefixes can be configured for other languages in `book.toml` or per block with `hidelines=`. — [mdBook](https://rust-lang.github.io/mdBook/format/mdbook.html)
- pymdownx.snippets: `dedent_subsections` (9.10, experimental, default `False`) removes common leading whitespace from sections or line ranges. No highlight, remove, or replace. "Snippets is not a template engine." — [PyMdown Snippets](https://facelessuser.github.io/pymdown-extensions/extensions/snippets/)
- mkdocs-include-markdown-plugin: `dedent`, `trailing-newlines`, `rewrite-relative-urls`, `comments`, `recursive`, `order`. — [mkdocs-include-markdown-plugin README](https://github.com/mondeja/mkdocs-include-markdown-plugin)
- mkdocs-codeinclude-plugin: "Any code included will be have its indentation reduced"; adjacent includes become tabs. — [mkdocs-codeinclude-plugin README](https://github.com/rnorth/mkdocs-codeinclude-plugin)
- Microsoft Learn: `highlight="2-4,6"`, "The numbering is relative to the lines displayed (as specified by range or id), not the file"; `interactive=` for Cloud Shell and Try .NET. "You can't highlight code when you include it in the article Markdown file. It works only for code snippets included by reference to a code file." — [Microsoft Learn](https://learn.microsoft.com/en-us/contribute/content/code-in-docs)
- DocFX: `?highlight=2,5-7,9-`. — [DocFX Markdown](https://dotnet.github.io/docfx/docs/markdown.html)
- Doxygen `\snippet{lineno,trimleft,doc,local,strip,nostrip}`; `trimleft` can "remove the common spacing in front of all lines". — [Doxygen special commands](https://www.doxygen.nl/manual/commands.html)
- MarkdownSnippets: parenthesized metadata after the snippet key is passed to the rendered fence, for example `(title=Program.cs {1})` in Expressive Code syntax; a leading `lang=` overrides the language. — [MarkdownSnippets README](https://github.com/SimonCropp/MarkdownSnippets)
- snipsync: `enable_code_dedenting` (default false); per-use JSON on the target marker: `selectedLines` ("relative to the snippet, not the source file") and `highlights`. — [snipsync README](https://github.com/temporal-community/snipsync)
- remark-code-import: `removeRedundantIndentations`, `preserveTrailingNewline`. — [remark-code-import README](https://github.com/kevin940726/remark-code-import)
- Expressive Code `<Code>`: `mark` / `ins` / `del` props define text and line markers on the imported string. — [Expressive Code: Code component](https://expressive-code.com/key-features/code-component/)
- AWS tooling strips snippet tags and SPDX headers when writing snippet files (`strip_snippet_tags`, `strip_spdx_header`). — [snippets.py](https://github.com/awsdocs/aws-doc-sdk-examples-tools/blob/main/aws_doc_sdk_examples_tools/snippets.py)

### Inferences
- Transformations sit in one of two places. Bluehawk and Javadoc write them in the code file next to the lines they affect, so they move with the code. Sphinx, Learn, DocFX, snipsync, and Expressive Code write highlight and line selection in the docs by line number relative to the snippet, which is the same brittleness as line-number regions at a smaller scale.
- Removing lines and replacing text (hiding test scaffolding, secrets, connection strings) exists only in Bluehawk (`remove`, `replace`), Javadoc (`@replace`), and mdBook's hidden lines. Multiple named variants of one file exist only in Bluehawk (`state`).
- Callouts attached to extracted code are not offered by any tool read here except through the host format (AsciiDoc callouts were not checked).

### Gaps
- AsciiDoc callouts (`<1>`) in included source and how they survive tag filtering: not checked.
- Whether Bluehawk dedents snippets automatically: not stated on the pages read.
- MarkdownSnippets' whitespace trimming and max-width behaviour: the README links to separate pages I did not read.

## 3. Which tools read at build time versus copy snippets into the docs repository, and what do users say about each?

### Takeaway
Site generators' built-in features (Sphinx, AsciiDoc, mdBook, pymdownx, Learn/DocFX, Doxygen, Javadoc, Docusaurus and Expressive Code imports) read the file during the build. The standalone tools (Bluehawk, MarkdownSnippets, embedme, snipsync, markdown-magic, markdown-autodocs, mdsh, cog) write code into files that are then committed, and nearly all of them ship a "verify nothing changed" mode for CI to make up for it.

### Cited Findings

- Bluehawk writes files: one file per snippet (`snip`) or processed whole files (`copy`), optionally pre-formatted as rST, Markdown, or Docusaurus code blocks. — [Bluehawk CLI reference](https://mongodb-university.github.io/Bluehawk/reference/cli)
- MongoDB's current process: "Every test suite includes a provided `snip.js` script to snip the tested examples for inclusion in documentation. The snip script copies the examples and output files from the test suite to the `content/code-examples/tested` directory. If you use markup in your example files, the snip script uses the Bluehawk CLI to extract and transform the relevant code." Output is committed: "git only shows files that have changed ... you should only see the new or changed files in your PR." — [MongoDB Meta: Format and Snip Examples](https://www.mongodb.com/docs/meta/grove/code-testing/snip-tested-examples.md)
- MongoDB's snip script "is not aware of what files you have changed in the PR. It always snips all files in the `examples` directory." — [MongoDB Meta: Format and Snip Examples](https://www.mongodb.com/docs/meta/grove/code-testing/snip-tested-examples.md)
- Open Bluehawk requests that follow from the copy model: "Publish a GHA that ensures `bluehawk snip` was run" ([#142](https://github.com/mongodb-university/Bluehawk/issues/142)), "Clean output directory" ([#155](https://github.com/mongodb-university/Bluehawk/issues/155)), "Allow auto-creating the output directory" ([#154](https://github.com/mongodb-university/Bluehawk/issues/154)), "Generate code block ID index/manifest" ([#118](https://github.com/mongodb-university/Bluehawk/issues/118)).
- MarkdownSnippets has two modes: SourceTransform (default) merges `*.source.md` into `.md` and "will overwrite any existing `.md` files that have matching `.source.md` files"; InPlaceOverwrite rewrites every `*.md` directly. It writes an HTML comment marker, an anchor `<a id='snippet-KEY'>`, the fenced code, and a "snippet source" link with a line range such as `#L1-L11`. `--read-only` marks generated files read-only. — [MarkdownSnippets README](https://github.com/SimonCropp/MarkdownSnippets)
- embedme rewrites the Markdown in place, keeping the path comment as the first line of the fence so it can re-run; `--verify` will "Verify that running embedme would result in no changes. Useful for CI"; also `--dry-run`, `--stdout`. — [embedme README](https://github.com/zakhenry/embedme)
- snipsync splices code between `<!--SNIPSTART id-->` and `<!--SNIPEND-->` in target files; "Any text inside of the placeholders will be replaced by the code snippet when the tool runs." — [snipsync README](https://github.com/temporal-community/snipsync)
- mdsh updates blocks in place: "Most other tools would produce a new file but we really want a sort of idempotent operation here." `--frozen` will "Fail if the output is different from the input. Useful for CI"; `--clean` removes generated blocks. Its `<` command reads a file as is; `$` runs a command. — [mdsh README](https://github.com/zimbatm/mdsh)
- markdown-magic replaces content between comment blocks in Markdown from local or remote sources (`CODE`, `FILE`, `REMOTE` transforms); "Default behavior is replacing the original file." — [markdown-magic README](https://github.com/DavidWells/markdown-magic)
- markdown-autodocs is a GitHub Action that fills `<!-- MARKDOWN-AUTO-DOCS:START (CODE:src=...) -->` blocks and commits the result (`commit_message: Apply automatic changes`). — [markdown-autodocs README](https://github.com/dineshsonachalam/markdown-autodocs)
- cog describes itself as a "content generation tool. Small bits of computation for static files." — [cog README](https://github.com/nedbat/cog)
- Build-time readers: Sphinx `literalinclude`, AsciiDoc `include::`, mdBook `{{#include}}`, pymdownx.snippets (a Markdown preprocessor, so it "also processes content inside fenced code blocks"), remark-code-import (`readFileSync` by default), Docusaurus raw-loader, Expressive Code `?raw`, Hugo `readFile`, Jekyll includes, Doxygen `\snippet`, Javadoc `{@snippet}`. — [Sphinx](https://www.sphinx-doc.org/en/master/usage/restructuredtext/directives.html); [Asciidoctor](https://docs.asciidoctor.org/asciidoc/latest/directives/include-tagged-regions/); [mdBook](https://rust-lang.github.io/mdBook/format/mdbook.html); [PyMdown Snippets](https://facelessuser.github.io/pymdown-extensions/extensions/snippets/); [remark-code-import](https://github.com/kevin940726/remark-code-import); [Docusaurus](https://docusaurus.io/docs/markdown-features/react); [Expressive Code](https://expressive-code.com/key-features/code-component/); [Hugo](https://gohugo.io/functions/os/readfile/); [Jekyll](https://jekyllrb.com/docs/includes/); [Doxygen](https://www.doxygen.nl/manual/commands.html); [JEP 413](https://openjdk.org/jeps/413)
- docusaurus-theme-github-codeblock is a third kind: "The plugin will download the code and display the desired lines" from a public GitHub URL. — [docusaurus-theme-github-codeblock README](https://github.com/saucelabs/docusaurus-theme-github-codeblock)
- A prospective Bluehawk user describes the appeal of the copy model: "If we switched to Bluehawk (so that all code snippets would be browsable in our documentation in one folder)". — [Bluehawk #166](https://github.com/mongodb-university/Bluehawk/issues/166)
- Microsoft states the reason to prefer file references over inline code: "Inline code is generally more difficult to test and keep up to date compared to a code file that is part of a complete project." — [Microsoft Learn](https://learn.microsoft.com/en-us/contribute/content/code-in-docs)

### Inferences
- MongoDB runs both models in sequence: Bluehawk copies snippet files into the docs tree, then the site build reads those files with `literalinclude` (the meta docs recommend `literalinclude` or `io-code-block` per the search summary of the Grove pages; I confirmed the copy step at the primary page, not the `literalinclude` recommendation).
- The copy model's recurring cost is a second source of truth that can lag: every copy tool grew a verify flag (`embedme --verify`, `mdsh --frozen`, Bluehawk's requested "ensure snip was run" action), and MongoDB's script re-snips everything on each run to avoid tracking what changed.
- The copy model's benefits, as stated by users: the docs repository builds without the code repositories present, snippets are browsable and reviewable as diffs in pull requests, and README files on GitHub (which has no build step) can show real code.

### Gaps
- I found no blog post or conference talk from a docs team explaining why it adopted or left one of these tools; two searches for a Bluehawk "why we built it" post returned nothing. User opinion here comes only from issue trackers and tool documentation.
- cog's documentation site (cog.readthedocs.io) was not read; its marker syntax (`[[[cog` ... `]]]` ... `[[[end]]]`) and `--check` flag are from memory and unconfirmed here.

## 4. How does each handle code that lives in a different repository or a different version than the docs?

### Takeaway
Three approaches exist: mount another repository into the build under a path name (Microsoft Learn's dependent repositories, Antora's components), fetch by URL at build or run time (pymdownx, mkdocs-include-markdown, markdown-magic, MarkdownSnippets, snipsync, docusaurus-theme-github-codeblock), or none (most tools read only the local file system). Pinning to a commit is explicit only in snipsync (`ref`) and in URL-based tools when the URL carries a commit.

### Cited Findings

- Microsoft Learn: a code repository is declared a "dependent repository" in `.openpublishing.publish.config.json` with `path_to_root`, `url`, `branch`, and `branch_mapping`; "That name then acts like a folder name for purposes of code references", used as `source="~/samples-durable-functions/samples/csx/shared/Location.csx"`. — [Microsoft Learn](https://learn.microsoft.com/en-us/contribute/content/code-in-docs)
- Microsoft's caution: "If you are including code blocks from another repository, work with the owners on a maintenance strategy for the code so that your included code does not break or go out of date as new versions of the libraries the code uses are shipped." — [Microsoft Learn](https://learn.microsoft.com/en-us/contribute/content/code-in-docs)
- Antora: examples live in a module's `examples` directory and are included by resource ID, `include::version@component:module:example$file[tag=...]`. If the version is omitted for another component, "Antora uses the latest version of the target example's component"; with version and component both omitted, the current page's are used. `lines`, `tag`, and `tags` attributes apply. — [Antora: Include an example](https://docs.antora.org/antora/latest/page/include-an-example/)
- snipsync: `origins` list GitHub repositories with `owner`, `repo`, and optional `ref` (the example pins a full commit SHA); "If the `ref` key is left blank or not specified, then the most recent commit from the main branch will be used." Local file globs are also allowed. — [snipsync README](https://github.com/temporal-community/snipsync)
- pymdownx.snippets: URL snippets (9.5) need `url_download: true`; options `url_max_size` (about 32 MiB default), `url_timeout` (10 s), `url_request_headers`; HTTP 429 retried with backoff (`max_retries` 3, added in 10.16). Nested snippets inside a URL snippet must also be URLs. Stated risk: with `url_download` it can make outbound HTTP requests (SSRF-style risk), and the extension is not for untrusted Markdown. — [PyMdown Snippets](https://facelessuser.github.io/pymdown-extensions/extensions/snippets/)
- mkdocs-include-markdown-plugin: `include_from_url: true` allows URLs; `cache` sets an expiry in seconds for cached HTTP requests, stored via platformdirs or `cache_dir`. — [mkdocs-include-markdown-plugin README](https://github.com/mondeja/mkdocs-include-markdown-plugin)
- markdown-magic: `REMOTE` and `CODE` can read remote URLs; options `failOnMissingRemote` (default `true`), `allowPrivateGithub` (default `false`), and a remote cache with `ttl` 5 minutes and `immutableTtl` 30 days for "GitHub files pinned to a full 40-character commit SHA". — [markdown-magic README](https://github.com/DavidWells/markdown-magic)
- MarkdownSnippets: `snippet: https://...` downloads a whole remote file (cached in `%temp%MarkdownSnippets`, at most 100 files); `web-snippet: URL#key` pulls one named snippet from remote content. — [MarkdownSnippets README](https://github.com/SimonCropp/MarkdownSnippets)
- docusaurus-theme-github-codeblock: references "code examples from public GitHub repositories" by blob URL with a line range. — [docusaurus-theme-github-codeblock README](https://github.com/saucelabs/docusaurus-theme-github-codeblock)
- Bluehawk has no remote reading documented; an open question from a team pulling snippets from 67 external repositories asks whether "our compilation script first have to download every single code repository locally, or can Bluehawk generate snippets from a file in Github at a URL" (opened 2026-05-20, no answer captured). — [Bluehawk #166](https://github.com/mongodb-university/Bluehawk/issues/166)
- MongoDB's current layout keeps tested examples in the docs repository itself (`code-example-tests/<language>/<suite>/examples` snipped to `content/code-examples/tested/...`). — [MongoDB Meta: Format and Snip Examples](https://www.mongodb.com/docs/meta/grove/code-testing/snip-tested-examples.md)
- Local-only by restriction: pymdownx `restrict_base_path` (default `True`) requires snippets to be under the base path; Jekyll `include_relative` cannot use `../`; Javadoc looks in `snippet-files` or `--snippet-path`; Doxygen uses `EXAMPLE_PATH`. — [PyMdown Snippets](https://facelessuser.github.io/pymdown-extensions/extensions/snippets/); [Jekyll](https://jekyllrb.com/docs/includes/); [JEP 413](https://openjdk.org/jeps/413); [Doxygen](https://www.doxygen.nl/manual/commands.html)
- Hugo `readFile` resolves relative to the project root, then each module's root, then the `content` directory of the unified file system. — [Hugo os.ReadFile](https://gohugo.io/functions/os/readfile/)
- AWS: the tooling "is used by the AWS Doc SDK Examples team, as well as tributary sources of example snippets", and the GitHub Action can be pinned to a dated release tag (`@2024-08-26-A`) rather than `main`. — [aws-doc-sdk-examples-tools README](https://github.com/awsdocs/aws-doc-sdk-examples-tools)

### Inferences
- "Version" means two different things across these tools: the version of the docs (Antora's component versions, Learn's `branch_mapping`) and the commit of the code (snipsync `ref`, a SHA in a URL). Only Antora ties the two together in the reference syntax itself.
- A branch-tracked reference (Learn's `"branch": "main"`, snipsync without `ref`, a URL to `main`) makes builds non-reproducible: the docs can change with no commit in the docs repository. Microsoft's guidance to "work with the owners on a maintenance strategy" is an admission that the tool does not solve this.
- Line-number references into another repository at a moving branch (docusaurus-theme-github-codeblock) combine the two weakest choices.

### Gaps
- Antora's content-source configuration (multiple repositories, `branches`, `tags`, `start_path`, clone cache): the playbook page URL I tried returned 404. How branches and tags map to component versions is not confirmed here.
- How `branch_mapping` works in Microsoft's build, and whether the dependent repository can be pinned to a commit: not stated on the page read.
- Sphinx and mdBook: no cross-repository feature was found in the pages read; users typically use git submodules, which I did not source.
- Asciidoctor's `allow-uri-read` for `include::https://...`: not confirmed at a primary page in this pass.

## 5. What do they report when a region is missing or changed?

### Takeaway
A missing file or region is a warning in most build-time tools and an error only where configured (pymdownx `check_paths`, Bluehawk `check`, AWS and Google CI checks). None of the tools read here detects that a region's content changed in meaning; the closest are Javadoc's hybrid-snippet equality check and the copy tools' "output differs" checks.

### Cited Findings

- Asciidoctor logs warnings: "tag 'x' not found in include file", "mismatched end tag (expected 'a' but found 'b')", "unexpected end tag", "detected unclosed tag 'x' starting at line N"; a missing include file is logged at error level ("include file not found"), or at info level if the include is marked optional. — [asciidoctor/lib/asciidoctor/reader.rb](https://github.com/asciidoctor/asciidoctor/blob/main/lib/asciidoctor/reader.rb)
- Sphinx messages in `literalinclude`: "Include file '%s' not found or reading it failed", "Object named %r not found in include file %r" (pyobject), "Line spec %r: no lines pulled from include file %r", "line number spec is out of range(1-%d): %r", "Cannot use \"lineno-match\" with a disjoint set of \"lines\"". — [sphinx/directives/code.py](https://github.com/sphinx-doc/sphinx/blob/master/sphinx/directives/code.py)
- pymdownx.snippets: `check_paths` defaults to `False`; when `True`, the build fails. Without it, a snippet whose file is missing is removed from the output silently. Error texts: "Snippet at path '...' could not be found", "Snippet section '...' could not be located", "Cannot download snippet '...' (HTTP Error N)". — [PyMdown Snippets](https://facelessuser.github.io/pymdown-extensions/extensions/snippets/); [pymdownx/snippets.py](https://github.com/facelessuser/pymdown-extensions/blob/main/pymdownx/snippets.py)
- Bluehawk `check` "exits with a non-zero status if processing any input file produces a Bluehawk error". Known reporting defect: "Double errors printed on mismatch commands" (open since 2022-07-01). — [Bluehawk CLI reference](https://mongodb-university.github.io/Bluehawk/reference/cli); [Bluehawk #116](https://github.com/mongodb-university/Bluehawk/issues/116)
- Javadoc: unrecognized `@` names in markup comments are ignored, errors in recognized tags are reported; a hybrid snippet whose inline and external forms differ is an error. "The Standard Doclet does not compile or otherwise test snippets." — [JEP 413](https://openjdk.org/jeps/413); [Oracle: Programmer's Guide to Snippets](https://docs.oracle.com/en/java/javase/21/javadoc/snippets.html)
- AWS check-in tests fail the pull request on unpaired or duplicate snippet tags and when a `snippet_file` named in metadata is not present in the repository; they also scan for strings that look like secret access keys. — [aws-doc-sdk-examples-tools README](https://github.com/awsdocs/aws-doc-sdk-examples-tools)
- Google snippet-bot reports mismatched region tags as a GitHub Check and posts a pull request comment summarizing region tag changes; it supports a full-repository scan by opening an issue titled "snippet-bot full scan". — [snippet-bot README](https://github.com/googleapis/repo-automation-bots/tree/main/packages/snippet-bot)
- Antora "will report an error because it won't be able to find the example" when the `example$` family coordinate is left out. — [Antora: Include an example](https://docs.antora.org/antora/latest/page/include-an-example/)
- Drift checks in copy tools: `embedme --verify`, `mdsh --frozen`, markdown-magic `failOnMissingRemote` (default `true`). — [embedme README](https://github.com/zakhenry/embedme); [mdsh README](https://github.com/zimbatm/mdsh); [markdown-magic README](https://github.com/DavidWells/markdown-magic)
- Microsoft on silent drift with line numbers: "You don't necessarily get notified of such changes." — [Microsoft Learn](https://learn.microsoft.com/en-us/contribute/content/code-in-docs)
- mdBook's include preprocessor logs an error on a failed include ("Error updating ...") and continues. — [mdBook links.rs](https://github.com/rust-lang/mdBook/blob/master/crates/mdbook-driver/src/builtin_preprocessors/links.rs)

### Inferences
- Detection covers three different failures, and tools rarely cover more than the first: (a) the reference no longer resolves (missing file, tag, object); (b) the copied text is out of date; (c) the code still resolves but no longer shows what the prose says. Nothing here addresses (c); line-number tools cannot even detect (a) unless the range runs past the end of the file.
- Defaults lean toward a build that succeeds with a hole in it: Asciidoctor warns, pymdownx drops the snippet silently unless `check_paths` is on. A missing snippet becomes a published blank unless warnings are made fatal.
- Google's and AWS's tooling checks tags on the code side, in the code repository's pull requests, which is where a rename or deletion happens; the build-time tools only notice on the docs side at the next build.

### Gaps
- Sphinx's behaviour when `start-after` / `end-before` text is not found: the directive documentation does not say, and the message was not among the translated strings I listed from the source. Unconfirmed.
- mdBook's behaviour when a named anchor does not exist in the file (empty output, warning, or error): not confirmed.
- MarkdownSnippets' behaviour on missing or duplicate snippet keys: not on the README page read.
- DocFX's and Microsoft Learn's build messages for a missing `id` or region: not found.
- Doxygen's warning for a missing `[block_id]`: not confirmed.
- Whether snippet-bot knows a region tag is in use by published docs before warning on its removal: its README does not say.

## 6. Which are actively maintained and widely used, and which are abandoned?

### Takeaway
The built-in features of large generators and a few standalone tools (MarkdownSnippets, markdown-magic, mkdocs-include-markdown-plugin, pymdown-extensions) are active in 2026. Bluehawk is in maintenance: no release since October 2024 and no feature commits since, though MongoDB's docs process still depends on it. embedme, remark-code-import, markdown-autodocs, and docusaurus-theme-github-codeblock have had no release in two or more years but are still downloaded.

### Cited Findings

Status table (GitHub API and registry APIs, read 2026-10-09):

| Tool | Latest release | Last push | License | Stars | Downloads |
|---|---|---|---|---|---|
| Bluehawk | 1.6.0, 2024-10-18 | 2025-11-27 | Apache-2.0 (per LICENSE.txt; GitHub shows "Other") | 35 | npm 2,309 / month |
| MarkdownSnippets | 28.5.0, 2026-09-27 | 2026-10-09 | MIT | 249 | NuGet tool 410,878 total |
| embedme | v1.22.1, 2022-09-07 | 2024-10-06 | MIT | 238 | npm 20,985 / month |
| markdown-magic | 4.11.0, 2026-06-29 | 2026-07-27 | MIT (package.json) | 871 | npm 60,421 / month |
| markdown-autodocs | v1.0.7, 2022-08-29 | 2026-01-31 | MIT | 196 | npm 2,066 / month |
| mdsh | tag v0.9.2 (no GitHub release) | 2026-07-23 | MIT | 174 | not checked |
| cog | tag v3.6.0 (no GitHub release) | 2026-08-05 | MIT | 407 | PyPI 245,827 / month |
| snipsync | no GitHub release | 2026-07-29 | none declared | 83 | npm 1,218 / month |
| remark-code-import | v1.1.0, 2022-01-16 | 2024-07-11 | MIT | 73 | npm 156,478 / month |
| docusaurus-theme-github-codeblock | 0.3.0, 2024-10-09 | 2024-10-09 | MIT | 104 | npm 25,707 / month |
| mdBook | v0.5.4, 2026-07-06 | 2026-10-05 | MPL-2.0 | 22,206 | not checked |
| pymdown-extensions | 12.1, 2026-09-23 | 2026-10-07 | "Other" per GitHub | 1,132 | PyPI 18,328,199 / month |
| mkdocs-include-markdown-plugin | v7.3.0, 2026-05-15 | 2026-07-01 | Apache-2.0 | 154 | PyPI 433,065 / month |
| mkdocs-codeinclude-plugin | 0.3.1, 2026-02-20 | 2026-02-20 | MIT | 18 | PyPI 33,289 / month |
| Expressive Code | 0.44.2, 2026-08-31 | 2026-10-07 | MIT | 967 | npm @expressive-code/core 5,513,583 / month |
| Sphinx | v9.1.0, 2025-12-31 | 2026-10-05 | "Other" per GitHub | 8,057 | not checked |
| Asciidoctor | v2.0.26, 2025-10-24 | 2026-09-01 | "Other" per GitHub | 5,220 | not checked |
| DocFX | v2.81.0, 2026-09-25 | 2026-10-04 | MIT | 4,447 | not checked |
| Doxygen | 1.18.0, 2026-08-13 | 2026-09-30 | GPL-2.0 | 6,589 | not checked |
| aws-doc-sdk-examples | n/a | 2026-10-09 | Apache-2.0 | 10,466 | n/a |
| aws-doc-sdk-examples-tools | dated tags | 2026-10-01 | Apache-2.0 | 14 | n/a |
| googleapis/repo-automation-bots (snippet-bot) | n/a | 2026-10-06 | Apache-2.0 | 747 | n/a |

- Sources for the table: [Bluehawk](https://github.com/mongodb-university/Bluehawk), [Bluehawk releases](https://github.com/mongodb-university/Bluehawk/releases), [Bluehawk LICENSE.txt](https://github.com/mongodb-university/Bluehawk/blob/main/LICENSE.txt), [npm bluehawk](https://api.npmjs.org/downloads/point/last-month/bluehawk); [MarkdownSnippets](https://github.com/SimonCropp/MarkdownSnippets), [NuGet MarkdownSnippets.Tool](https://www.nuget.org/packages/MarkdownSnippets.Tool); [embedme](https://github.com/zakhenry/embedme), [npm embedme](https://api.npmjs.org/downloads/point/last-month/embedme); [markdown-magic](https://github.com/DavidWells/markdown-magic), [npm markdown-magic](https://api.npmjs.org/downloads/point/last-month/markdown-magic); [markdown-autodocs](https://github.com/dineshsonachalam/markdown-autodocs), [npm markdown-autodocs](https://api.npmjs.org/downloads/point/last-month/markdown-autodocs); [mdsh](https://github.com/zimbatm/mdsh); [cog](https://github.com/nedbat/cog), [PyPI Stats cogapp](https://pypistats.org/packages/cogapp); [snipsync](https://github.com/temporal-community/snipsync), [npm snipsync](https://api.npmjs.org/downloads/point/last-month/snipsync); [remark-code-import](https://github.com/kevin940726/remark-code-import), [npm remark-code-import](https://api.npmjs.org/downloads/point/last-month/remark-code-import); [docusaurus-theme-github-codeblock](https://github.com/saucelabs/docusaurus-theme-github-codeblock), [npm](https://api.npmjs.org/downloads/point/last-month/docusaurus-theme-github-codeblock); [mdBook](https://github.com/rust-lang/mdBook); [pymdown-extensions](https://github.com/facelessuser/pymdown-extensions), [PyPI Stats](https://pypistats.org/packages/pymdown-extensions); [mkdocs-include-markdown-plugin](https://github.com/mondeja/mkdocs-include-markdown-plugin), [PyPI Stats](https://pypistats.org/packages/mkdocs-include-markdown-plugin); [mkdocs-codeinclude-plugin](https://github.com/rnorth/mkdocs-codeinclude-plugin), [PyPI Stats](https://pypistats.org/packages/mkdocs-codeinclude-plugin); [Expressive Code](https://github.com/expressive-code/expressive-code), [npm](https://api.npmjs.org/downloads/point/last-month/@expressive-code/core); [Sphinx](https://github.com/sphinx-doc/sphinx); [Asciidoctor](https://github.com/asciidoctor/asciidoctor); [DocFX](https://github.com/dotnet/docfx); [Doxygen](https://github.com/doxygen/doxygen); [aws-doc-sdk-examples](https://github.com/awsdocs/aws-doc-sdk-examples); [aws-doc-sdk-examples-tools](https://github.com/awsdocs/aws-doc-sdk-examples-tools); [repo-automation-bots](https://github.com/googleapis/repo-automation-bots)

**Bluehawk status in detail**

- License: LICENSE.txt reads "Copyright 2021 MongoDB, Inc. Licensed under the Apache License, Version 2.0". — [Bluehawk LICENSE.txt](https://github.com/mongodb-university/Bluehawk/blob/main/LICENSE.txt)
- The eight most recent commits (2025-08-11 to 2025-11-27) are a build fix, a docs dependency update, and a notification workflow; none changes Bluehawk's behaviour. — [Bluehawk commits](https://github.com/mongodb-university/Bluehawk/commits/main)
- 24 open issues; most date from 2022 (config files, VS Code extension, templated output paths, manifest of code block IDs, Dockerfile support, `:ignore:` tag). Newer ones: `--preserveDirs` and a `snipOrCopy` command (2025-07), remote files (2026-05). — [Bluehawk issues](https://github.com/mongodb-university/Bluehawk/issues)
- Open defects: "Exporting as MD fails" (`--format md` errors on a minimal file, open since 2022-09-30); "Emphasize lines are duplicated in formatted files" (2023-11-07). — [Bluehawk #131](https://github.com/mongodb-university/Bluehawk/issues/131); [Bluehawk #148](https://github.com/mongodb-university/Bluehawk/issues/148)
- MongoDB still uses it: the current docs contributor guide says the snip script "uses the Bluehawk CLI to extract and transform the relevant code" and tells writers to "Ensure that you have Bluehawk installed by running `npm install -g bluehawk`" (the page also requires Node.js 24 or newer). — [MongoDB Meta: Format and Snip Examples](https://www.mongodb.com/docs/meta/grove/code-testing/snip-tested-examples.md)
- A GitHub code search for "bluehawk" in the `mongodb` organization returned 164 hits on 2026-10-09, including `content/kotlin/current/examples/bluehawk.sh` and agent skill reference files `bluehawk-tags.md` and `bluehawk-syntax.md` in `mongodb/docs`. — [GitHub code search](https://github.com/search?q=org%3Amongodb+bluehawk&type=code)

**Others**

- JEP 413 is "Closed / Delivered" in JDK 18. — [JEP 413](https://openjdk.org/jeps/413)
- Docusaurus still labels raw-loader code imports experimental. — [Docusaurus: MDX and React](https://docusaurus.io/docs/markdown-features/react)
- docusaurus-theme-github-codeblock's README still says it is "A Docusaurus v2 plugin". — [docusaurus-theme-github-codeblock README](https://github.com/saucelabs/docusaurus-theme-github-codeblock)
- remark-code-import has been ESM-only since v1.0.0. — [remark-code-import README](https://github.com/kevin940726/remark-code-import)
- MarkdownSnippets' tool requires .NET 10 or higher. — [MarkdownSnippets README](https://github.com/SimonCropp/MarkdownSnippets)
- snipsync moved from `temporalio/snipsync` to `temporal-community/snipsync` (the GitHub API returns the latter name for the former path). — [snipsync](https://github.com/temporal-community/snipsync)

### Inferences
- Bluehawk is best described as dormant but depended on: one company uses it in a documented daily process, the public issue tracker has feature requests from 2022 unanswered, and its npm downloads (about 2,300 a month) are an order of magnitude below embedme's and two below remark-code-import's. Adoption outside MongoDB looks small.
- Downloads do not track maintenance: remark-code-import has had no release since January 2022 yet has about 156,000 monthly downloads, likely because it is a small transitive dependency that needs no changes.
- The stable choices are features built into a generator a team already uses. Standalone tools are mostly single-maintainer projects (MarkdownSnippets, markdown-magic, cog, mdsh, embedme).
- No tool in this family is both language-neutral and rich in transformations and actively developed: Bluehawk has the transformations but is dormant; Javadoc `{@snippet}` is maintained but Java-only and tied to API docs.

### Gaps
- "Latest release" for Sphinx (v9.1.0, 2025-12-31) and Asciidoctor (v2.0.26, 2025-10-24) is what the GitHub releases API marks latest; newer versions may exist on PyPI or RubyGems and were not checked.
- remark-code-import's and embedme's latest npm versions were not checked against their GitHub releases.
- Licenses GitHub reports as "Other" (pymdown-extensions, Sphinx, Asciidoctor) were not read individually.
- Whether MongoDB staffs Bluehawk maintenance, and any plan to replace it, is not stated in any source found.
- Antora's repository is on GitLab and was not queried; its release status is unconfirmed here.
- Adoption figures for built-in features (how many Sphinx or mdBook projects use region includes) do not exist in any source found.
