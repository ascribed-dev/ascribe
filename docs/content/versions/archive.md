---
type: how-to
title: Archive and retire a line
description: Keep a copy of a release line that needs nothing else, host it, then take the line out of the source and send its addresses on.
---

Before a line leaves the window of live versions, keep a copy of it that needs nothing else, then take it out of the source so the pages stop carrying it. The order matters: once the line is retired, the source no longer holds what its archive needs.

## Before you start

- The line is the oldest live line of a [versioned build](index.md), and every line's content is still in the source.
- Commit your work first: `ascribe versions retire` writes only files git can put back as they were.

## Archive and retire a line

### 1. Archive the line
@id: archive-a-line

```sh
ascribe versions archive "self-managed 3.4" --to archive/3.4 --current https://docs.example.com/docs/
```

The command checks the line's build first, and writes nothing on an error. It names each link it wrote as its text, since it led out of the line, and says what to add to `ascribe.toml` once the archive is hosted. In `examples/quill`:

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/versions-archive.txt

### 2. Look over the archive

Open `archive/3.4/index.html` from the disk. Every page of the line is there as HTML, `install.md` as `install.html`, with every link between them relative, so it works opened from a disk or served at any address:

- Every page says at its top that it's the documentation for 3.4 and that it's no longer maintained, and with `--current`, links to the current documentation.
- A sidebar lists the pages in the line's [navigation](../navigation/index.md) order, with a search box that works from a disk too, and previous and next links.
- `all.html` has every page in one document, to print the whole line or save it as one PDF.
- Every page is marked `noindex`, and the folder has a `robots.txt`.

The archive is Ascribe's own render, not your site's design. The [archive contract](../contracts/archive.md) has the folder's layout.

### 3. Host it, and record its address

Put the folder on any host, then add its address under `archived`, so the live site lists it among the lines:

```toml
[releases.self-managed.archived]
"3.4" = { url = "https://archive.example.com/3.4/" }
```

If you retire the line next, skip the entry: the retirement writes it.

### 4. Retire the line
@id: retire-a-line

```sh
ascribe versions retire "self-managed 3.4" --url https://archive.example.com/3.4/
```

It moves 3.4 from `live` to `archived`, with its address and the day it ended, so `ascribe build` stops writing its output. Then each mark for the target is rewritten to the simplest spec that means the same in every line still live or planned, and so is a feature's spec in `ascribe.toml`:

| A mark | With 3.5 and 3.6 live |
|---|---|
| `self-managed 3.4` | `self-managed` |
| `self-managed deprecated 3.2` | `self-managed deprecated` |
| `self-managed (preview 3.3, ga 3.5, deprecated 3.6)` | `self-managed (ga 3.5, deprecated 3.6)` |
| `cloud, self-managed 3.4` | `cloud, self-managed` |

No live line's output changes, and the command proves it by building before and after. A build that isn't a line, such as a badge build that shows "3.4+", does change: the command names its pages, and needs `--force` to go ahead. Run it with `--dry-run` first to read what it would do, as in `examples/quill`:

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/versions-retire.txt

### 5. Add the redirects

Each address 3.4 published should go somewhere: to the same page in the current line, to the page that replaced it there, or to its page in the archive. The command lists them, the replaced pages in its text and every address in its JSON, each as `from` and `to`. Add them to your host's redirects; Ascribe writes nothing for a host.

### 6. Remove what no line has

The retirement deletes nothing on its own. List the content only 3.4 had, and the marks that say nothing any more, such as `@available: self-managed` on a page of a site for self-managed alone:

```sh
ascribe versions retire "self-managed 3.4" --url https://archive.example.com/3.4/ --show-unused
```

Read the list, then run it again with `--remove-unused` to delete them. A mark that says nothing still draws a badge, which is why it's yours to remove.

## See also

- [`ascribe versions archive`](release.md#cli-versions-archive) and [`ascribe versions retire`](release.md#cli-versions-retire): their options, and when each refuses.
- [The archive contract](../contracts/archive.md): the folder's layout.
- [Publish a site for each version](index.md).
