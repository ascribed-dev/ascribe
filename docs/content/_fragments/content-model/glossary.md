The glossary ([SPEC §5.4]({repo}/blob/main/SPEC.md#54-glossary-terms)): terms, their definitions, and how occurrences are matched. Authors don't mark terms in source; processors link them ([SPEC §9.2]({repo}/blob/main/SPEC.md#92-resolution) step 7).

@snippet: code:examples/content-models/full.toml#glossary

### 12.1 Settings

| Key | Type | Default | Description |
|---|---|---|---|
| `match` | string | `"first"` | Which occurrences are linked: `"first"`, the first occurrence of each term on each page, or `"every"`, every occurrence. "Page" means the resolved page for a build, after includes and build modes. A term can override it. |
| `case-sensitive` | boolean | `false` | Whether occurrences must match a term's case exactly. When `false`, matching ignores case. A term can override it. |
| `terms` | table of terms | `{}` | The terms, keyed by term id (`key` rule). |

### 12.2 Terms

| Key | Type | Default | Description |
|---|---|---|---|
| `term` | string | **required** | The term as it appears in prose. Non-empty. |
| `aliases` | array of strings | `[]` | Other forms that count as occurrences, such as plurals. |
| `definition` | string | **required** | A short plain-text definition, shown on hover in the editor and included in the JSON output. |
| `link` | string (path) | none | Where the full definition lives: a source file path relative to the content root, with an optional `#id`, written as in a link ([SPEC §5.2]({repo}/blob/main/SPEC.md#52-links); a leading `/` is allowed and means the same). It MUST name a page, not a fragment. Occurrences are linked here. |
| `case-sensitive` | boolean | the `[glossary]` setting | Overrides `case-sensitive` for this term, for terms like `Go` that collide with ordinary words. |
| `match` | string | the `[glossary]` setting | Overrides `match` for this term: `"first"`, `"every"`, or `"marked"`. A `"marked"` term isn't linked automatically at all, for words like `build` that also have ordinary senses: link the occurrences that mean the term with an ordinary link to its page. `"marked"` is a term setting only; `[glossary]` doesn't accept it. |
