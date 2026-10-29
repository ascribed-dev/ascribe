# Security review

One review of everything in Ascribe that handles input it doesn't control, to find what an attacker could make it do, fix what's exploitable, and harden what isn't yet. It finds and recommends; each fix is separate work, with a test that keeps the hole closed.

Ascribe's [security policy](../../SECURITY.md) says it parses documentation written by other people, and that the language server and the VS Code extension run inside the user's editor. That's the premise here: a repository, a pull request, an agent, a spec, or a page on the web can all be written by someone else.

## Who does what

- **The maintainer** decides: the threat model in phase 0, and in phase 6 which findings are fixed, in what order, and which are accepted.
- **An agent** does the rest, one phase per session, reading the code with the questions each phase file asks and writing what it finds. A phase's findings are checked by a second, adversarial agent before they're recorded: each must come with the input that triggers it, or it's a hardening note, not a vulnerability.

## What exists today

Checked against `main` on 26 October 2026, so a phase doesn't spend time re-finding it:

- **Reading project files** goes through `ascribe_resolve::FileSystem`, which knows the content root, the project boundary, exact-case names, and symbolic links; a test (`crates/ascribe-resolve/tests/all/file_reads.rs`) fails on a new direct `std::fs` read of a project file. A spec outside the project's folder (OpenAPI) and a `[sources]` folder are read with `read_file`, after the caller checks they're in the same git repository.
- **Ascribe's binary makes no network request.** External links go to lychee and the published site to afdocs, programs the project installs, run through `crates/ascribe-check/src/tool.rs`; prose goes to Vale; history and remote sources go to `git`.
- **`git` is constrained:** `crates/ascribe-sources/src/remote.rs` runs it with `protocol.allow=never` and only `https`, `http`, `ssh`, `git`, and `file` allowed, the five SPEC §7.4 names, and both it and `crates/ascribe-diff/src/git.rs` pass `--` before paths.
- **The site output passes raw HTML through** (`crates/ascribe-emit/src/render/mod.rs` sets comrak's `unsafe`): a page's author is trusted to write HTML, as in any Markdown site generator.
- **The extension's webviews** have a content security policy with a nonce (`packages/vscode/src/webview/viewShell.ts`), and the extension runs nothing in Restricted Mode (`packages/vscode/src/extension.ts`, `ui/tasks.ts`); it runs a project's own `node_modules/.bin/ascribe` before its bundled binary.
- **CI:** every third-party action is pinned to a commit, every workflow sets `permissions`, and `cargo deny` checks advisories, bans, licences, and sources (`.github/workflows/rust.yml`). Nothing audits the JavaScript dependencies. `report.yml` and `site-npm.yml` are triggered by a schedule and by `workflow_run`.
- **Releases:** npm packages are published with trusted publishing and provenance (`release.yml`, `canary.yml`); no package runs an install script.
- **Three fuzz targets exist,** `fuzz/fuzz_targets/` (`parse`, `model`, `format`), run nightly by `fuzz.yml`, which opens an issue per distinct crash (decision #1020). Frontmatter, the OpenAPI reader, and the example validator aren't fuzzed.
- **An earlier pass on 26 October landed decisions 75 to 80** in `project-docs/decisions.md` (`git` options, document link schemes, webview messages, the output directory and links, include limits, the GitHub token's host). The threat model lists them.

## Decisions

Settled for every phase. If one can't be met, stop and report.

1. **An exploitable finding is never public until it's fixed.** It goes into a draft GitHub security advisory on this repository, as `SECURITY.md` asks of anyone, not into an issue, a commit, a pull request description, or a file under `project-docs/`. The phase's committed notes say only that a private finding exists, and its severity.
2. **A finding is exploitable when it comes with the input that triggers it** and says what it achieves: a file read or written outside the project, a command run, script run in a page or a webview, a secret exposed, or a crash or hang a project's content can cause in a long-running process (the language server, the MCP server). Anything else is a hardening note.
3. **Who is trusted.** The person running Ascribe on their own project, and the content model they wrote, are trusted. Untrusted: a repository opened in the editor before Workspace Trust is granted; a pull request's content from a fork, in CI; a remote `[sources]` repository; an OpenAPI spec generated or fetched by a pipeline; anything an agent passes to the MCP server or the language server; review comments from GitHub; what lychee, afdocs, Vale, and `git` print; and the web pages a link check reaches. Phase 0 writes this down; later phases use it.
4. **An author writing HTML in a page is trusted,** since the site output passes it through by design. A finding about raw HTML is about where a page's HTML reaches someone the author isn't, such as the review overlay or the editor's preview, not about raw HTML itself.
5. **Each fix comes with a regression test** at the layer that proves it, and a fuzz target is kept in the repository and run in CI on a schedule, not on every pull request.
6. **No surface is redesigned by this review.** A finding recommends one change; a change that needs a design gets a proposal of its own.
7. **The trust boundaries are the threat model's eight** ([threat-model.md](threat-model.md)): the CLI in CI, the CLI on a writer's machine, the language server inside the editor, the MCP server under an agent, the agents' hook, the extension's webviews, the published site and its review overlay, and the release pipeline, each with the untrusted side its table names. `ascribe observe`'s shell line is trusted input run by a person's command, and in scope only where a workflow runs `observe` on a pull request's checkout, which `rust.yml` does for the tested example in a job that already runs the checkout's tests. _Proposed by phase 0 on 27 October 2026, for the maintainer to settle; a plan's decisions stay in its README while it runs ([decisions/README.md](../decisions/README.md))._

## Phases

| Phase | What it reviews |
|---|---|
| [0: Threat model](phase-0-threat-model.md) | Assets, actors, trust boundaries, and every entry point, written down and settled by the maintainer |
| [1: Untrusted content](phase-1-content.md) | Parsing, paths, and what the outputs write from a project's content, with fuzz targets |
| [2: Running other programs](phase-2-programs.md) | `git`, lychee, afdocs, Vale, and the binary the editor picks |
| [3: The servers](phase-3-servers.md) | The language server and the MCP server, and the agents' hook |
| [4: The editor and the web packages](phase-4-editor-web.md) | The extension's webviews and processes, `@ascribed/review`, `@ascribed/astro`, and the elements |
| [5: Supply chain and release](phase-5-supply-chain.md) | Dependencies, CI, and how a release is built and published |
| [6: Findings](phase-6-findings.md) | The maintainer reads the findings and says what's fixed, in what order |

Phase 0 comes first because every later phase asks "is this input trusted?" Phases 1 to 5 are independent and can run in any order; by likely impact, the order above. Phase 6 waits for all of them.

**Status, 27 October 2026:** phases 0 to 5 ran, each phase's findings checked by a second, adversarial agent. Five private findings are with the maintainer (two high, two medium, one low; decision 1), and the hardening notes are in `notes/phase-1.md` to `notes/phase-5.md`, each ending with what was checked and held. The fuzz targets phase 1 added are in `fuzz/`. Phase 6 is next.

**Re-checked on 29 October 2026** against `main` at `85c4c85c`, two hundred commits on (the `ascribe-workspace`, `ascribe-edit`, and `ascribe-report` crates, the git guard): all five private findings hold, one wider than first written; of the hardening notes one is fixed (phase 1's H4) and one improved (phase 2's H6), the rest hold; each notes file ends with what the two hundred commits added to its surface, looked at, with two new notes (phase 1's H5 and H6).

## No thread

A review, not a feature: no layer is built, so the steel-threads ordering doesn't apply.

## Rules for every phase

- Read the threat model (phase 0) and the decisions above first.
- Read the code, not only its comments: a comment that says a path is checked is a claim to verify.
- Each candidate finding gets the input that triggers it, tried, before it's written down; a second agent tries to refute it. What can't be triggered is a hardening note, or nothing.
- Exploitable findings go to a draft security advisory (decision 1). Hardening notes go in the phase's notes file, `project-docs/security-review/notes/phase-<n>.md`, each with where it is, what it costs, one recommendation, and a size.
- Don't fix anything in a review phase. Fixes follow phase 6, each its own pull request with its test.

## Later, not in this plan

- A review by someone outside the project, once there are users to protect.
- Signing the release binaries, if a platform or a user asks for it beyond npm's provenance.
