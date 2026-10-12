---
type: reference
title: "`ascribe.toml` format contract: consumer, builds, and sources"
description: "The normative contract for the content model's consumer, builds, editor, and sources."
---

Part of the [`ascribe.toml` format contract](../content-model.md), whose [contents](../content-model.md#contents) list every section.

## 15. `[consumer]`

@include: ../../_fragments/content-model/consumer.md

| Key | Type | Default | Description |
|---|---|---|---|
| `profile` | string | `"astro"` | The consumer profile. Supported: `"astro"`. |
| `site` | string (URL) | none | The published site's origin, such as `"https://docs.quill.dev"`: an `http` or `https` URL with no path, query, or fragment. The plain-markdown output needs it to write absolute links (SPEC §9.4). Without it, plain-markdown links are root-relative (they start with `base-path`) and `ascribe build` warns. |
| `base-path` | string | `"/"` | **Routing.** The URL path every route starts with, such as `"/docs/"`, including any locale prefix (`"/en/"`). It MUST start with `/`. A trailing `/` is optional and doesn't change the meaning. |
| `trailing-slash` | string | `"always"` | **Routing.** Whether page URLs end in `/`: `"always"` (`/guides/setup/`) or `"never"` (`/guides/setup`). Match the consumer's own setting (Astro's `trailingSlash` and `build.format`). |
| `slugger` | string | `"github"` | **Slugging.** The algorithm for heading ids (SPEC §5.5), which must be the one the consumer uses. Supported: `"github"`, a port of `github-slugger`, which Astro uses. |
| `html` | boolean | `true` | **HTML passthrough.** Whether the consumer renders raw HTML in markdown. The site output's custom elements need it, so the `astro` profile supports only `true`. |
| `agents` {available="ascribe 0.3.0"} | boolean | `false` | **Outputs for agents.** Whether each build also publishes what AI agents read, following the Web Documentation Delivery Spec 0.6.0: `llms.txt` and each page's Markdown version, in the plain-markdown output laid out by URL, and a pointer to them on each page of the site output. It requires `site`. See below. |
| `permalink-path` {available="ascribe 0.3.0"} | string | `go/` under `base-path` | **Permalinks.** The path each permalink's address is published under (SPEC §4.1), from the site's root: an address is the path, the permalink's id, and a final `/` when `trailing-slash` is `"always"`. It MUST start with `/` (`model-consumer-permalink-path`); a missing final `/` is added. The site output's `_ascribe/permalinks.json` gives it as `path` (contracts/output-layout.md §1.2). |
| `headers` {available="ascribe 0.3.0"} | string | none | **Response headers.** The host whose format the site's response headers are written in. Supported: `"netlify"`, a `_headers` file at the root of what's deployed, which the consumer's integration writes from each build's `_ascribe/headers.json` (contracts/output-layout.md §1.4). The compiler writes the list whatever this says; without it, nothing is written for a host. Any other value is `model-invalid-value`. |

**Heading ids, image attributes, and assets** (SPEC §9.5) are part of the profile, not keys. The `astro` profile has exactly one way to do each, defined by the contracts:

- **Heading ids and image attributes:** the site output writes each as a `<ascribe-attributes>` marker that the consumer's markdown plugin applies, so the consumer keeps its own heading, table-of-contents, and image processing. See [`contracts/site-render.md`](../site-render.md).
- **Assets:** copies mirror their source paths inside each output, and images are referenced relatively so Astro's image processing still applies. Other files a page links to are published under `_ascribe/files/`. See [`contracts/assets.md`](../assets.md).

A later profile that offers a choice adds a key for it then.

**How file paths become routes** (the `astro` profile, verified against Astro): a page's route is `base-path`, then its path relative to the content root with the `.md` extension removed and each segment slugged the way Astro's content loader computes entry ids; a final `index` segment is dropped (`guides/index.md` → `/guides/`); then the trailing slash per `trailing-slash`. The root `index.md` (Astro's entry id `index`) is at `base-path`, which under `trailing-slash = "never"` loses its final `/` unless it's `/`. Two pages with one entry id (`My File.md` and `my-file.md`, or `index.md` and `index/index.md`) can't both be published, and `ascribe build --emit site` fails, naming them. The same router answers the reverse question, which page a route-like link names, for the `link-route` warning and its fix. Source files never contain routes (SPEC §5.2).

**The outputs for agents** (`agents = true`; decision 64). The plain-markdown output becomes what a site publishes for agents, laid out so that copying it to `base-path` publishes it (contracts/output-layout.md):

- Each page's file is its route's path with `.md`: Astro's entry id with `.md` added (`Guides/My Setup.md` → `guides/my-setup.md`, `guides/index.md` → `guides.md`, the root `index.md` → `index.md`). It opens with the blockquote `> For the complete documentation index, see [llms.txt](../<url>).` and a blank line, `<url>` being `llms.txt`'s absolute URL.
- Files the pages link to are copied under `_ascribe/files/`, mirroring their source paths, and linked by absolute URL; the manifest gives each its root-relative `url`.
- `llms.txt` is the index of every page the build publishes, in page order: `# <name>`, a blank line, `> <summary>`, then a section per folder, each `## <heading>` followed by a line per page, `- [<title>](../<url>)` or `- [<title>](../<url>): <description>`, with `<url>` the absolute URL of the page's Markdown. `<name>` and `<summary>` are the title and description of the page whose entry id is `index`; without one, the site's host and a fixed sentence. A page's description is the field its type marks `role = "description"` (§5.4), as plain text on one line (its Markdown's text: a link is its text). Pages directly in the content root are the section "Pages", first, with the `index` page first among them; the rest are grouped by their entry id's first segment, in order of first appearance, each headed by the title of the folder's index page (entry id equal to the segment), which is listed first, or else the folder's name with `-` and `_` as spaces and its first letter capitalized.
- When `llms.txt` would be longer than 50,000 characters, it's split: the root file keeps the header and "Pages", and adds `## Sections` with a line per folder linking to `<segment>/llms.txt`, described by the folder's index page's description or "N pages". Each `<segment>/llms.txt` is a whole index of its own: the section's heading as its name, the folder's index page's description (or a sentence naming the site and the section) as its summary, and the section. A file still over the limit is written, and `ascribe check` reports it (`llms-section-large`).

