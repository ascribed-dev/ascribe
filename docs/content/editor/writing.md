---
type: reference
title: Writing in the editor
description: "Completion, hover, navigation, renaming, formatting, and highlighting for Ascribe pages in VS Code."
---

What the editor does as you write a page: what it completes, what it shows on hover, where it takes you, and what it keeps in step when you rename.

## Completion

- After `@` at the start of a line: the built-in directives and the project's widgets, with their descriptions.
- In an attribute block: the directive's attribute keys, then their allowed values, such as note types or a dimension's values with their labels. After an image: the declared image attributes.
- In `@available:` and frontmatter `available:`: targets, dimension names, feature keys, and lifecycle states.
- In frontmatter `type:`: the content types `ascribe.toml` declares, each with what it's for.
- After `{` in text: the declared phrases, each with its value.
- In `@include:`: source files, then, after `#`, the ids of the file.
- In a link destination: pages and headings, **searched by title**. It inserts the file path and id, so you never need to type a path.

@available: ascribe 0.4.0
- `@term` only at the top of a section on a glossary page, or in a fragment one includes, under the heading and the directives that bind it, and not on a heading that's a term already. Inside its braces, `aliases`, `match`, and `case-sensitive`, and `match`'s values `first`, `every`, and `marked`.

@available: ascribe 0.3.2
- In a source's [`ascribe.examples.toml`](../examples/tested.md): its keys, and an environment entry's `label` and `command`. The file's problems appear as you type, where each is. A config no source in `ascribe.toml` names is just a TOML file.

@available: ascribe 0.4.0
- In the [navigation file](../navigation/index.md), `ascribe.nav`: its keys, a `folder` from the project's sections it doesn't list yet, and in `pages` the section's pages it doesn't name yet. The file's [problems](../navigation/index.md#problems) appear as you type, where each is, with their fixes, and a page its section leaves out offers **Add `name.md` to `folder/`'s pages in ascribe.nav**, which edits the file.

## Hover

- A link, image, or include: its target's path, title, and the first paragraph, with phrases filled in. A fragment, a missing file or id, a link written as a URL, and a name whose case doesn't match are called out.
- A phrase: its value.
- An availability spec or feature key: what it means, in the words the published site uses.
- A directive or attribute key: its description. A dimension key: its values and labels.

@available: ascribe 0.4.0
- A glossary term, in prose: its definition for the editor's build. On the term's heading: the definition, and how many pages use the term.

@available: ascribe 0.3.2
- A version in an availability spec, for a target with [release lines](../versions/lines.md): the line it falls in, and that line's status, such as "in line 3.4, live".

@available: ascribe 0.3.2
- `replaces` in frontmatter: the page it replaces, and which page each line publishes. Go to Definition opens the replaced page.

@available: ascribe 0.3.2
- A `@snippet`'s address: its file, as a link that opens it, and its region and lines. For a [tested example](../examples/tested.md), what it was tested with, "Changed after its last test run", or "Not tested", with the observations behind it, and where to run `ascribe observe`.

## Navigation

- **Go to definition** on a link or include opens the target file at its heading. On an `@id`, it goes to the heading; on a glossary term in prose, to its heading on the glossary page; on a phrase or feature key, to its entry in `ascribe.toml`.
- **Find All References** (`Shift+F12`, or **Go to References**) lists every place that uses what's under the cursor: on a heading or its `@id`, the links to it, including links through a page that includes it, and the includes of its section; at the start of a page, or on its frontmatter or title, the links and includes from other files; on a link or include, those of what it names; on a phrase, a feature key, a glossary term, a note's type, a widget, or a `@variant` attribute, every use of that phrase, feature, term, note type, widget, or dimension; on a page's frontmatter `type`, the pages of that type. A glossary term counts where its text, or an alias, is in prose, whether or not the build links it there, and where a link names its heading. A term with `match = "marked"` is only a word until you link it, so those links are its only uses. On a term's heading, or its `@term`, the list is the term's uses and the links to the heading together, each once. In `ascribe.toml`, on a content model entry's key or table header, it lists the entry's uses, as the Content model view's **Find All References** does.
- **Document links**: links, images, and includes are clickable. A link outside the docs is clickable when it's a web address (`http`, `https`) or an email address (`mailto`); one with any other scheme, such as `tel:` or `vscode:`, stays as it's written for the site's readers, and isn't a link in the editor.
- **CodeLens** above each `@include` names the included file and section, and opens it.
- **Inline hints** show the title an empty-text link, `[](keys.md#rotate-keys)`, will get.

