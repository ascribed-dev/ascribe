---
title: Asset contract
description: Where assets go in each output and how pages refer to them.
---

Every output is self-contained: it works without access to the source files (SPEC §9.4, "Assets"). So local files a page refers to are copied into the output, and references to them are rewritten to point at the copies. This contract says which references those are, how they resolve, where the copies go in each output, what they're called, and how references are rewritten.

| Phase | Uses this contract to |
|---|---|
| 10 | Report references to files that don't exist |
| 11 | Resolve each asset reference from the file it's written in, with provenance |
| 12 | Carry the references that survive a build into each resolved page |
| 18 | Copy assets and rewrite references in the plain-markdown and JSON outputs |
| 20 | Do the same in the site output, with the `astro` profile's placement |
| 25 | Resolve the same references to source files for the preview |

Paths below are `/`-separated and relative, as `ascribe_core::RelPath` represents them. A **content path** is relative to the content root; the **project root** is the directory containing `ascribe.toml`.

## 1. Which references are assets

An **asset reference** is either:

- the source of an image (SPEC §5.3), in any CommonMark image form, inline or reference; or
- the destination of a link (SPEC §5.2) that is local and doesn't name an Ascribe source file under the content root.

A local destination that names a Markdown file (`.md`) under the content root is a page or a fragment, never an asset: a link to a page becomes a route (SPEC §5.2), and a link to a fragment is an error (`link-to-fragment`). Every other local file is an asset, including a `.md` file outside the content root. An image source is an asset whatever its extension.

Not assets:

- **External destinations**: those with a URL scheme (`https:`, `mailto:`, and so on) or starting with `//`. They pass through unchanged (SPEC §5.2).
- **References in raw HTML**, such as `<img src="x.png">` in an HTML block. Ascribe doesn't parse raw HTML, so it passes through unchanged and the file isn't copied (SPEC §9.4).
- **Include paths** (SPEC §4.2). Included content is expanded into the page; the fragment itself isn't copied.

## 2. Resolving a reference

A reference resolves **from the file it's written in** (SPEC §4.2). In content included from a fragment, that's the fragment, not the page that includes it; for a reference-style image or link, it's the file holding the link reference definition, which CommonMark requires to be the same file. `ascribe_core::classify_destination` and `LocalDestination::resolve` implement steps 1–4.

1. **Classify.** A destination with a scheme or starting with `//` is external (§1). Otherwise it's local.
2. **Split.** Everything after the first `#` is the fragment. For an asset, the fragment is kept and written after the rewritten reference (`manual.pdf#page=2`). `?` has no special meaning.
3. **Decode.** Percent-encoded bytes are decoded, as GitHub and browsers do: `my%20diagram.png` names `my diagram.png`. If the decoded bytes aren't valid UTF-8, the text is used as written.
4. **Join.** A path starting with `/` is relative to the content root; any other path is relative to the directory of the file it's written in. `.` and `..` segments are resolved. An empty path (a destination that's only `#fragment`) names the file itself.
5. **Check the boundary.** The resolved file must be inside the content root or inside the project root, and must not be inside the output directory (`[project] output-dir`). Otherwise it's reported as not existing, with the `outside` message of `image-source-missing` or `link-target-missing` (SPEC §9.4).
6. **Check the file.** It must exist and be a regular file (a symbolic link to one counts). Names are compared **exactly, on every platform**: on a case-insensitive file system, `Logo.png` doesn't find `logo.png`, and the `case` message suggests the real name. This keeps a project that checks cleanly on macOS or Windows from breaking on Linux (SPEC §9.4). A reference to a directory doesn't exist.

Problems are reported at file level, at the reference's span: `image-source-missing` for images and `link-target-missing` for links (SPEC §8.2).

The result is the asset's **source path**: its content path, which starts with `..` segments when the file is outside the content root.

## 3. Where copies go

Each emitter writes to its own root, `<output-dir>/<build>/<emitter>/` ([output-layout contract](output-layout.md)), and copies there every asset its pages use. Outputs never share copies.

### 3.1 The mirrored path

Every asset has one **mirrored path**, computed from its source path alone:

- inside the content root: the source path itself (`guides/img/settings.png`);
- outside it: `_ascribe/up/`, then the source path with each leading `..` segment replaced by `up` (`../shared/logo.png` becomes `_ascribe/up/shared/logo.png`, and `../../x.png` becomes `_ascribe/up/up/x.png`).

Pages are written at their own source paths too (the output-layout contract), so an asset beside a page in the source is beside it in the output, and relative references between them keep their shape.

**File names are never changed**, so there's no naming scheme and no hashing. Two different files can't collide, because two different files have different source paths, and so different mirrored paths. The `_ascribe/` directory holds only files Ascribe places, so an asset outside the content root can't land on one inside it, unless the content root itself has a `_ascribe/` directory. The one remaining collision, two outputs at the same path or at paths that differ only in case, fails the build (output-layout contract, §4).

### 3.2 Placement in each output

