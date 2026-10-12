---
title: Output-layout contract
description: Where ascribe build writes each [build](../reference/content-model.md#16-buildsname)'s output, how it records what it wrote, and how it replaces a previous output.
permalink: output-layout-contract
---

This contract says where `ascribe build` writes each build's output, how it records what it wrote, and how it replaces a previous build's output without ever deleting a file it didn't write. It's implemented once for every emitter, and the site emitter uses the same code; the Astro integration reads the output it describes.

## 1. Directory layout

```text
<output-dir>/                   [project] output-dir, default .ascribe/build
  .lock                         held while a build writes (§4)
  .staging/                     work in progress (§4)
  <build>/
    <emitter>/                  one emitter's output: the emitter root
    <emitter>.manifest.json     what that emitter root holds (§3)
```

- `<build>` is the build's name from `ascribe.toml`, as written (`site`, `cloud`, `self-managed-3.3`). Build names are unique ignoring case (content-model.md, `model-build-name-case`), and never start with `.`, so they can't clash with `.lock` or `.staging`.
- `<emitter>` is `site`, `plain`, or `json`.
- Each emitter root is complete on its own: its pages, and a copy of every asset they use ([asset contract](assets.md)). Nothing is shared between emitter roots or builds.

### 1.1 Inside an emitter root

Pages mirror their source paths relative to the content root:

| Emitter | Page `guides/install.md` is written to |
|---|---|
| `site` | `guides/install.md` |
| `plain` | `guides/install.md` |
| `json` | `guides/install.json` |

Only pages the build publishes are written (SPEC §9.3); fragments never are.

**With `[consumer] agents = true`** {available=next} (content-model.md §15), the `plain` root is laid out by URL instead, so that copying it to the site's base path publishes what agents read:

| Emitter | Page `Guides/My Setup.md` is written to |
|---|---|
| `plain` | `guides/my-setup.md`: the page's Astro entry id with `.md` (the root `index.md` stays `index.md`) |

Beside the pages it holds `llms.txt`, and one `<folder>/llms.txt` per section when the index is split (content-model.md §15); both are `generated` in the manifest. Every asset the pages use is under `_ascribe/files/`, mirroring its source path, with its `url` in the manifest. The `site` and `json` roots don't change.

Assets go where the [asset contract](assets.md) §3 puts them: at their mirrored path, or, for links in the site output, under `_ascribe/files/`.

**`_ascribe/` is reserved** in every emitter root for files Ascribe places other than pages and assets inside the content root: assets from outside the content root (`_ascribe/up/`), the site output's published files (`_ascribe/files/`), and generated files an emitter adds, such as the site emitter's Zod schema (`_ascribe/schema.ts`), its list of permalinks (`_ascribe/permalinks.json`, §1.2), its page records (`_ascribe/pages.json`, §1.3), and the headers its files need (`_ascribe/headers.json`, §1.4).

### 1.2 The permalinks list
@available: next

The site output holds `_ascribe/permalinks.json`: every permalink the build publishes (SPEC §2.1, §4.1), for the site to give each one an address that forwards to its page or heading. It's `generated` in the manifest, and written for every build, with an empty table when the build has none, so a consumer can tell "no permalinks" from an Ascribe that doesn't write the list.

```json
{
  "version": 1,
  "path": "/go/",
  "dimensions": { "pm": ["npm", "pnpm", "yarn"] },
  "permalinks": {
    "cli-check": { "route": "/reference/cli/", "fragment": "cli-check", "source": "reference/cli.md" },
    "cli-reference": { "route": "/reference/cli/", "source": "reference/cli.md" },
    "check": { "route": "/reference/cli/", "fragment": "cli-check", "source": "reference/cli.md", "alias": "cli-check" },
    "old-check": { "route": "/reference/cli/", "fragment": "cli-check", "source": "reference/cli.md", "retired": "Merged into the CLI reference" }
  },
  "removed": {
    "legacy-sync": { "reason": "The sync agent was removed in 4.0" }
  }
}
```

| Field | Meaning |
|---|---|
| `version` | The list's format version: `1`. A reader rejects versions it doesn't know. |
| `path` | Where the addresses are published: `[consumer] permalink-path`, or `go/` under `base-path`, as a path from the site's root ending in `/` (content-model contract §15). A permalink's address is `path`, its name, and a final `/` when `trailing-slash` is `"always"`. |
| `dimensions` | The content model's dimensions, keyed by name and sorted by name, each with its values in declaration order: the parameters an address takes, each setting the reader's choice for a dimension (`?pm=pnpm`). |
| `permalinks` | One entry per permalink, keyed by its name, sorted by name. A name is unique in a build (`permalink-duplicate`). |
| `permalinks.*.route` | The route of the page the permalink is on, as the site output's links write it. |
| `permalinks.*.fragment` | The heading's anchor on that page, without `#`. Absent for a page's own permalink, which leads to the top of the page. |
| `permalinks.*.source` | The page's source path, relative to the content root. |
| `permalinks.*.alias` | For an alias in `ascribe.toml`'s `[permalinks.aliases]` (SPEC §7.6), the name it leads to, as written there. It's listed with the page and anchor of the permalink its aliases end at, when the build publishes that permalink, and not listed otherwise. |
| `permalinks.*.retired` | For a retired permalink in `[permalinks.retired]` whose `to` leads to a page the build publishes, why it was retired; `route`, `fragment`, and `source` are the `to`'s. |
| `removed` | The retired permalinks with nowhere to lead in this build, keyed by name and sorted: those without `to`, and those whose `to`'s page the build doesn't publish. Each has `reason`, why it was retired. Their addresses show that what was there was removed, and why. Absent when there are none. |

An address forwards to `route`, then `#` and `fragment` when there is one. A later version may add keys to the file or to an entry; it doesn't change these.

### 1.3 The page records
@available: next

The site output holds `_ascribe/pages.json`: a record of each page the build publishes (SPEC §9.4), saying what every reader that isn't a person is told about it, and what the project says to crawlers, for a site to write each page's head, its sitemap, and its `robots.txt` from. `llms.txt` is written from the same records. It's `generated` in the manifest, and written for every build, with an empty table when the build has no pages.

```json
{
  "version": 1,
  "site": "https://docs.example.com",
  "pages": {
    "/guides/install/": {
      "source": "guides/install.md",
      "title": "Install the agent",
      "description": "Install and connect the sync agent.",
      "canonical": "https://docs.example.com/guides/install/",
      "listing": "listed",
      "listing_from": { "level": "default" },
      "markdown": "/guides/install.md"
    }
  },
  "robots": {
    "lines": ["Disallow: /drafts/"],
    "purposes": { "training": "disallow", "search": "allow" },
    "groups": [
      {
        "purpose": "training",
        "user_agents": ["GPTBot", "ClaudeBot"],
        "lines": ["Disallow: /"]
      }
    ]
  }
}
```

| Field | Meaning |
|---|---|
| `version` | The file's format version: `1`. A reader rejects versions it doesn't know. |
| `site` | The project's address, `[consumer] site`, without a final `/`. Absent when it isn't set. |
| `pages` | One entry per page the build publishes, keyed by its route, sorted by route. An entry doesn't repeat its route. |
| `pages.*` key | The page's route, from the host's root, as the site output's links write it. |
| `pages.*.source` | The page's source path, relative to the content root, with `/`, as the page's frontmatter `source` has it. |
| `pages.*.title` | The page's title, as plain text: the site output's `title`, never its formatted form. Absent when the page has none. |
| `pages.*.description` | The page's description, the field its type marks `role = "description"`, as plain text on one line, as `llms.txt` shows it. Absent when it has none. |
| `pages.*.canonical` | The page's own address, absolute: `site`, then the route. Absent when `site` is. |
| `pages.*.listing` | The page's `listing` (SPEC §2.1): `listed`, `unquoted`, or `unlisted`, the strictest of its own and the rules of `[listing]` it's under. Every page has one; a page that nothing sets is `listed`. |
| `pages.*.listing_from` | Where `listing` comes from. `level` is `default` when nothing sets it, `page` for the page's own `listing`, and for a rule of `[listing]` (content-model contract §22), its level: `site` for `default`, `folder`, or `type`. A rule's has `folder` or `type`, the key it's under; `rule`, the rule as written in `ascribe.toml`, `key = value`; and `line`, its line there, from 1. Of the rules a page is under, the strictest is named, and of rules as strict, the narrowest: the deepest folder, then the type, then `default`. A page as strict as its rule, or stricter, has `page`. |
| `pages.*.markdown` | The address of the page's Markdown version, from the host's root, when the build publishes one (`[consumer] agents = true`). Absent otherwise. |
| `robots` | What the project says to crawlers. |
| `robots.lines` | The project's `[crawlers] lines` (content-model contract §21), in order, each path in an `Allow` or `Disallow` line with the base path before it, so that each is from the host's root, and every other line as written. Lines before any `User-agent` line are for every crawler. Empty when the project has none. |
| `robots.purposes` | What the project says of each purpose in `[crawlers]` (content-model contract §21), keyed by `training`, `search`, or `on-request`, each `allow` or `disallow`. Only the purposes it sets; empty when it sets none. A site whose projects share one `robots.txt` reads it to find two that disagree. |
| `robots.groups` | A group of `robots.txt` for each purpose the project disallows, in the order `training`, `search`, `on-request`: `purpose`; `user_agents`, every crawler of that purpose Ascribe knows, each to be written as a `User-agent` line; and `lines`, the group's rules, `Disallow: /`. Empty when the project disallows none. A site writes these groups before the `*` group, and each purpose once. |

A page that's `unlisted` isn't in `llms.txt`. A later version may add keys to the file, to an entry, or to `robots`; it doesn't change these.

### 1.4 The headers
@available: next

The site output holds `_ascribe/headers.json`: the response headers the build's files need from the host, as rules, each a path and the headers it gets. A Markdown version (content-model contract §15) has no head to carry what its page's head says, so the host sends it, and the host is the site's: `@ascribed/astro` writes the rules into Netlify's `_headers` with `[consumer] headers = "netlify"`, and a site built another way reads the list. It's `generated` in the manifest, and written for every build, with an empty list when the build publishes no Markdown versions, so a consumer can tell "no rules" from an Ascribe that doesn't write the file.

```json
{
  "version": 1,
  "headers": [
    {
      "path": "/docs/guides/install.md",
      "headers": { "Link": "<https://docs.example.com/docs/guides/install/>; rel=\"canonical\"" }
    },
    {
      "path": "/docs/internal/runbook.md",
      "headers": {
        "Link": "<https://docs.example.com/docs/internal/runbook/>; rel=\"canonical\"",
        "X-Robots-Tag": "noindex"
      }
    }
  ]
}
```

| Field | Meaning |
|---|---|
| `version` | The file's format version: `1`. A reader rejects versions it doesn't know. |
| `headers` | One rule for each page's Markdown version, sorted by `path`. |
| `headers[].path` | The path the rule is for, from the host's root, matched exactly and never as a pattern: the page record's `markdown` (§1.3). |
| `headers[].headers` | Each header's value, keyed by its name, sorted by name. `Link` is `<canonical>; rel="canonical"`, the page's own address from its record, so a search engine lists the page and not its Markdown version. `X-Robots-Tag` is the rule the page's head gives a crawler: `noindex` for a page that's `unlisted`, `nosnippet` for one that's `unquoted`, and absent for one that's `listed`. |

Whatever answers a request for a page's Markdown sends the same headers, whether it serves the file or answers the page's own address asked for with `Accept: text/markdown`. A later version may add keys to the file, such as a list of redirects beside `headers`, or more headers to a rule; it doesn't change these.

## 2. What Ascribe owns

Ascribe owns exactly these, and touches nothing else under `<output-dir>`:

- the files listed in a manifest (§3);
- the manifests themselves;
- `<output-dir>/.staging/` and everything in it;
- `<output-dir>/.lock`.

Everything else under `<output-dir>` belongs to the user, even inside an emitter root. A user's file is never deleted, moved, or overwritten, and a build that would need to overwrite one fails instead (§4). A directory is removed only when a build empties it by removing files it owned.

**Links.** {available=next} A build writes and removes only through real folders. It fails, naming the link and touching nothing, when a symbolic link (or, on Windows, a junction) is any of these:

- a folder on the way from the project's folder to `<output-dir>`, or `<output-dir>` itself;
- `<output-dir>/.lock`, `.staging/`, or a folder in it on the way to a build's staging;
- `<build>/`, an emitter root, or a manifest;
- a folder between an emitter root and a file the build writes or removes, or that file.

A link is never followed, replaced, or removed. The folders above the project's folder aren't looked at.

**`output-dir` outside the project.** `output-dir` may lead outside the project's folder with `..` (content-model contract). Ascribe then writes there as it would inside, under the same rules, and the folders on the way out of the project aren't checked for links: whoever set `output-dir` chose that folder.

## 3. The manifest

Each emitter root has a manifest beside it, `<output-dir>/<build>/<emitter>.manifest.json`, listing every file in the root that Ascribe wrote. It's JSON:

```json
{
  "format": "ascribe-manifest",
  "version": 1,
  "build": "site",
  "emitter": "site",
  "files": [
    { "path": "_ascribe/files/downloads/quill.yaml", "kind": "asset", "source": "downloads/quill.yaml",
      "url": "/docs/_ascribe/files/downloads/quill.yaml" },
    { "path": "_ascribe/up/shared/logo.png", "kind": "asset", "source": "../shared/logo.png" },
    { "path": "guides/img/settings.png", "kind": "asset", "source": "guides/img/settings.png" },
    { "path": "guides/install.md", "kind": "page", "source": "guides/install.md" }
  ]
}
```

| Field | Meaning |
|---|---|
| `format` | Always `"ascribe-manifest"`. A file at a manifest's path without it isn't a manifest, and isn't Ascribe's (§4). |
| `version` | The manifest format's version: `1`. A reader rejects versions it doesn't know. |
| `build`, `emitter` | Which output this is. |
| `anchors` {available="ascribe 0.2.0"} | `true` when the site output has source anchors (`ascribe build --anchors`; site-render contract §7). Absent otherwise. |
| `files` | Every file in the emitter root that Ascribe wrote, sorted by `path`. |
| `files[].path` | The file's path relative to the emitter root, `/`-separated. A manifest is read as input, not trusted as Ascribe's own: a path that's absolute, has a `.` or `..` segment, or that the operating system reads as on a drive (`C:/x`, `C:x` on Windows) isn't a manifest's, and fails the build. |
| `files[].kind` | `"page"`, `"asset"`, or `"generated"` (anything else Ascribe writes, such as a schema). |
| `files[].source` | For a page or asset, its source path relative to the content root, starting with `..` for a file outside it. Absent for generated files. |
| `files[].url` | For an asset under `_ascribe/files/` (the site output's, or the plain output's with `agents = true`), the root-relative URL pages use for it, which the consumer must serve it at (asset contract §3.2). Absent otherwise. |

The manifest lists files only. Directories are implied by their paths.

## 4. Replacing a previous build's output

A build writes to a staging directory and replaces the previous output only when it has succeeded. Each build-and-emitter pair is replaced separately: building one build, or one emitter, doesn't touch the others.

1. **Lock.** Take an exclusive advisory lock on `<output-dir>/.lock` (creating it if needed), held until the build ends and released by the operating system if the process dies. If another build holds it, fail: "another ascribe build is writing to `<output-dir>`".
2. **Read the previous manifest**, if there is one. A file at the manifest's path that isn't a manifest (§3) fails the build: Ascribe can't tell what it owns. So does a link on the way to where the build writes or removes (§2, **Links**), before anything is touched.
3. **Emit into staging:** `<output-dir>/.staging/<build>/<emitter>/`, removing any leftover staging for that pair first. If emitting fails, or the build has errors (SPEC §8.2: "a build MUST fail on errors"), remove the staging directory and stop. The previous output is untouched.
4. **Check before touching anything.** Fail, naming every problem, and leave the output untouched, if:
   - a file would be written where a file exists that the previous manifest doesn't list (a user's file is in the way);
   - a file would be written where a directory exists (even one holding only Ascribe's files: a directory is removed only when a build empties it), or a directory is needed where a file exists that the previous manifest doesn't list. A file the previous manifest lists, in the way of a directory, is Ascribe's: it's removed just before the directory is made;
   - two new files have the same path, or paths that differ only in case (they'd be one file on macOS and Windows).
5. **Record ownership first.** Write the manifest listing every file in the previous manifest *and* every new file. Write each manifest by writing a temporary file in `.staging/` and renaming it into place, so it's never half-written.
6. **Move the new files into place**, replacing the previous build's files at the same paths. A file whose content hasn't changed may be left as it is, so watchers such as Astro's dev server see only real changes.
7. **Remove stale files:** those in the previous manifest that this build didn't produce. Then remove any directory that this step emptied.
8. **Write the final manifest**, listing only this build's files, the same way as in step 5.
9. **Clean up:** remove the staging directory and release the lock.

**The invariant:** at every moment, every file Ascribe has written into an emitter root is listed in the manifest on disk. So if a build is interrupted, the root holds either the previous output or a mix of old and new files that are all listed, and the next build removes what it no longer produces. A user's file is never at risk, because Ascribe only deletes files a manifest lists.

A file listed in a manifest that the user has since edited is still Ascribe's: the next build overwrites or removes it. The output directory is generated; edit the source instead.

## 5. Why this shape

- **One root per build and emitter**, with nothing shared, keeps each output self-contained (SPEC §9.4) and lets a consumer point at exactly one directory, such as an Astro content collection at `.ascribe/build/site/site/`.
- **Mirrored paths** make the output's structure the source's structure, keep relative references between pages and assets valid, and make collisions between different files impossible (asset contract §3.1).
- **The manifest beside the root, not in it**, keeps the root holding only output, so a consumer loading every file in it loads nothing else.
- **Failing on a user's file** instead of overwriting it follows the rule that output never destroys what Ascribe didn't create.
