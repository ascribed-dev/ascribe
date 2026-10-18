The glossary ([SPEC §5.4]({repo}/blob/main/SPEC.md#54-glossary-terms)): the pages that hold its terms, and how occurrences are matched. A term is a heading on a glossary page, marked [`@term`](../../writing/directives.md#term), and its definition is the paragraph under it. Authors don't mark terms where they're used; processors link them ([SPEC §9.2]({repo}/blob/main/SPEC.md#92-resolution) step 7).

@snippet: code:examples/content-models/full.toml#glossary

### 12.1 Settings

| Key | Type | Default | Description |
|---|---|---|---|
| `pages` {available="ascribe 0.4.0"} | array of strings (paths) | `[]` | The glossary pages, each a path from the content root, written as in a link ([SPEC §5.2]({repo}/blob/main/SPEC.md#52-links); a leading `/` is allowed and means the same). Each MUST be a page, not a fragment. A term is linked to the first of its pages that a build publishes. |
| `match` | string | `"first"` | Which occurrences are linked: `"first"`, the first occurrence of each term on each page, or `"every"`, every occurrence. "Page" means the resolved page for a build, after includes and build modes. A term can override it. |
| `case-sensitive` | boolean | `false` | Whether occurrences must match a term's case exactly. When `false`, matching ignores case. A term can override it. |

@available: ascribe 0.4.0
There's no `terms` table: a project that has one gets an error saying to write each term on a glossary page.

### 12.2 Terms
@available: ascribe 0.4.0

Each heading on a glossary page with `@term` directly under it, written on the page or in a fragment it includes, is a term:

```markdown
## Rollout
@id: rollout
@term {aliases=rollouts}

A plan for turning a flag on for a growing share of users: by percentage, by segment, or on a schedule.
```

- **The term** is the heading's text, and **its id** is the heading's id, which the site output's `data-ascribe-term` names. An `@id` keeps the id when the heading's text changes.
- **The definition** is the paragraph that opens the section, after the heading's directives, as plain text, with the build's phrases. A paragraph a fragment brings in counts, and a section that opens with a `@variant` group has the paragraph each arm opens with, in the builds that keep the arm. The site output shows it as the link's title, and the editor on hover; the JSON output lists it with each use. Keep it to a sentence or two, and put the rest in a second paragraph.
- **A term follows its section in each build.** A build that removes the section, by `@available` or by a variant, has no such term, and a build that removes only its paragraph leaves it without a definition: either way, the build doesn't link it.
- `@term`'s attributes override the glossary's settings for the term:

| Attribute | Type | Default | Description |
|---|---|---|---|
| `aliases` | string | none | Other forms that count as occurrences, such as plurals, separated by `\|`: `aliases="API keys\|keys"`. |
| `case-sensitive` | boolean | the `[glossary]` setting | For terms like `Go` that collide with ordinary words. |
| `match` | `first`, `every`, or `marked` | the `[glossary]` setting | A `marked` term isn't linked automatically at all, for words like `build` that also have ordinary senses: link the occurrences that mean the term to its heading, as `[build](glossary.md#build)`. `marked` is a term setting only; `[glossary]` doesn't accept it. |

An unused term (`glossary-term-unused`) is acknowledged with [`@intended`](../../writing/directives.md#intended) among the directives under its heading.

**A link to a term's heading is a use of the term,** wherever it's written: it gets the term's marker and its definition as its title, as an automatic link does, and it counts as a use. For a term with `match = "first"`, it's the page's first link to the term, so no later occurrence is linked. A link to the glossary page, or to another heading in the term's section, is an ordinary link.