| Output | Images | Link targets |
|---|---|---|
| Plain markdown | Mirrored path, relative reference | Mirrored path, relative reference |
| JSON | Mirrored path, relative reference | Mirrored path, relative reference |
| Site, `astro` profile | Mirrored path, relative reference | `_ascribe/files/` + mirrored path, root-relative URL |
| Plain markdown, `agents = true` {available=next} | `_ascribe/files/` + mirrored path, absolute URL | `_ascribe/files/` + mirrored path, absolute URL |

The consumer profile decides placement in the site output (SPEC §9.5), through `ConsumerProfile::asset_placement` (`Mirror` or `Published`):

- **Images: mirrored, so Astro processes them.** Astro optimizes an image in a content-collection entry when the entry's markdown refers to it by a relative path; it resolves the path from the entry's file. The site output is the collection, so the mirrored copy is found relative to the page, and Astro's image processing applies as it would to the author's own files. The image must stay a markdown image, not raw HTML (the site-render contract keeps it one).
- **Link targets: published, because Astro doesn't copy them.** Astro leaves a markdown link to a local file alone, and a relative `href` resolves against the page's URL, not its file, so it would break. These copies go under `_ascribe/files/`, which the Astro integration serves at `<base-path>_ascribe/files/`, and links use that URL.

With `[consumer] agents = true` (content-model.md §16), the plain output is published beside the site for agents to read, page by page from wherever they fetch it, so every asset goes under `_ascribe/files/` and is referred to by its absolute URL: `site`, then the URL as above.

The Astro integration's tests verify both behaviors against the Astro version it targets.

## 4. Rewriting references

Every asset reference in a page is rewritten to point at the copy, including references in content included from fragments. The rewritten reference is relative to **the page's** output location, not the fragment's, since included content becomes part of the page (SPEC §4.2).

- **Relative references** go from the directory of the page's output file to the copy, and always start with `./` or `../` (`ascribe_core::RelPath::relative_from`), so no consumer takes them for a package name or a URL. From `guides/install.md` to `_fragments/diagram.png`: `../_fragments/diagram.png`.
- **URLs** (site links under `astro`) are `base-path`, then `_ascribe/files/`, then the mirrored path, with each segment percent-encoded where a URL path needs it. With `base-path = "/docs/"`: `/docs/_ascribe/files/downloads/quill.yaml`.
- **Fragments** from step 2 of §2 are appended after `#`.
- **Writing the destination.** In markdown outputs, the emitter writes the reference so that CommonMark parses it back to exactly that text: in angle brackets (`<../My Diagrams/a.png>`) when it contains a space or a parenthesis, with `<`, `>`, and `\` backslash-escaped. `%`, `#`, and `?` in a file name are percent-encoded (`%25`, `%23`, `%3F`), since a consumer would otherwise read them as an escape, a fragment, or a query. The JSON output records the rewritten reference as a plain string, next to the asset's source path.

## 5. Copying

- Each asset is copied once per emitter output, however many pages or references use it, byte for byte. A symbolic link is copied as the file it points to.
- Only assets referenced by content that survives the build are copied (the resolved tree carries the surviving references). An asset used only in an arm or page a build removes isn't in that build's output. The conformance format's `builds.<name>.assets` lists exactly these, by source path.
- Every copy is listed in the output's manifest, with its source path (output-layout contract, §3).

## 6. The guarantee

With the source directory, and everything outside the output, removed or moved, every asset reference in a build's output resolves to a file in that output. In the site output, this holds once the consumer serves `_ascribe/files/` as §3.2 says. The output tests check it by building a page that includes a fragment with an image beside it, deleting the source, and resolving every reference in the output.

## 7. The preview

The editor preview doesn't copy assets. It resolves references with the same rules (§1 and §2), from the file each reference is written in, and rewrites them to webview URLs of the source files. So a fragment's image shows in the preview as it's found for a build.

The preview serves the content root and, for an asset elsewhere in the project, that asset's own directory. Outside the content root, it doesn't serve an asset that sits directly in the project root, or one in `node_modules` or `.git`. Such an asset shows as a broken image in the preview, with a warning that names it and says why, though a build copies it. A reference into the output directory never becomes an asset: §2 step 5 reports it as not existing.

## Example

```text
docs/                          content root
  guides/install.md            a page: ![Settings](img/settings.png)
  guides/img/settings.png
  _fragments/prerequisites.md  included by install.md: ![Pipeline](pipeline.png)
                                 and [sample config](../downloads/quill.yaml)
  _fragments/pipeline.png
  downloads/quill.yaml
```

The site output of build `site`, with `base-path = "/docs/"`, in `.ascribe/build/site/site/`:

| Copy | Written in `guides/install.md` as |
|---|---|
| `guides/img/settings.png` | `![Settings](./img/settings.png)` |
| `_fragments/pipeline.png` | `![Pipeline](../_fragments/pipeline.png)` |
| `_ascribe/files/downloads/quill.yaml` | `[sample config](/docs/_ascribe/files/downloads/quill.yaml)` |

In the plain-markdown output, the link is `../downloads/quill.yaml`, and `quill.yaml` is copied to `downloads/quill.yaml`.
