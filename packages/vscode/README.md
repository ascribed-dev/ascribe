<p><img src="https://raw.githubusercontent.com/ascribed-dev/ascribe/main/design/out/header-vscode.png" alt="Ascribe" width="266" height="96"></p>

# Ascribe for VS Code

Write [Ascribe](https://github.com/ascribed-dev/ascribe) documentation with the checks, completion, navigation, and preview a programming language gets. Ascribe is Markdown with directives for the structure documentation needs: callouts, procedures, alternatives by platform or product, availability, and reusable content.

The extension activates in a workspace that contains an `ascribe.toml`. A workspace can hold several projects, each with its own language server: see [workspaces with several projects](https://ascribed-dev.com/guides/editor/#workspaces-with-several-projects).

## Features

- **Diagnostics as you type**, across the whole project, the same ones `ascribe check` reports in CI. Many come with a quick fix.
- **Completion** for directives, attributes and their values, phrases, availability, include paths, and links, which you search by page and heading title rather than by path.
- **Hover** for links and includes (the target's title and first paragraph), phrases (their values), and availability (what it means on the published site).
- **Navigation**: go to definition, Find All References for pages, headings, phrases, and the rest of the content model, clickable links, a CodeLens on each include, and hints that show the title an empty-text link will get.
- **Refactoring**: renaming or moving a file updates the links and includes that point to it; renaming a heading's `@id`, a phrase, or a dimension value updates its uses.
- **Formatting** into canonical form, on request or on save, and **highlighting** of directives, phrases, and title lines.
- **A live page preview** that renders the page as the published site does, as you type, for any build, and a **site preview** that opens it on the site's dev server.
- **Actions** that write Ascribe for you: wrap a paragraph in a note, turn a list into steps, link to a page by its title, insert content that varies, mark where a section is available, or make the selected text a phrase. **The actions bar** (`Ctrl+K A`, `Cmd+K A` on macOS) lists the ones that apply where the cursor is; each asks what it needs, so you never type the syntax.
- **The Ascribe sidebar**: every project in the workspace, what links to the page you're on, the project's pages and the ones nothing links to, and its content model with how much each entry is used.
- **The build you're looking at**, in the status bar: switch it, and the preview renders it and **the build lens** dims what it leaves out of the page.
- **Review**: the preview marks what changed against a git revision, lists the pages a change touches, and shows the pull request's review comments beside the blocks they're about, where you can reply, resolve, comment, and submit your review. [Reviewing a pull request](https://ascribed-dev.com/guides/review/) walks through it.
- **Agents**: an agent in VS Code gets Ascribe's MCP server with nothing to set up, and tools that see what's in the editor before it's saved and the review that's open. **Prompt agent** builds a prompt about a problem or a review comment for your agent. See [Agents in VS Code](https://ascribed-dev.com/guides/editor/#agents-in-vs-code).

After you install it, the **Get Started with Ascribe** walkthrough goes through the first steps. It's under **Help → Welcome** whenever you want it again.

## The `ascribe` binary

The extension includes `ascribe` for your platform, so it works immediately. When a project installs its own (`npm install --save-dev @ascribed/cli`), the extension uses that one for it instead, so the editor matches the version the project pins for CI. The `ascribe.path` setting overrides both.

## Settings and commands

The extension's **Feature Contributions** tab lists them, and [Editing with Ascribe](https://ascribed-dev.com/guides/editor/#settings) describes each one.

## Learn more

- [Editing with Ascribe](https://ascribed-dev.com/guides/editor/): everything the extension does.
- [Review](https://ascribed-dev.com/guides/review/): reviewing a pull request in the preview.
- [Getting started](https://ascribed-dev.com/getting-started/) with Ascribe.
- [Diagnostics](https://ascribed-dev.com/reference/diagnostics/): every problem Ascribe reports, and its fix.

Other Markdown formatters that reflow paragraphs don't know that Ascribe's directive and title lines start new blocks; exclude Ascribe files from them.