Without `site` the model is invalid (`model-consumer-agents-site`), since every URL above is absolute.

@include: ../../_fragments/content-model/consumer-astro-settings.md

---

## 16. `[builds.<name>]`

@include: ../../_fragments/content-model/builds.md

---

## 17. `[editor]`

@include: ../../_fragments/content-model/editor.md

---

## 18. `[sources.<name>]`
@available: ascribe 0.2.0

A source is a folder outside the project's content, or another repository, that its pages may take code examples from (SPEC §4.8, §7.3, §7.4). It's the only way a page can read a file outside the project's folder.

```toml
[sources.code]
path = ".."                               # relative to the folder ascribe.toml is in
include = ["crates/**", "examples/**"]    # what's readable; everything else isn't
ignore = ["**/target/**"]

[sources.api]
git = "https://github.com/acme/api.git"
branch = "main"
include = ["src/**"]
```

`<name>` is the source's name (§1.2), which a snippet's address starts with: `code:examples/quill/ascribe.toml#dimensions`.

| Key | Type | Default | Description |
|---|---|---|---|
| `path` | string (path) | **required** without `git` | The source's folder, relative to the project root. It MUST exist and be a directory, and, when the project is in a git repository, it MUST be inside that repository. |
| `git` | string (URL) | **required** without `path` | Another repository: `https://`, `http://`, `ssh://`, `git://`, `file://`, or `user@host:path`. Its files are read from copies in `sources/<name>/`, beside `ascribe.toml`. |
| `branch` | string | the repository's default branch | The branch `ascribe sources update` moves the pin to. Only with `git`. |
| `include` | array of patterns | every file | The files a snippet may read, matched against their paths relative to the source's folder (§1.3). |
| `ignore` | array of patterns | none | Files left out even when `include` matches them. |

A source has `path` or `git`, never both.

### 18.1 `ascribe.lock` and the copies

A source with `git` is pinned to a commit in `ascribe.lock`, beside `ascribe.toml`, and the files its snippets use are copied into `sources/<name>/` at the paths they have in their repository. Both are written by `ascribe sources fetch` and `ascribe sources update`, and committed. `check` and `build` read only the copies.

```toml
# Written by `ascribe sources fetch` and `ascribe sources update`. Don't edit it by hand.
version = 1

[[source]]
name = "api"
git = "https://github.com/acme/api.git"
commit = "9f2c41d0e0c4a1b2c3d4e5f60718293a4b5c6d7e"
missing = ["examples/quickstart.ts"]

[source.files]
"src/auth.rs" = "sha256:5d41402abc4b2a76b9719d911017c592ae2fd2b1f6b3b0f0d5ce64b4c7d0e1a2"
```

| Key | Type | Description |
|---|---|---|
| `version` | integer | The lock's format: `1`. |
| `source[].name` | string | The source's name. Each source is pinned once. |
| `source[].git` | string | The source's `git` when it was pinned. |
| `source[].commit` | string | The commit, in full: 40 or 64 hexadecimal digits. |
| `source[].files` | table | Each copy's path, relative to the source's folder, and the SHA-256 of its bytes: `sha256:` and 64 lowercase hexadecimal digits. |
| `source[].missing` | array of strings | Optional. The paths, relative to the source's folder, that snippets named and that weren't in the repository at the commit, so weren't copied. No path is in both `files` and `missing`. |

`check` reads the lock and the copies with the source files, and reports what doesn't match: a lock it can't read (`lock-invalid`); a pin of a source `ascribe.toml` doesn't declare with `git`, or declares with another `git` (`lock-source-unknown`); a copy that's missing or doesn't have its hash (`source-copy-changed`); a file in `sources/<name>/` the lock doesn't list (`source-copy-unlocked`); and a copy no snippet uses (`source-copy-unused`, a warning). A snippet that names a file with no copy is `snippet-file-missing`: its message says to run `ascribe sources fetch`, or, for a file the lock lists in `missing`, that the file isn't in the repository at the pinned commit, which `fetch` can't change. These are source-file diagnostics; see [the diagnostics reference](../../reference/diagnostics.md).

**Rules** (§24.8): `path` is relative (`model-path-absolute`), and names an existing directory (`model-source-path-missing`) inside the project's repository (`model-source-outside-repository`).

@available: ascribe 0.3.0
With no repository around the project, the language server and `ascribe mcp` are bounded by what the user opened: a folder of the editor's workspace, or the folder `ascribe mcp` was started in. A source outside it is `model-source-outside-repository` with the variant `opened`. A command, such as `ascribe build` or `ascribe check`, reads a source wherever `path` leads, so a copy of a repository without `.git` still builds.

A source has exactly one of `path` and `git`, `branch` only with `git`, a URL and a branch name `git` accepts, and copies outside the content root (`model-source-remote`).
