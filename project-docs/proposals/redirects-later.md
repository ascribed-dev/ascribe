# Proposal: redirects, the later steps

A proposal, not a plan. It holds what [redirects](redirects.md) leaves for after it ships: each part widens that design, and none changes what it ships. Nothing here starts until the first proposal has shipped, and each part waits for the need that brings it back, named in its section. Both rest on the same research, spike, and host test, which the first proposal records.

The parts are independent of each other, in the order they're likeliest to be needed.

## 1. Versions

**Brought back by:** the first project that retires a release line with `ascribe.addresses` in place. `examples/quill` exercises it in tests until then.

- **A retirement** writes `[lines.*]` in `ascribe.addresses` when `versions retire` runs: each address the line published, its pages, Markdown versions, and explicit ids, to the same page in the current line, the page that replaced it, or its page in the archive, since the line can't be built from the sources once it's retired. The archive is usually on another host, and it's the one cross-host redirect Ascribe writes, since the archive is the line's own home. This supersedes decision #502k's "the host's to write".

  ```toml
  # A retired release line: each address it published, and where it goes now.
  [lines."self-managed 3.4"]
  "/docs/3.4/install/" = "install.md"
  "/docs/3.4/configure-legacy/" = "configure.md"
  "/docs/3.4/upgrade/" = "https://archive.quill.dev/3.4/upgrade.html"
  ```

- **`address-lost`** counts a `[lines.*]` entry as accounting for an address, and a build the base has and the working tree doesn't, which the first proposal reports as one piece of advice, becomes a warning whose fix is `versions retire`'s.
- **An availability mark** that takes a page out of a build while it's live in another, or in an older line, gives its address a page saying in which release it went and linking to where it's still published. An address in `[lines.*]` whose page a mark had taken out of the current line forwards to that page in the archive.
- **A release** whose new line removes a page that another `replaces` forwards the old address to the page that replaced it; one removed with no replacement gets the availability page.

## 2. Folders

**Brought back by:** a project whose folder moves leave `[moved]` long with an entry per page, or `ascribe rename` learning to move a folder.

- **A folder entry** in `[moved]`: a key or a value that's a folder (`"tutorials/" = "learn/"`), so a folder move is one entry.
- **Expanded through later moves:** if `learn/x.md` moved on to `guides/x.md`, `/tutorials/x/` follows it there. A page entry beats a folder entry it falls under.
- **`ascribe rename`** moves a folder, writing one entry; the editor's folder move writes one entry instead of one per page; `ascribe addresses move --from-git` folds a whole folder into one entry.
- **`address-lost`** reports a folder as one source, listing its pages' addresses.

## 3. Anchors and stub pages

**Brought back by:** readers landing on a page at the wrong place after a heading's id changed, or a site deployed under an Astro base that isn't `/` adopting redirects.

- **`anchors`** in `_ascribe/redirects.json`, a key version 1 gains: for each page, its old `#id`s and where each goes, for a script that runs before the page scrolls; an anchor not listed passes through. A fragment never reaches the server, so this is the only way a renamed heading or a split page keeps its links.
- **The script** is a contract change to the elements (`CONTRACT.md` §0 says only `<ascribe-tabs>` runs JavaScript), and how a page gets its anchors, the site markup or `@ascribed/astro`'s head, is decided when it's built.
- **A lost explicit `#id`** becomes a warning, as a page is, once its decision reaches a reader.
- **Stub pages under an Astro base that isn't `/`,** where there's no host root and the first proposal writes nothing: routes the integration adds, one per old address, with the `go/` pages' component (the first proposal's Settled 7): a 0-second refresh with `noindex`, a canonical, and a link, carrying the anchors' map and the address's fragment. Without scripts a stub's refresh drops the fragment, and the docs say so. Whether to write the host's file or stubs is decided by where the site is deployed, not by Ascribe's `base-path`.

## 4. A build's own host, site, and base path

**Brought back by:** a project whose builds deploy to different hosts, which today all claim the same addresses (`ContentModel::for_build` changes only a line's folder).

- **A build sets its own host, site, and base path,** as its own step in the content model.
- **Checks across builds:** a redirect to a page another build publishes is checked against that build's live addresses, where the first proposal checks every address within one build.

## 5. Other hosts

**Brought back by:** a project on that host. Each host is added after a test on it like the first proposal's on Netlify, since that test found behavior the hosts' docs don't mention.

- **The host's file from `redirects.json`,** with `[consumer] host`: Cloudflare's and GitLab's `_redirects`, Apache's `.htaccess`, and nginx's `map` for an operator to include. A host that reads config from the project's root (Vercel's `vercel.json`, Firebase's `firebase.json`) gets a file of rules to merge, and a check that the root file has them. `@ascribed/astro` gives Astro's own `redirects` setting the pairs where Astro's adapter writes them for the host.
- **Rule limits:** the integration counts the merged rules, the site's own included, against the host's limit (50 on S3, 1,000 on GitLab, which drops the rest silently, 2,048 on Vercel): an error above it, a warning at 90%.
- **The `gone` page:** Apache, nginx, and Azure can serve it with a 410 from a rule. Cloudflare's and GitLab's `_redirects` and Vercel can't set a status on a page, so there it's a page with `noindex`, which the delivery spec scores as a soft 404, and the docs say so.
- **`ascribe addresses import`** reads a site's existing redirects file into `ascribe.addresses`, and reports each rule it can't represent, a splat, a rewrite, a condition, leaving it in the site's own file.

## 6. Review's report

**Brought back by:** reviewers asking what a branch does to redirects.

- **Review's report** lists the redirects a branch adds and removes, beside its pages.

## What it changes that's a contract

Each with a decision recorded in the part that changes it:

- **SPEC:** `ascribe.addresses`'s `[lines.*]` and folder entries.
- **The output layout:** `redirects.json`'s `anchors`, and each host's file.
- **The content model:** a build's own host, site, and base path; `[consumer] host`'s other values.
- **The elements:** the anchors' script (`CONTRACT.md` §0).
- **The commands' JSON and the MCP tools:** `versions retire`'s output, `ascribe addresses import`, and review's report (`diff.schema.json`).
- **Decisions:** #502k superseded.
