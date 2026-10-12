What the project says to the crawlers that read its site: search engines, AI assistants, and anything else that fetches pages without a person. Each build writes it into its page records, `_ascribe/pages.json` in the site output ([output-layout contract](../../contracts/output-layout.md) §1.3), for a site to write its `robots.txt` from; `@ascribed/astro` writes it ([the Astro guide](../../guides/astro.md#search-engines-and-crawlers)).

```toml
[crawlers]
training = "disallow"
search = "allow"
on-request = "allow"
lines = [
  "Disallow: /drafts/",
  "User-agent: ExampleBot",
  "Disallow: /",
]
```

| Key | Type | Default | Description |
|---|---|---|---|
| `training` | `"allow"` or `"disallow"` | none | What crawlers gathering text to train a model may do. `disallow` writes one group naming every training crawler Ascribe knows, with `Disallow: /`. |
| `search` | `"allow"` or `"disallow"` | none | What crawlers building the index an AI search product answers from may do, written the same way. |
| `on-request` | `"allow"` or `"disallow"` | none | What fetchers that read a page because a person asked an assistant about it may do, written the same way. |
| `lines` | array of strings | `[]` | The project's own `robots.txt` lines, in order. Each is one line, not empty. A path in an `Allow` or `Disallow` line is from the project's own root, and the records hold it from the host's, with `base-path` before it: under `base-path = "/docs/"`, `Disallow: /drafts/` is `Disallow: /docs/drafts/`. Any other line is kept as written. The lines before any `User-agent` line are for every crawler, in the `User-agent: *` group. A `User-agent` line that names a crawler starts a group the project writes whole: a crawler obeys only the most specific group that names it, and not the `*` group as well. A crawler a disallowed purpose names can't be named in `lines` too, and no crawler can be named in two groups of `lines`. |

`allow` writes nothing, and neither does leaving a purpose out: a group that named those crawlers and said `Allow: /` would take them out of the `*` group, and with it out of every path `lines` disallows, drafts and all. So an allowed crawler stays under the lines for every crawler. `allow` is the project writing its choice down. A project that says nothing gets no rules about AI crawlers.

`robots.txt` has the purposes' groups first, in the order above, then the `User-agent: *` group with the lines for every crawler, then the groups `lines` writes whole, then the sitemap's address.
