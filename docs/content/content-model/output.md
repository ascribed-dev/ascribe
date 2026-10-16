---
type: reference
title: "`ascribe.toml`: consumer, builds, and sources"
description: "How the content model sets the site's consumer, the builds, the editor's build, and the source repositories."
---

Part of the [`ascribe.toml` reference](index.md), whose [contents](index.md#contents) list every section.

## 15. `[consumer]`
@id {permalink=true}: consumer

@include: ../_fragments/content-model/consumer.md

| Key | Type | Default | Description |
|---|---|---|---|
| `profile` | string | `"astro"` | The consumer profile. Supported: `"astro"`. |
| `site` | string (URL) | none | The published site's origin, such as `"https://docs.quill.dev"`: an `http` or `https` URL with no path, query, or fragment. The plain-markdown output needs it to write absolute links ([SPEC §9.4]({repo}/blob/main/SPEC.md#94-outputs)). Without it, plain-markdown links are root-relative (they start with `base-path`) and `ascribe build` warns. |
| `base-path` | string | `"/"` | **Routing.** The URL path every route starts with, such as `"/docs/"`, including any locale prefix (`"/en/"`). It must start with `/`. A trailing `/` is optional and doesn't change the meaning. |
| `trailing-slash` | string | `"always"` | **Routing.** Whether page URLs end in `/`: `"always"` (`/guides/setup/`) or `"never"` (`/guides/setup`). Match the consumer's own setting (Astro's `trailingSlash` and `build.format`). |
| `slugger` | string | `"github"` | **Slugging.** The algorithm for heading ids ([SPEC §5.5]({repo}/blob/main/SPEC.md#55-heading-ids)), which must be the one the consumer uses. Supported: `"github"`, a port of `github-slugger`, which Astro uses. |
| `html` | boolean | `true` | **HTML passthrough.** Whether the consumer renders raw HTML in markdown. The site output's custom elements need it, so the `astro` profile supports only `true`. |
| `agents` {available="ascribe 0.3.0"} | boolean | `false` | **Outputs for agents.** Whether each build also publishes what AI agents read, as the [Web Documentation Delivery Spec](https://agentdocsspec.com/spec/web/) lays it out: `llms.txt`, the index of every page, and each page's Markdown at its URL with `.md`. It needs `site`, since every link agents read is absolute. See below. |
| `permalink-path` {available="ascribe 0.3.0"} | string | `go/` under `base-path` | **Permalinks.** Where each [permalink](../writing/directives.md#permalinks)'s address is published, as a path from the site's root, such as `"/go/"`: `/go/cli-check/` forwards to the heading marked `cli-check`. It must start with `/`. Set it when the addresses a product prints shouldn't move when `base-path` does. Under the `astro` profile it must be under `base-path`, since an Astro site serves nothing outside it. |
| `headers` {available="ascribe 0.3.0"} | string | none | **Response headers.** The host whose format a build's response headers are written in, so that the Markdown version of a page that isn't listed isn't indexed either. Supported: `"netlify"`, the `_headers` file Netlify reads. `@ascribed/astro` writes it into the built site ([the `@ascribed/astro` reference](../astro/reference.md#response-headers)). Without it, nothing is written for a host, and the headers are in `_ascribe/headers.json` for a deploy step to read. See below. |

**Heading ids, image attributes, and assets** ([SPEC §9.5]({repo}/blob/main/SPEC.md#95-consumer-profile)) are part of the profile, not keys. The `astro` profile has exactly one way to do each:

- **Heading ids, image attributes, and glossary links:** the site output writes each as a `<ascribe-attributes>` marker that the consumer's markdown plugin applies, so the consumer keeps its own heading, table-of-contents, and image processing, and a glossary link carries `data-ascribe-term`.
- **Assets:** copies mirror their source paths inside each output, and images are referenced relatively so Astro's image processing still applies. Other files a page links to are published under `_ascribe/files/`.

A later profile that offers a choice will add a key for it.

**How file paths become routes** (the `astro` profile): a page's route is `base-path`, then its path relative to the content root with the `.md` extension removed and each segment slugged the way Astro's content loader computes entry ids; a final `index` segment is dropped (`guides/index.md` → `/guides/`); then the trailing slash per `trailing-slash`. The root `index.md` (Astro's entry id `index`) is at `base-path`, which under `trailing-slash = "never"` loses its final `/` unless it's `/`. Two pages with one entry id (`My File.md` and `my-file.md`, or `index.md` and `index/index.md`) can't both be published, and `ascribe build --emit site` fails, naming them. The same router answers the reverse question, which page a route-like link names, for the `link-route` warning and its fix. Source files never contain routes ([SPEC §5.2]({repo}/blob/main/SPEC.md#52-links)).

**The outputs for agents** (`agents = true`) change what the plain-markdown output is, and add to the site output:

- **The plain-markdown output is laid out by URL,** so it can be copied as it is to `base-path` beside the site: each page at its route with `.md` (`Guides/My Setup.md` → `guides/my-setup.md`, the root `index.md` → `index.md`), each opening with a line pointing at `llms.txt`; the files they link to under `_ascribe/files/`, linked by absolute URL; and `llms.txt`.
- **`llms.txt`** lists every page the build publishes as `- [Title](../url): description`, linking to its Markdown. Its heading is the title of the page at `base-path`, and its summary that page's description. Pages in a folder are a section named after the folder (the title of its `index.md`, when it has one); pages outside every folder come first, under "Pages". An index over 50,000 characters is split into one `llms.txt` per folder, which the root file links to. A page's description is the field its type marks [`role = "description"`](index.md#54-field-roles), as plain text; a type without one lists its pages without descriptions.
- **Each page of the site output** opens with a pointer to `llms.txt` and the page's Markdown, which the element library hides from sight ([element contract §8]({repo}/blob/main/packages/elements/CONTRACT.md#8-ascribe-for-agents)).

**The headers of a Markdown version.** A Markdown version has no head, so what its page's head says is a response header, which the host sends. Each build lists them in `_ascribe/headers.json` in the site output ([output-layout contract](../contracts/output-layout.md#14-the-headers)), a rule for each Markdown version's path: `X-Robots-Tag: noindex` for a page that's `unlisted`, `nosnippet` for one that's `unquoted`, and `Link: <page>; rel="canonical"` on every one, so a search engine lists the page and not its Markdown copy, while an agent can still find either. With `headers = "netlify"`, `@ascribed/astro` writes them into `_headers`.

`@ascribed/astro` publishes the plain output with the site, and serves it in `astro dev`. With another generator, copy `<output-dir>/<build>/plain/` to the site's `base-path`. `ascribe check` then reports two advice checks: a description too long for its line in `llms.txt` (`description-too-long`), and an `llms.txt` still over the limit when it's split (`llms-section-large`).

@include: ../_fragments/content-model/consumer-astro-settings.md

---

## 16. `[builds.<name>]`

@include: ../_fragments/content-model/builds.md

@available: ascribe 0.3.2
A versioned build's outputs, their names and addresses, and what each line keeps and shows are in [Versions](../versions/index.md#build-a-site-for-each-line). An older line's output says nothing to agents: it has no `llms.txt`, and its pages point to no Markdown version.

---

## 17. `[editor]`

@include: ../_fragments/content-model/editor.md

---

## 18. `[sources.<name>]`
@available: ascribe 0.2.0

A source is a folder outside the project's content, or another repository, that its pages may take code examples from, with [`@snippet`](../writing/directives.md#snippet) ([SPEC §7.3]({repo}/blob/main/SPEC.md#73-sources)). It's the only way a page can read a file outside the project's folder.

```toml
[sources.code]
path = ".."                               # relative to the folder ascribe.toml is in
include = ["crates/**", "examples/**"]    # what's readable; everything else isn't
ignore = ["**/target/**"]

[sources.api]
git = "https://github.com/acme/api.git"   # another repository
branch = "main"                           # what `ascribe sources update` follows
include = ["src/**", "examples/**"]
```

A page names a file through its source as `<source>:<path>`, with the path relative to the source's folder: `@snippet: code:examples/quill/ascribe.toml#dimensions`. Both kinds of source are addressed the same way, so moving code to another repository, or back, changes `ascribe.toml` and no page.

| Key | Type | Default | Description |
|---|---|---|---|
| `path` | string (path) | **required** without `git` | The source's folder, relative to the project root. It must exist, be a directory, and be inside the git repository the project is in, when the project is in one. |
| `git` | string (URL) | **required** without `path` | Another repository, as `git` takes it: `https://`, `http://`, `ssh://`, `git://`, `file://`, or `user@host:path`. |
| `branch` | string | the repository's default branch | The branch `ascribe sources update` moves the pin to. |
| `include` | array of patterns | every file | The files a snippet may read, matched against their paths relative to the source's folder (§1.3). |
| `ignore` | array of patterns | none | Files left out even when `include` matches them. |

`<name>` follows the rules for keys (§1.2). A project can declare several sources, of either kind.

**Rules.** `path` is relative (`model-path-absolute`); its folder must exist and be a directory (`model-source-path-missing`) inside the project's repository (`model-source-outside-repository`). With no repository, as in a copy made with `git archive`, `ascribe build` and the other commands read a source wherever `path` leads, but the editor reads one only inside a folder of the workspace it opened, and `ascribe mcp` only inside the folder it was started in: open the folder that contains the source. A source has `path` or `git`, never both, and `branch` only with `git` (`model-source-remote`).

### A source in another repository

`check` and `build` never fetch anything. A source with `git` is read from copies kept in the project:

- `ascribe.lock`, beside `ascribe.toml`, pins each such source to a commit, and records the hash of each file copied from it, and which files snippets name that aren't in the repository at that commit.
- The files the project's snippets use are copied, whole, into `sources/<name>/` beside `ascribe.toml`, at the paths they have in their repository. Only those files are copied, not the repository, and not everything `include` matches. `sources/` can't be inside the content root.

Both are written by [`ascribe sources fetch` and `ascribe sources update`](../examples/sources.md#cli-sources), and committed with the rest of the docs. `check` reports a copy that was changed by hand, a copy the lock doesn't list, a copy no snippet uses any more, and a snippet whose file hasn't been copied yet, or isn't in the repository at the pinned commit because it was moved or deleted there. The copies are published to everyone who can read the docs repository: for a private code repository and public docs, that's a decision to make knowingly.
