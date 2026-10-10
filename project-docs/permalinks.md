# Proposal: permalinks

A proposal, not a plan. It says what the feature is and what it would look like, so the idea can be judged before anything is designed in detail.

## The goal

Something outside the docs can link to a page or a section by a name that never changes, and Ascribe keeps that link working while the docs are renamed, moved, and rewritten.

The main customer is software that links to its own documentation: a help link in an interface, an error message that says where to read more, a command's `--help`. Ascribe is one of them. Its binary builds addresses such as `https://ascribed-dev.com/reference/diagnostics/#asc036-link-target-missing`, which holds the site's layout, a page's path, and a heading's anchor. Moving any of the three breaks every copy of the binary already installed.

This is the problem context-sensitive help has solved for thirty years, always the same way: the product holds an id, and the documentation decides where the id leads.

## What Ascribe has, and what's missing

- **A page's identity is its file's path,** and its address is computed from that path.
- **A heading can have an `@id`,** unique within its page, which is also its anchor.
- **Links inside a project name files,** are checked, and are rewritten when a file is renamed.

So everything inside a project survives a move. Nothing outside does, because nothing outside can name a page except by its address.

## Declaring one

A heading's `@id` gains an attribute, and a page gets a frontmatter key with the same name. That's the pattern availability already follows: `@available` on a section, `available` on a page.

```markdown
## Rotate your API key
@id {permalink=true}: rotate-keys
```

```yaml
---
title: Command reference
permalink: cli-reference
---
```

Marking an id as a permalink changes three things about it:

- **It's unique across everything a build publishes,** not only within its page. Pages and headings share one set of names.
- **It's published:** in a list, and at an address of its own that leads to wherever the content is now.
- **Removing it is noticed,** and is something an author does on purpose.

Everything else about `@id` stays as it is, including that it's the heading's anchor. An id without the attribute behaves exactly as today.

**Not a second way to link inside a project.** Pages go on linking to files, which is checked and survives renames already. A permalink is for references from outside.

**Not needed everywhere.** Only what something outside points at needs one.

## What a build writes

**An address for each permalink.** Under a path the project chooses, `/go/` by default:

```
https://docs.example.com/go/rotate-keys   →   /guides/security/#rotate-keys
```

Ascribe hosts nothing, so this can't depend on a server. Each address is a small page that sends the reader on, which works on any static host. Redirect rules written for a particular host can be added later, as a faster path to the same place.

**A list of them,** with each one's current address, in the build's output, for a site or a tool that wants to resolve them itself.

**A page for a name nobody declared.** An old copy of a product may ask for an id this docs set never had. The reader gets a page that says so and offers search and the docs' home, not a bare error.

## What the product reads

Every context-sensitive help system hands developers a file of ids. Here it's a file Ascribe keeps beside `ascribe.toml`, committed with the docs:

```toml
# ascribe.permalinks. Written by Ascribe. Don't edit it.
cli-reference = "reference/cli.md"
rotate-keys = "guides/security.md#rotate-keys"
```

It has two jobs.

- **The product's code can be checked against it.** `ascribe permalinks verify`, given the ids a product uses, fails when one isn't in the list or has been retired. Run in the product's CI, it means a help link to nothing breaks a build, not a reader's click. GitLab does the same job with a linter for each of its languages; one list makes it one command.
- **It's how Ascribe knows a permalink went away.** `check` sees the files as they are now, with no memory. An id in the list that's neither in the content nor retired was removed, and can be reported. A reviewer also sees the line disappear from a file in the pull request.

The cost is one more file that follows the content, like `ascribe.lock`. `check` says when it's behind, a command and the editor bring it up to date, and adding a permalink without doing so is reported, not silently accepted.

## Retiring one

Features are deprecated and products reach the end of their life, so permalinks have to be removable. The rule is that it's deliberate and recorded, never blocked.

An id leaves the content, and an entry says what happens to its address from then on:

