# Phase 3: The words everywhere, and the maker

Part of [The website, and the brand on everything else](README.md). Needs phase 1. It widens the thread from the site to every other surface that says what Ascribe is or who makes it.

## Goal

- **One home for the sentence and the headline.** Every other surface that says what Ascribe is either takes them from it or is tested against it.
- **Bryo Studio as the maker** everywhere a maker is named, the extension's publisher included.

## Context

- [The story](../brand/story.md), "One home for the words" and "Bryo Studio everywhere a maker is named".
- What says what Ascribe is today, quoted in the inventory this plan started from:
  - `README.md`'s opening;
  - the `description` in each `packages/*/package.json` and in `packages/vscode/package.json`, which is the Marketplace's;
  - `crates/ascribe-cli/Cargo.toml`;
  - the command's `about` in `crates/ascribe-cli/src/cli.rs`;
  - `DESCRIPTION` in `crates/ascribe-cli/src/agents/plugin.rs`, which writes the plugin's manifests;
  - the MCP server's instructions;
  - `docs/content/getting-started.md`'s first sentence;
  - the site's meta descriptions.
- `design/mark.toml`: it already holds the social card's line.
- [project-docs/outside.md](../outside.md): the Marketplace publisher, the GitHub repository's description and topics, and the npm organization.
- "One home per fact" in `AGENTS.md`.

## Design

### The home

The phase decides where it lives, and the obvious place is beside the social card's line. Either `design/mark.toml` grows a `[words]` table, or a new file takes the line and the sentence, and `mark.toml` points at it. It holds:

- the headline;
- the sentence;
- a short form under 100 characters for a package description, such as "A toolchain for documentation written as code.".

### Each surface

| Surface | What it says |
|---|---|
| `README.md`'s first paragraph | The sentence, then the four pillars as its bullets |
| The Marketplace description | The short form, with the editor's part in it |
| Each npm package's `description` | The short form, then the package's job. Each gets `keywords` and a `homepage` on ascribed-dev.com |
| `crates/ascribe-cli` | The short form |
| The command's `about` | The short form. Its list of subcommands is left to `--help` |
| The plugin's `DESCRIPTION` | Starts with the short form, then what the plugin adds |
| Getting started's first sentence | The sentence |
| The site's meta descriptions | The landing page's is the sentence, from phase 1 |

The MCP server's instructions are for an agent, not a reader, and keep their own wording. SPEC.md and the docs keep "language", "content model", and "compiler" where they're defining things.

A test compares each surface with the home. Where a file can be generated from it, as `plugin.rs`'s manifests already are, it's generated.

### The maker

- **The plugin's `author` and `owner`** become Bryo Studio, `https://bryostudio.com`, in `plugin.rs`, and the generated manifests are rewritten.
- **The VS Code publisher** moves to a Bryo Studio publisher, which the maintainer creates on the Marketplace. The extension's id changes from `Ascribe.ascribe-vscode` to the new publisher's. Every place that names the id changes with it: the landing page's and getting started's install commands, which a test keeps the same, the docs, and the release workflow. The old listing is unpublished. There's no migration (decision 8).
- **`project-docs/outside.md`** records the new publisher, who can change it, and the GitHub repository's description and topics, set by hand from the home.

### The record

- **A decision** for the publisher and the extension's new id.
- **A changelog entry** under the editor's area. It's a **Behavior change**, because the extension's id changes and it's installed from a new listing.

## Tasks

1. Choose the home, write the three lines in it, and add the test.
2. Change each surface in the table, generating where possible.
3. Move the plugin's author and owner to Bryo Studio.
4. Once the maintainer has created the publisher, change the extension's id everywhere, and publish from the new listing at the next release.
5. Update `outside.md`, write the decision, and write the changelog entry.

## Out of scope

- The MCP server's instructions and the skill's text, which are for agents doing a task.
- SPEC.md's definitions.
- The crates' descriptions other than `ascribe-cli`'s. They aren't published (`publish = false`), so nobody reads them as a product's.

## Acceptance criteria

- Changing the sentence in its home fails a test until every surface in the table agrees.
- No surface names Ascribe as a maker, publisher, author, or owner.
- `code --install-extension` with the new id installs the extension, once the next release is published.

## Verify

The full set in `AGENTS.md`, and `cd site && npm test` after a build.

## Commits

1. "Give the sentence and the headline one home"
2. "Say what Ascribe is the same way on every surface"
3. "Name Bryo Studio as the plugin's author"
4. "Publish the extension as Bryo Studio"
