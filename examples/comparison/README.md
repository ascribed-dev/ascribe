# Markup comparison: one page, four dialects

The same fictional page, "Install the Quill agent" (Quill is a made-up docs-sync tool), written four ways so the markup can be compared directly:

| File                     | Dialect                                                          |
| ------------------------ | ---------------------------------------------------------------- |
| [tessera.md](tessera.md) | Tessera, **strictly per the current spec** (BRAINSTORM.md §6–§8) |
| [astro.mdx](astro.mdx)   | Astro + Starlight, as withastro/docs writes it                   |
| [elastic.md](elastic.md) | Elastic docs-builder, as elastic/docs-content writes it          |
| [docker.md](docker.md)   | Hugo, as docker/docs writes it                                   |

The page is built to hit every pressure point from the reaction test (BRAINSTORM.md §10). The Tessera version does not paper over the spec's gaps. Each place the spec can't express something is marked with a `<!-- GAP §12.n -->` comment (invisible when rendered), pointing at the open question that covers it.

Include targets (`_fragments/prerequisites.md` and its equivalents) and link targets are not included; they're referenced only for their syntax.

## Construct by construct

| Construct                                    | Tessera (current spec)                                                      | Astro / Starlight                                     | Elastic                                                                  | Docker                                 |
| -------------------------------------------- | --------------------------------------------------------------------------- | ----------------------------------------------------- | ------------------------------------------------------------------------ | -------------------------------------- |
| Page applicability                           | **gap** (§12.11; decided: `available:` frontmatter)                         | `<Badge>` components (display only)                   | `applies_to:` frontmatter                                                | `{{< summary-bar >}}` → registry       |
| Product-name substitution                    | `@phrase: product` (proposed: `{product}`)                                  | hardcoded                                             | `{{quill}}`                                                              | hardcoded                              |
| Titled tip                                   | `@note {type=tip}`, **title lost** (§12.9)                                  | `:::tip[Title]`                                       | `:::{admonition} Title`                                                  | `> [!TIP]` (titles unused)             |
| Heading id                                   | `@id: install-agent` on the next line                                       | auto-slug only                                        | `## Title [id]`                                                          | `## Title {#id}`                       |
| Include                                      | `@include {heading=false}: path`                                            | MDX import + `<Prerequisites />`                      | `:::{include} path`                                                      | `{{% include "file" %}}`               |
| Steps                                        | plain ordered list (proposed: `@steps` + list)                              | `<Steps>` wrapper                                     | `{stepper}` > `{step}`                                                   | plain ordered list                     |
| Package-manager variants                     | 3× `@variant … @end` (§12.13)                                               | `<PackageManagerTabs>` + slots                        | `{tab-set}` > `{tab-item}`                                               | `{{< tabs group= >}}`                  |
| Callout inside a step                        | `@note` (following block), indentation **unspecified** (§12.14)             | indented `:::note`                                    | `:::{note}` inside `{step}`                                              | indented `> [!NOTE]`                   |
| Version inside code                          | **gap**, hardcoded (§12.10)                                                 | hardcoded                                             | `{{quill-version}}` + `subs=true`                                        | `{{% param "quill_version" %}}`        |
| Rich platform variants                       | 2× `@variant … @end`                                                        | `<Tabs syncKey>`                                      | `{applies-switch}` > `{applies-item}`                                    | `{{< tabs group= >}}`                  |
| Inline applicability badge                   | **gap** (§12.11; decided: `@available`)                                     | `<Badge>`                                             | `` {applies_to}`ess: ga` ``                                              | `{{< summary-bar >}}`                  |
| Phrase glued to `'s`                         | **breaks**: parses as key `cloud's` (§12.10)                                | hardcoded                                             | `{{quill-cloud}}'s`                                                      | hardcoded                              |
| Substitution in a URL                        | **gap**, hardcoded (§12.10)                                                 | hardcoded                                             | `]({{quill-api}}streaming)`                                              | hardcoded                              |
| Multi-block warning                          | `@note {type=warning}:` … `@end`                                            | `:::caution` … `:::`                                  | `:::{warning}` … `:::`                                                   | `> [!WARNING]` + `>` on every line     |
| Cross-reference                              | `@ref: keys.md#rotate-keys` (auto text; decided: `[](keys.md#rotate-keys)`) | link to site route (`/en/…/#slug`)                    | link to file path (`/…/x.md#id`)                                         | link to file path (`/manuals/…md#id`)  |
| Max directive/component nesting on this page | 1 (`@variant` inside a list item)                                           | 3 (`<Steps>` > `<PackageManagerTabs>` > `<Fragment>`) | 4 (`stepper` > `step` > `tab-set` > `tab-item`; outer fence is 6 colons) | 2 (`tabs` > `tab`, inside a list item) |

