How pages are listed, set once for many: the whole site, the pages under a folder, or the pages of a content type. Each value is one of the values of a page's `listing` ([SPEC §2.1]({repo}/blob/main/SPEC.md#21-files)), from open to closed: `listed`, `unquoted`, `unlisted`.

```toml
[listing]
default = "listed"

[listing.folders]
"internal/" = "unlisted"

[listing.types]
changelog = "unquoted"
```

| Key | Type | Default | Description |
|---|---|---|---|
| `default` | string | `"listed"` | Every page's listing. |
| `folders` | table of strings | `{}` | A listing for every page under a folder, keyed by the folder's path from the content root, ending in `/`, with its name's case as it is on disk. A folder's pages are every page whose path starts with it, in folders under it too. |
| `types` | table of strings | `{}` | A listing for every page of a content type, keyed by the type's name. |

A page's listing is the strictest of its own and the rules it's under: the site's `default`, every folder it's under, and its type's. A folder and a type have no order between them: a page under both gets the stricter. A rule can only restrict. A page MAY set the same value as its rule, or a stricter one, and a page that sets a looser one is an error at the page (`listing-loosened`), since the project contradicts itself. A folder under another folder, and a folder or a type under the `default`, MUST NOT be looser than it (`model-listing-looser`).

A `default` stricter than `listed` is for a site that isn't public yet: no page in it can be listed until the default changes, and a page that says otherwise is the same error.

Each page's record, in `_ascribe/pages.json`, says where its listing came from: the page, the rule's level and its line in `ascribe.toml`, or neither ([output-layout contract](../../contracts/output-layout.md) §1.3).