```toml
[permalinks.retired]
rotate-keys = { to = "guides/security.md", reason = "Merged into the security guide" }
legacy-sync = { reason = "The sync agent was removed in 4.0" }
```

- **With `to`,** the address leads to another page or heading.
- **Without it,** the address shows a short page saying the content was removed, and why. With [versions](versions.md), it can point at the archive of the last version that had it.

Where the friction is:

- **A reason is required,** as it is for an acknowledgement in the [content checks plan](content-checks/README.md). One sentence, written by someone who had to stop and think about who links here.
- **The message says what can't be seen.** "`rotate-keys` is a permalink. Links to it from outside these docs, such as from the product, aren't found by this check."
- **The editor offers the fix,** which writes the entry and leaves the cursor in the reason.

Where it isn't:

- **Removing one without retiring it is a warning, not an error.** The build goes ahead, and the address shows the "removed" page with no reason. A team that wants it stricter raises the level.
- **Moving isn't removing.** A heading that moves to another page, or a page that's renamed, takes its id with it. Nothing is recorded.
- **Many at once.** `ascribe permalinks retire` takes a page or a folder and one reason, so the end of a product isn't forty entries written by hand.

Retired entries pile up, and that's intended: they're the record of promises ended on purpose.

## More than one name for a place

Three screens of a product may all send readers to the same section today, and each should be able to change its mind later without the others moving. Help systems allow several ids for one topic for that reason. A heading has one `@id`, so the others are declared apart from it:

```toml
[permalinks.aliases]
agent-settings = "settings"
cli-settings = "settings"
```

Each alias has its own address and its own line in the list, and leads where its target leads until someone repoints it.

## What a link can carry

A product knows things about its reader that the docs don't: which version is running, on what platform. Firefox's help links carry the version, the operating system, and the language, and the site decides which of them to act on.

A permalink's address can take the same kind of context, as parameters named for the content model's own dimensions:

```
/go/rotate-keys?self-managed=3.4
```

The page at that address reads them. With versions declared, it sends the reader to that line, or to its archive. With a dimension the site shows as tabs, it opens the right one. A parameter the site doesn't know is ignored, so a product can send what it has without asking first.

## Ascribe's own links

The binary would stop building addresses and link to ids:

- A diagnostic's link becomes `/go/ASC036`. The reference's headings take the codes as their ids.
- A command's `--help` ends with `/go/cli-check`.

A test checks that every id the binary can produce is in the docs' list. Then the docs can be moved under `/docs/`, as the [website proposal](website.md) needs, with no address in the binary changing.

## What this leaves out

- A server. Addresses are pages in the site's output.
- Links between pages by id.
- Finding which permalinks nothing uses. Ascribe can't see the product's code. `verify` answers the question in the other direction, and a report of unused ids would need the product to say what it uses.
- Serving a section's text to a product, so help appears inside it. See "Embedded help" in the [brainstorm](brainstorm.md).

## Later

- **Links from one project into another.** A project's links stay inside it today. A project that can read another's list could link into it by id and have the link checked. The website's pages linking into the docs are the first case.
- **The list in a form code imports:** constants for a language, generated from the list, so a wrong id fails to compile.

## Open questions

- **Does one docs set need separate sets of names for separate products?** DITA pairs every id with an application's name, so two products can each have a `settings`. The alternative is a convention (`agent-settings`, `cli-settings`) and one flat set. A convention costs nothing; a real namespace would let `verify` check one product's ids alone.
- **Should the same permalink on two pages mean "the same page in different versions"?** If a permalink only has to be unique within what one build publishes, a page rewritten for 3.5 and the page it replaces could carry one id, and the version switcher would follow it. That would do the job of `replaces` in the versions proposal with no new key. It would also mean a permalink is needed on every page that's ever rewritten.
- **Who may add a permalink?** The id is a contract between the docs and the product. Declaring it in the docs' source means the product's team proposes one with a pull request against the docs. Is that the intended way, or should the product be able to reserve a name before the page exists?
