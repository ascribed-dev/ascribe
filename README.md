<h1>
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="design/out/header-dark.svg" />
    <img src="design/out/header-light.svg" alt="Ascribe" width="218" height="48" />
  </picture>
</h1>

Ascribe is a markup language, content model, and toolchain for documentation written as code. It's Markdown with a small set of directives for the structure documentation needs: callouts, procedures, alternatives by platform or product, availability by deployment and version, and reusable content. A schema, `ascribe.toml`, says what a documentation set may contain; the editor checks it as you type; and one compiler builds it into a website, plain Markdown, and JSON.

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

- **It reads as Markdown.** A directive is one line that starts with `@`. There are no conditionals, loops, or computed content.
- **It's checked.** Links and includes name files, not URLs, and must exist. Attributes, frontmatter, and availability are validated against the content model. `ascribe check` in CI reports exactly what the editor reports.
- **One source, several builds.** A build can keep every alternative as tabs, or only one deployment's, or only what's available in a given version.
- **It publishes with Astro.** The site output is Markdown with web components, loaded into an Astro content collection with a generated schema.

## Get started

```sh
npm install --save-dev @ascribed/cli
npx ascribe check
```

Read [Getting started](https://ascribed-dev.com/getting-started/), then the rest of the [documentation](https://ascribed-dev.com/).

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

[Mozilla Public License 2.0](LICENSE).
