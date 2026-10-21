How pages are listed, set once for many: the whole site, the pages under a folder, or the pages of a content type. Each value is one of the values of a page's `listing` ([SPEC §2.1]({repo}/blob/main/SPEC.md#21-files)), from open to closed: `listed`, `unquoted`, `unlisted`.

```toml
[listing]
default = "listed"

[listing.folders]
"internal/" = "unlisted"

[listing.types]
changelog = "unquoted"
```

@include: ../../_generated/content-model-listing.md

A page's listing is the strictest of its own and the rules it's under: the site's `default`, every folder it's under, and its type's. A folder and a type have no order between them: a page under both gets the stricter. A rule can only restrict. A page can set the same value as its rule, or a stricter one, and a page that sets a looser one is an error at the page (`listing-loosened`), since the project contradicts itself. A folder under another folder, and a folder or a type under the `default`, can't be looser than it (`model-listing-looser`).

@available: ascribe 0.3.2
**`older-lines`** (a string, `"listed"` by default) is the listing of every page of a live line of a versioned build other than its current line: `site@3.4`'s, and not `site`'s. It governs search engines and agents together. A page in an older line is under it as well as the rules above, between `default` and a folder's or a type's; it can't be looser than `default` (`model-listing-looser`); and a page that sets a value looser than it is an error in each older line that publishes it (`listing-loosened-in-line`), and fine in the current line. An older line's pages point their canonical links at the current line's, so a search engine sends readers to the current line however it's set: `older-lines = "unlisted"` keeps an older line out of the index altogether, its pages out of the sitemap and `llms.txt`, for a project whose old pages still outrank its new ones.

A `default` stricter than `listed` is for a site that isn't public yet: no page in it can be listed until the default changes, and a page that says otherwise is the same error.

Each page's record, in `_ascribe/pages.json`, says where its listing came from: the page, the rule's level and its line in `ascribe.toml`, or neither ([output-layout contract](../../contracts/output-layout.md) §1.3).