## What changes in tessera-proposed.md

Ideas from the language survey (BRAINSTORM.md §5). Sibling grouping (§12.13) and phrase syntax (§12.10) are both decided.

**Siblings close siblings (§12.13).** A run of `@variant` openers is one group: each new opener ends the previous arm, and one `@end` closes the group (like Ruby's `if … elsif … end`).

- Package managers: 3 containers and 3 `@end`s become 1 group and 1 `@end`.
- Cloud vs self-managed: 2 `@end`s become 1, and the rich prose-and-code arms need nothing extra.

**Phrases as `{key}` placeholders (§12.10, now decided).** Phrases are no longer a directive. A declared key in braces is substituted; braces are always bounded, so they glue to punctuation, letters and hyphens and work inside URLs. Fenced code is literal unless the fence opts in with `phrases=true`.

| Current spec                                             | Proposed                                              |
| -------------------------------------------------------- | ----------------------------------------------------- |
| `@phrase: product`                                       | `{product}`                                           |
| `@phrase: cloud's` (broken: parses as the key `cloud's`) | `{cloud}'s`                                           |
| hardcoded `Quill Cloud-hosted`                           | `{cloud}-hosted`                                      |
| hardcoded API URL                                        | `]({api}streaming)`                                   |
| hardcoded `3.4.1` in a code fence                        | `{version}` in a fence marked `phrases=true`          |
| `` `3.4.1` `` in inline code                             | `{version}` as plain text (inline code stays literal) |

The unmarked cloud fence keeps `${QUILL_KEY}` literal, which is why code has to opt in.

**`@steps` (§12.12, decided).** The install procedure is marked with `@steps` directly above the ordered list; the list itself stays plain markdown.

**AsciiDoc-style titles (§12.9, decided).** The tip's title is a `.Try it without installing` line directly above `@note`.

Directives inside list items follow CommonMark's container rules (§12.14, decided).

**Applicability with `@available` (§12.11, decided).** The page's availability is a frontmatter spec, `available: cloud, self-managed preview 3.3`, and the streaming-sync section states its own with `@available` directly under its heading. The same spec string works in both places. Alternatively, a features registry could hold the spec so the section just says `@available: streaming-sync`.

**Empty-text links replace `@ref` (§12.15, decided).** `See [](keys.md#rotate-keys).` takes its text from the target heading, as Elastic's docs-builder does.

The proposed version has no remaining gap markers.

## Registries each version depends on

Substitutions and applicability live outside the page. What each version assumes:

**Tessera**: phrases registry (content model; format not yet specced):

```yaml
product: Quill
cloud: Quill Cloud
# used only by tessera-proposed.md:
version: 3.4.1
api: https://api.quill.dev/v3/
```

Features registry, if the registry form of `@available` is used:

```yaml
streaming-sync:
  name: Streaming sync
  available: cloud, self-managed preview 3.4
```

**Elastic**: `docset.yml`:

```yaml
subs:
  quill: "Quill"
  quill-cloud: "Quill Cloud"
  quill-version: "3.4.1"
  quill-api: "https://api.quill.dev/v3/"
```

The `applies_to` keys (`ess` = cloud-hosted, `self` = self-managed) are Elastic's real deployment dimensions, borrowed for the fake product.

**Docker**: `hugo.yaml` for the version param, and `data/summary.yaml` for the feature metadata:

```yaml
# hugo.yaml
params:
  quill_version: "3.4.1"
```

```yaml
# data/summary.yaml
Quill agent:
  availability: GA
  requires: Quill Server 3.3 or later (self-managed)
Quill streaming sync:
  availability: Beta
  requires: Quill Server 3.4 or later (self-managed)
```

**Astro**: none. Astro docs hardcode product names and versions, and has no applicability model.
