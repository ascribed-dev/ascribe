<h1>
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="design/out/header-dark.svg" />
    <img src="design/out/header-light.svg" alt="Ascribe" width="235" height="45" />
  </picture>
</h1>

[Website](https://ascribed-dev.com) · [Docs](https://ascribed-dev.com/go/docs/) · [Install](https://ascribed-dev.com/go/install/) · [Changelog](CHANGELOG.md)

[![CI](https://github.com/ascribed-dev/ascribe/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/ascribed-dev/ascribe/actions/workflows/ci.yml)
[![npm](https://img.shields.io/npm/v/@ascribed/cli?label=npm)](https://www.npmjs.com/package/@ascribed/cli)
[![VS Code](https://vsmarketplacebadges.dev/version/Ascribe.ascribe-vscode.svg?label=VS%20Code)](https://marketplace.visualstudio.com/items?itemName=Ascribe.ascribe-vscode)
[![License: MPL-2.0](https://img.shields.io/badge/license-MPL--2.0-blue)](LICENSE)
[![Status: early](https://img.shields.io/badge/status-early-orange)](https://ascribed-dev.com)

Ascribe is a toolchain for documentation written as code: Markdown with the structure docs need, checked as you write, and built into a website, plain Markdown, and JSON.

````markdown
## Install the agent
@available: cloud, self-managed 3.3

@variant {pm=npm}:
```sh
npm install -g @quill/agent
```
@variant {pm=pnpm}:
```sh
pnpm add -g @quill/agent
```
@end

@note {type=tip}: {product} checks your configuration on startup.

See [](keys.md#rotate-keys) to rotate your API key.
````

- **The tools of a docs team.** Reusable content with `@include` and phrases, variants by platform or product shown as tabs, and a build for each release line from one source. A glossary, page types and templates from a content model in `ascribe.toml`, notes and steps, and a page and site preview in VS Code.
- **Checked like code.** Broken links, missing headings, and undeclared names are errors, in the editor as you type and in CI. Code examples are read from the code's own files with `@snippet`, and a pull request is reviewed as rendered pages, with what changed marked.
- **Built for agents, writing and reading.** AI agents that write pages get the project's rules from `ascribe agents sync`, the checks through the MCP server, and a hook that checks after each edit. Agents that read a published site get `llms.txt`, a Markdown version of every page, and each permalink's help as JSON. The build's page JSON is for tools, and isn't served.
- **Plain files you can leave.** A page reads as Markdown without Ascribe, and the plain output is CommonMark. Ascribe hosts nothing and calls no model, and it's open source under the MPL.

## Get started

```sh
npm install --save-dev @ascribed/cli
code --install-extension Ascribe.ascribe-vscode
npx ascribe check
```

To install it another way, such as without Node, see [Install](https://ascribed-dev.com/go/install/).

Read [Getting started](https://ascribed-dev.com/go/getting-started/), then the rest of the [documentation](https://ascribed-dev.com/go/docs/).

To write docs with an AI coding agent, such as Claude Code, GitHub Copilot, Cursor, or Codex, read [Agents](https://ascribed-dev.com/go/agents-guide/): `npx ascribe agents sync` gives your agent the project's rules and a skill that teaches it to check its own work.

| Package | What it is |
|---|---|
| [`@ascribed/cli`](packages/cli) | The `ascribe` command: `check`, `build`, `fmt`, and the language server |
| [Ascribe for VS Code](packages/vscode) | The editor extension (`Ascribe.ascribe-vscode`) |
| [`@ascribed/astro`](packages/astro) | The Astro integration |
| [`@ascribed/elements`](packages/elements) | The web components the site output uses |
| [`@ascribed/review`](packages/review) | Pull request review threads, placed on rendered pages |

The [specification](SPEC.md) defines the language. [`examples/quill`](examples/quill) is a small, complete project, [`examples/monorepo`](examples/monorepo) is a repository with several, and [`examples/astro-site`](examples/astro-site) publishes one with Astro.

## Working on Ascribe

The compiler is a Rust workspace; the npm packages and the extension are a pnpm workspace.

| Directory | Contents |
|---|---|
| `crates/` | The compiler: `ascribe-syntax` (parsing), `ascribe-model` (`ascribe.toml`), `ascribe-resolve` (includes, builds, links), `ascribe-check`, `ascribe-emit` (outputs), `ascribe-diff` (what changed since a git revision), `ascribe-sources` (sources in other repositories), `ascribe-fmt`, `ascribe-query` (answers about a project, for the commands agents use), `ascribe-lsp`, `ascribe-cli` (the `ascribe` binary), and `comrak-ascribe`, a fork of the CommonMark parser |
| `packages/` | `cli`, `astro`, `elements`, `review`, and `vscode` |
| `tests/` | The conformance suite and its diagnostics registry, the CommonMark suite, real-world corpora, and cross-implementation fixtures |
| `examples/` | Example projects and content models |
| `docs/`, `site/` | The user docs, an Ascribe project, and the Astro site that publishes them, installed from npm outside the workspace |
| `scripts/release/` | Versioning, packing, and publishing a release |

```sh
cargo test --workspace                 # the compiler, with the conformance suite
corepack enable && pnpm install
pnpm -r test                           # the packages
```

[ARCHITECTURE.md](ARCHITECTURE.md) is the map: what each crate and package is for, and how they reach each other. Each crate and package has a README with its own details. [CONTRIBUTING.md](CONTRIBUTING.md) says what CI checks, and [SECURITY.md](SECURITY.md) how to report a vulnerability. [RELEASING.md](RELEASING.md) is how a release is made.

## License

[Mozilla Public License 2.0](LICENSE). Ascribe is made by [Bryo Studio](https://bryostudio.com).
