---
type: reference
title: "`ascribe templates`"
description: "List the page templates Ascribe carries, write them into a project with their content types, and say what's changed in the set since."
---

The commands that write the page templates into a project and say what's changed in them since. [Page templates](index.md) is the task they're used in. The options every command accepts are in the [command reference](../command/index.md#options-every-command-accepts), and the JSON each writes with `--format json` is in [their JSON reports' schemas](../contracts/json-reports/templates.md).

## `ascribe templates`
@id {permalink=true}: cli-templates
@summary: Writes the page templates Ascribe carries into a project, as its own files, and declares a content type for each.

Writes the page templates Ascribe carries into a project, as the project's own files, and declares a content type for each, which takes them as its [template and guide](../content-model/index.md#41-page-types) from `templates/templates.lock`. The templates are [The Good Docs Project](https://www.thegooddocsproject.dev/template)'s, converted into Ascribe pages, as [Where the templates come from](index.md#where-the-templates-come-from) says.

The set has six templates, which `ascribe templates list` names. A page is started from a template with [`ascribe new`](index.md#cli-new), or the editor's **New Page**; the template names its type in its frontmatter, as the page must. `eject` adds to a project, and never starts one: [`ascribe init --templates`](../install.md#cli-init) does, with the set. `status` says what's changed in the set since the project's copy, and changes nothing.

### `ascribe templates list`
@id {permalink=true}: cli-templates-list

@summary
Shows each template the set has, in a line, with what it's for, and which release of theirs the set was converted from. It reads only what the binary carries, so it works anywhere, in a project or not. It takes no options, and exits with `0`.

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/templates-list.txt

### `ascribe templates eject`
@id {permalink=true}: cli-templates-eject

@summary
Writes each template named, with its guide and Ascribe's notes on its type, into the project, and adds its type to `ascribe.toml`. With no name, it takes every template of the set.

```sh
ascribe templates eject how-to
```

@include: ../_generated/cli-templates-eject-options.md

For each template, it writes the template, its guide, and Ascribe's notes on the type into `templates/`, beside `ascribe.toml`, or into `--to`'s folder, which is a path from the current folder. The folder must be inside the folder `ascribe.toml` is in, and outside the content root and the output directory, where no type can name a file. Then it adds the type at the end of `ascribe.toml`, changing nothing above it, so its comments and order stay as they are:

```toml
[types.how-to]

[types.how-to.frontmatter]
description = "string"
```

The type names no `template` or `guide`: a type that names neither takes them from `templates/templates.lock`, which records them under its name, the guide theirs and then Ascribe's notes, in the order to read them. The content model looks for the lock only in `templates/`, so with `--to` another folder, `eject` names them in the type's table, as paths from the folder `ascribe.toml` is in, whatever folder `--to` was given from:

```toml
[types.how-to]
template = "writing/how-to.md"
guide = ["writing/how-to.guide.md", "writing/how-to.ascribe.md"]
```

Its frontmatter has the fields the template's has, apart from `type`, each a string a page must have, and apart from those every type has already: a field `[frontmatter]` declares, and `title` as a plain string. When one of the project's types declares one of those fields as a string with more to it, such as `title = { type = "string", inline = "code" }` or `description = { type = "string", role = "description" }`, the type added declares it the same way, so its pages' titles and descriptions read as the project's other pages' do. A project that declared no types had every page of the implicit `page` type, so `eject` writes that type out first, as it was, a type with nothing of its own:

```toml
[types.page]
```

A project that declares types has each page name one, so its pages that named none, which were of `page`, now need `type: page`: `eject` lists them, and `ascribe check` reports each with a fix that adds it.

Beside the templates, it writes `templates.lock`: for each template, the files written, with a hash of each as written, their template and guide under `files` and Ascribe's notes under `ascribe-files`, then their repository, tag, commit, and license, their summary of the type, and the version of Ascribe that wrote it. A later `eject` adds to it. Commit it with the templates, and don't edit it: the types take their templates from it, and a file it names that isn't there is the type's error, `model-type-template`.

A project that declares a template's type already, with its own frontmatter, takes the template for it with `--keep-type`:

```sh
ascribe templates eject reference --keep-type
```

It writes the same files and `templates.lock`, and leaves `ascribe.toml` as it is: the type takes its template and guide from the lock. With `--to` another folder, it adds only `template` and `guide` to the type's table, after its last entry, leaving its frontmatter, its other keys, and every comment as they are. A template whose type isn't declared is added as without it. It refuses a type written as an inline or a dotted table, which it can't add a line to, rather than a table under a `[types.<name>]` header of its own; a type that names a `template` or a `guide` already, or takes them from the lock; and a type whose frontmatter doesn't declare each field the template's frontmatter has as a string, since no page started from the template would check. It names the field.

It checks everything before it writes anything. A type `ascribe.toml` declares already, without `--keep-type`, a file where one would be written, and a name the set doesn't have are each named, on standard error, and then nothing is written. Without `ascribe.toml`, it says so, and that `ascribe init --templates` starts a project with the templates.

It lists each file it wrote, `ascribe.toml` last, from the current folder; each type it added, with the fields it declared as the project's other types do; the pages that name no type, up to ten, which `ascribe check` reports until each has its `type:`; each type it kept, with the keys it left as they were; and how to start a page. In a project `ascribe init` started:

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/templates-eject.txt

With `--format json`, it writes the document its [schema](../contracts/json-reports/templates.md#ascribe-templates-eject) describes. When `refused` isn't empty, `written`, `added`, and `kept` are.

.The JSON for this example
@details:
@snippet {lang=json}: code:crates/ascribe-cli/tests/output/templates-eject.json
@end

| Code | Meaning |
|---|---|
| `0` | Every template named was written, and its type added, or kept, taking its template and guide from the lock |
| `2` | It wrote nothing: a template was refused, there's no `ascribe.toml`, the content model has errors, the folder can't hold templates, `templates.lock` isn't one it can add to, or the types can't be added to `ascribe.toml`. Or a file couldn't be written; the files it wrote before that stay. |

### `ascribe templates status`
@id {permalink=true}: cli-templates-status
@summary: Says what's changed in the templates since the project took them: in the set this `ascribe` carries, and in the project's copies, and which templates the project doesn't have.

Says what's changed in the templates since the project took them: for each one, whether the set this `ascribe` carries has changed it since the copy, and whether the project has changed its copy; then each template of the set the project doesn't have. It compares the project's files and `templates.lock` with the set the binary carries, and reads nothing else: no network, so what it says is as of this `ascribe`, and its first line names the release of theirs the set is from. It changes no file: a team that wants a change copies it by hand.

@include: ../_generated/cli-templates-status-options.md

`templates.lock` keeps a hash of each file as `eject` wrote it, and that's what tells the two apart: when the set's text isn't what was written, the set changed it; when the project's file isn't, the project did. A template can be both. A file the set changed and the project didn't is listed with what's different by section: each heading the set added, removed, or changed the text under, with `the start` for what comes before the first heading. A file that's gone is `deleted`. Line endings a checkout changed to CRLF aren't a change.

It finds a template's entry in the `templates.lock` in `templates/`, or beside the template a type names, and finds the templates a project took from the lock's entries and from the types named as the set's templates are. When there's no lock, it's one `eject` didn't write, or it has no entry for a type of the set's, it says so, and says what it can from the files alone: whether each is the same as the set's or different, without telling who changed it.

With `--diff NAME`, it prints only the change to that template, as a unified diff from the project's files to the set's, for each file the set has changed since the copy. When the project hasn't changed its copy, that's the upstream change itself; when it has, or the lock remembers nothing of the file, it says so on standard error, since the diff then undoes the project's changes too.

In the project above, once its copy of the how-to template has a section of its own:

@snippet {lang=text}: code:crates/ascribe-cli/tests/output/templates-status.txt

With `--format json`, it writes the same in the document its [schema](../contracts/json-reports/templates.md#ascribe-templates-status) describes. Paths are from the folder `ascribe.toml` is in.

.The JSON for this example
@details:
@snippet {lang=json}: code:crates/ascribe-cli/tests/output/templates-status.json
@end

| Code | Meaning |
|---|---|
| `0` | It ran, whatever it found |
| `1` | With `--exit-code`, the set has changed a template the project took, or no longer has one; with `--diff`, the one named. Or `--diff` names a template of the set the project hasn't taken. |
| `2` | It couldn't run: no `ascribe.toml`, one that doesn't load, or `--diff` with a name the set doesn't have |
