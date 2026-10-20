What the project says to the crawlers that read its site: search engines, AI assistants, and anything else that fetches pages without a person. Each build writes it into its page records, `_ascribe/pages.json` in the site output ([output-layout contract](../../contracts/output-layout.md) §1.3), for a site to write its `robots.txt` from; `@ascribed/astro` writes it ([the `@ascribed/astro` reference](../../astro/reference.md#search-engines-and-crawlers)).

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

@include: ../../_generated/content-model-crawlers.md

`allow` writes nothing, and neither does leaving a purpose out: a group that named those crawlers and said `Allow: /` would take them out of the `*` group, and with it out of every path `lines` disallows, drafts and all. So an allowed crawler stays under the lines for every crawler. `allow` is the project writing its choice down. A project that says nothing gets no rules about AI crawlers.

`robots.txt` has the purposes' groups first, in the order above, then the `User-agent: *` group with the lines for every crawler, then the groups `lines` writes whole, then the sitemap's address.
