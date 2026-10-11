# Markup comparison: one page, four dialects

The same fictional page, "Install the Quill agent" (Quill is a made-up docs-sync tool), written four ways so the markup can be compared directly:

| File                                                 | Dialect                                                 |
| ---------------------------------------------------- | ------------------------------------------------------- |
| [ascribe/install-agent.md](ascribe/install-agent.md) | Ascribe, under this folder's [ascribe.toml](ascribe.toml) |
| [astro.mdx](astro.mdx)                               | Astro + Starlight, as withastro/docs writes it          |
| [elastic.md](elastic.md)                             | Elastic docs-builder, as elastic/docs-content writes it |
| [docker.md](docker.md)                               | Hugo, as docker/docs writes it                          |

The Ascribe version is the page from [SPEC Appendix B](../../SPEC.md#appendix-b-complete-example), and a test keeps the two identical. It's a valid Ascribe project: `ascribe check examples/comparison` passes, and a test runs it. So the pages it includes and links to are here too, as small stand-ins (`ascribe/_fragments/prerequisites.md`, `ascribe/quickstart.md`, and `ascribe/keys.md`). [`examples/quill`](../quill) is the same page in a fuller project, with builds and images.

The other three versions are other tools' formats, and nothing here checks or builds them. Their include targets (`_snippets/prerequisites.md`, `quill-prerequisites.md`, and the `Prerequisites` component) and link targets aren't included; they're referenced only for their syntax.

## Construct by construct

| Construct                                    | Ascribe                                                                           | Astro / Starlight                                     | Elastic                                                                  | Docker                                 |
| -------------------------------------------- | --------------------------------------------------------------------------------- | ----------------------------------------------------- | ------------------------------------------------------------------------ | -------------------------------------- |
| Page applicability                           | `available: cloud, self-managed preview 3.3` frontmatter                          | `<Badge>` components (display only)                   | `applies_to:` frontmatter                                                | `{{< summary-bar >}}` → registry       |
| Product-name substitution                    | `{product}`                                                                       | hardcoded                                             | `{{quill}}`                                                              | hardcoded                              |
| Titled tip                                   | `.Try it without installing` title line above `@note {type=tip}`                  | `:::tip[Title]`                                       | `:::{admonition} Title`                                                  | `> [!TIP]` (titles unused)             |
| Heading id                                   | `@id: install-agent` on the next line                                             | auto-slug only                                        | `## Title [id]`                                                          | `## Title {#id}`                       |
| Include                                      | `@include: _fragments/prerequisites.md`                                           | MDX import + `<Prerequisites />`                      | `:::{include} path`                                                      | `{{% include "file" %}}`               |
| Steps                                        | `@steps` above a plain ordered list                                               | `<Steps>` wrapper                                     | `{stepper}` > `{step}`                                                   | plain ordered list                     |
| Package-manager variants                     | one `@variant {pm=…}:` group: 3 arms, 1 `@end`                                    | `<PackageManagerTabs>` + slots                        | `{tab-set}` > `{tab-item}`                                               | `{{< tabs group= >}}`                  |
| Callout inside a step                        | `@note`, indented in the list item                                                | indented `:::note`                                    | `:::{note}` inside `{step}`                                              | indented `> [!NOTE]`                   |
| Version inside code                          | `{version}` in a fence marked `phrases=true`                                      | hardcoded                                             | `{{quill-version}}` + `subs=true`                                        | `{{% param "quill_version" %}}`        |
| Rich deployment variants                     | one `@variant {deployment=…}:` group: 2 arms, 1 `@end`                            | `<Tabs syncKey>`                                      | `{applies-switch}` > `{applies-item}`                                    | `{{< tabs group= >}}`                  |
| Section applicability                        | `@available: cloud, self-managed preview 3.4` under the heading                   | `<Badge>`                                             | `` {applies_to}`ess: ga` `` inline                                       | `{{< summary-bar >}}`                  |
| Phrase glued to `'s`                         | `{cloud}'s`                                                                       | hardcoded                                             | `{{quill-cloud}}'s`                                                      | hardcoded                              |
| Substitution in a URL                        | `]({api}streaming)`                                                               | hardcoded                                             | `]({{quill-api}}streaming)`                                              | hardcoded                              |
| Multi-block warning                          | `@note {type=warning}:` … `@end`                                                  | `:::caution` … `:::`                                  | `:::{warning}` … `:::`                                                   | `> [!WARNING]` + `>` on every line     |
| Cross-reference                              | `[](keys.md#rotate-keys)`: a file path, its text taken from the target heading    | link to site route (`/en/…/#slug`)                    | link to file path (`/…/x.md#id`)                                         | link to file path (`/manuals/…md#id`)  |
| Max directive/component nesting on this page | 1 (`@variant` inside a list item)                                                 | 3 (`<Steps>` > `<PackageManagerTabs>` > `<Fragment>`) | 4 (`stepper` > `step` > `tab-set` > `tab-item`; outer fence is 6 colons) | 2 (`tabs` > `tab`, inside a list item) |

In the Ascribe version, fenced code is literal unless the fence opts in with `phrases=true`, so the cloud fence keeps `${QUILL_KEY}` as written. `ascribe check` reports a link, include, or heading id that doesn't exist, an undeclared dimension or value in a `@variant` or availability spec, and a section whose availability is wider than its page's.

## Registries each version depends on

Substitutions and applicability live outside the page. What each version assumes:

**Ascribe**: [ascribe.toml](ascribe.toml), the project's content model. Its `[phrases]` section holds `product`, `cloud`, `version`, and `api`, and its `[dimensions]` section declares `pm` and `deployment`, which the `@variant` arms and the availability specs use. `@available` can also name a feature declared in a `[features]` section, so that one edit to the feature updates every section that names it; this page doesn't use one.

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