## Code files a page shows
@available: ascribe 0.4.0

A code file a page shows with [`@snippet`](../writing/directives.md#snippet) says which pages show it: above each region a snippet takes, and on the first line of a file a snippet takes whole, a lens reads "Shown in 3 pages". A page counts once however many of its `@snippet`s show the region, and a fragment's `@snippet` counts each page that includes the fragment. Choose the lens to see the places in the references peek, as **Find All References** lists them, and choose one to open the page at its `@snippet`. It's how you find the pages a change to your code will change, as you make it.

The lens is placed from the saved file, as a build reads it: the editor doesn't send code files to the language server. Lines you add above a region leave its lens where the region was until you save the file, wherever the source's folder is: in the project's folder, beside it, or in another project's. It's there for the files of the content model's [sources](../content-model/output.md#18-sourcesname), Markdown files among them, while the project's language server is running, and nowhere else. A file more than one project shows gets a lens from each project, each counting its own pages and naming the project, such as "docs: shown in 3 pages", since those pages are built and published apart; a file only one project shows has one lens, unnamed, whichever other projects' sources it's in.

## Refactoring

- **Rename or move a file or a folder** in the Explorer, and the links and includes that point to what it holds are updated, and so are the glossary's `pages` and the retired permalinks' `to` in `ascribe.toml` that name it, the `about` of each [memo file](actions.md#memo-files) about it or a heading in it, and [`ascribe.nav`](../navigation/index.md#the-file): a page's entry in its section's `pages`, taken out when it leaves the folder, and a folder's `folder`.
- **Rename** (`F2`) a heading's `@id`, or its text, to update the links to it, the retired permalinks' `to` in `ascribe.toml` among them, and the memo files about it, or a phrase key to update its uses, its declaration, and the memo files about it. A glossary term's id is its heading's, so renaming the `@id` renames the term, the memo files about `term:<id>` with it, and the site output's `data-ascribe-term` carries the new id. Rewording a term's heading changes the term's text; when the heading has no `@id`, its id changes too, and the editor says so.
- **Rename** (`F2`) a dimension value, in a `@variant` attribute or in its dimension's `values` in `ascribe.toml`, to update it everywhere: the dimension's `labels` and `versionless`, every `@variant` attribute and `variant:` field, every availability spec in pages, table rows, frontmatter, and `[features]`, and the builds' `variants` and `filter`.
- A phrase rename reaches every use: prose, link text and destinations, frontmatter fields with `phrases = true`, code blocks with `phrases=true`, and the files a `@snippet {phrases=true}` shows.
- `F2` on something that can't be renamed says why, and on something that can, selects what will be renamed. A new name the content model doesn't allow, or that's already taken, changes nothing.
- From the command line, [`ascribe rename`](../command/questions.md#cli-rename) makes the same edits, for a phrase, a dimension value, a heading's id, or a page that moves.

## Formatting

**Format Document** rewrites Ascribe constructs into canonical form, as `ascribe fmt` does. Turn on `ascribe.formatOnSave` to do it on every save. It changes only Ascribe constructs, never how a page renders.

When another extension is VS Code's formatter for Markdown, such as Prettier through `editor.defaultFormatter`, it formats your pages too, and can change what they build to: `version: {version}` becomes `version: { version }`. When you open a page and another formatter is set, the extension offers to be the Markdown formatter for that workspace folder. **Use Ascribe** writes `editor.defaultFormatter` under `[markdown]` in the folder's `.vscode/settings.json`, so **Format Document** and `editor.formatOnSave` run Ascribe on your pages. Ascribe formats only its projects' pages, so other Markdown files in the folder are no longer formatted. **Keep** leaves the setting as it is and doesn't ask again for that folder.

## Highlighting

Directive lines, attribute blocks, `@end`, and phrases are highlighted as soon as a file opens. Once the server is running, it adds what depends on the content model: declared and undeclared phrases, project widgets, and **title lines**, shown distinctly so a paragraph that accidentally became a title stands out.

@available: ascribe 0.4.0
The extension gives `ascribe.nav`, `ascribe.permalinks`, and `ascribe.lock` to VS Code's TOML language, like `ascribe.toml`. VS Code has no TOML coloring of its own, so they're colored only with a TOML extension installed, such as Even Better TOML.

@available: ascribe 0.4.0
A [memo](../writing/directives.md#memo)'s text is colored as a comment, in either form, and nothing in it is colored as a link, a phrase, or a heading. A container memo folds to its first line, beside the folding of headings, lists, and code blocks.
