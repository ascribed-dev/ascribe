---
type: reference
title: Actions, new pages, and memos
description: "The VS Code extension's actions, which write Ascribe for you, starting a page from its template, and memo files."
---

Actions write Ascribe for you: wrap a paragraph in a note, turn a numbered list into steps, link the selected text to a page, insert content that varies by a dimension, mark where a section is available. Each one asks what it needs in short steps, choosing from what the project has (its note types, pages and headings, phrases, dimensions and their values, features, images, fragments, sources, and widgets), so you never type a path, a key, or attribute syntax. What it writes is in canonical form, and one **Undo** takes it back.

Every action is in four places:

- The **actions bar** (`Ctrl+K A` / `Cmd+K A`), which lists only what applies where the cursor is. See [The actions bar](actions.md#the-actions-bar).
- The **Command Palette**, as `Ascribe: <action>`, in a Markdown file of a project. Run where it doesn't apply, it says where it does: "Put the cursor in a note to change its kind."
- The editor's context menu, under **Ascribe**, which lists the actions that apply where the cursor is.
- The lightbulb (`Ctrl+.` / `Cmd+.`), for the actions that rewrite what's at the cursor, beside the quick fixes.

**Insert a phrase** and **Insert a widget** also run from the **Content model** view: **Insert at the Cursor** in a phrase's or a widget's menu runs them with it chosen, at the cursor of the page you last had open. See [This page, Pages, and Content model](sidebar.md#this-page-pages-and-content-model).

@include: ../_generated/editor-actions.md

## The actions bar

Press `Ctrl+K A` (`Cmd+K A` on macOS), or run **Ascribe: Actions for the Cursor**, in a page of a project to see what you can do where the cursor is. The bar lists the fixes for problems at the cursor first, then the actions that apply to the cursor or the selection, grouped as **Write**, **Structure**, **Link**, **Media**, **Content model**, and **Memos**. Type to filter, and choose one: an action that needs something asks for it in the same box. When nothing applies, the bar says where to put the cursor.

![The actions bar open over a paragraph of a page, asking "What would you like to do here?": each action that applies there is listed with what it does and the syntax it writes, such as "Wrap in a note: Put the paragraph or the selected blocks in an @note callout", and each group, Write, Structure, Link, Content model, and Memos, is named at the right of its first action.](../images/actions-bar-light.png){scheme=light}
![The actions bar open over a paragraph of a page, asking "What would you like to do here?": each action that applies there is listed with what it does and the syntax it writes, such as "Wrap in a note: Put the paragraph or the selected blocks in an @note callout", and each group, Write, Structure, Link, Content model, and Memos, is named at the right of its first action.](../images/actions-bar-dark.png){scheme=dark}

The bar holds only what applies to the cursor or selection, so it never lists every action as the Command Palette does, and it has no searches or commands of its own.

To use another key, open **Preferences: Open Keyboard Shortcuts**, search for `ascribe.actions`, and change its keybinding. The default is on only in a Markdown file of a project, with the editor focused.

## Changing the content model

The actions in the bar's **Content model** group change `ascribe.toml`, or the glossary page, from a page: **Make this a phrase**, **Add to the glossary**, **Change a feature's availability**, **Rename this phrase everywhere**, and **Rename a dimension value everywhere**. **Add to the glossary** writes a term on the glossary page: see [Set up a glossary](../content-model/glossary.md). A rename changes what `F2` does (see [Refactoring](writing.md#refactoring)), for the phrase or value at the cursor, or one you pick from the project's wherever the cursor is.

These change `ascribe.toml` in place, keeping its comments, blank lines, and order, and an open `ascribe.toml` with unsaved changes is edited as it is. A rename that changes other files lists every change in the refactor preview first, where you check the changes to make and **Apply** them, or **Discard** them all.

## Start a page

**Ascribe: New Page** makes a page from its type's [template](../templates/index.md). Run it from the Command Palette, from the **Pages** view's title bar, or on a folder in the Explorer. It asks for the page's type, each with what it's for (the type's `purpose`, or for a type from the starter set, their summary of it), then its title. The page is the template with its `type` and `title` set, named for the title, in the content root; run on a folder, it makes the page in that folder. A page names its type in its frontmatter, so any folder of pages takes every type; a fragment's folder takes none. It's saved and opened with the first thing the template left to fill in selected, usually the description, so typing replaces it. A page is never made over a file: with a file of that name there, it says so, and you give another title.

**Ascribe: Go to Next Placeholder** selects the next thing the template left to fill in after the cursor: a frontmatter field still as the template has it, or a `\{placeholder}`, a whole one even when it runs across a paragraph and a list. After the last, it goes back to the first. They're the placeholders and fields [`template-placeholder-left`](../templates/index.md#whats-left-to-fill-in) reports, found the same way; the template's instructions aren't among them, since you read those rather than fill them in. To give it a key, open **Preferences: Open Keyboard Shortcuts** and search for `ascribe.nextPlaceholder`.

A type with no template makes a page with its frontmatter: its `type`, the title, and each other field the type requires, empty.

## Memo files
@available: ascribe 0.4.0

A [memo file](../writing/memo-files.md) says why a section or a block is written the way it is. The editor shows them where they apply, and writes them for you. [Leave a memo for whoever edits next](../writing/memos.md) says when to write one.

- A **CodeLens** above a heading or a block that has memos counts them: "1 memo", "2 memos", or "3 memos, 1 outdated" when one's quote isn't found any more. A lens at the top of the page counts the memos about the whole page. Click it to open the memo, or, with several, to pick one; each opens at its text.
- **Ascribe: Add a memo** makes a file in `memos/` about the section the cursor is in, by its id, or about the page when the cursor is above its first heading with an id. Its `quote` is the selected text, or the first line of the block at the cursor; on a heading it has none, and it's about the heading's section. The file is named from the quote's first five words, or the heading's id, cut to 100 bytes, and never made over a file that's there. When the file can't be written, the message says what the file system said. It's saved and opened at its text, so you start writing why. It's in the actions bar's **Memos** group, the Command Palette, and the editor's **Ascribe** submenu.
- **Ascribe: Add a personal memo** does the same in `.ascribe/memos/`, which stays on your machine: see [Personal memos](../writing/memo-files.md#personal-memos).
- In a memo file, completion after `about:` offers the project's pages, their headings with ids as `page.md#id`, and its phrases, features, glossary terms, and the rest by `prefix:name`, as [`ascribe refs`](../command/questions.md#cli-refs) takes them. `about` is a link to what it names, and the file shows the problems `ascribe check` would report in it as you type, a personal memo's too.

**This page**'s **Memos** section lists the memos about the heading at the cursor, or the page: see [This page, Pages, and Content model](sidebar.md#this-page-pages-and-content-model).
