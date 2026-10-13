# Phase 1: Decisions

Part of [Tested examples](README.md). Needs nothing. No code: a pull request that records what the maintainer decides.

## Goal

Five choices are made before anything ships, because each goes into a contract: a file format, an attribute both renderers read, the JSON output, three diagnostics, and what a project sees the day it adds a config.

This phase can't be folded into phase 2. A thread may narrow what it handles. It may not ship a shape the plan expects to change.

## Context

- The [proposal](../proposals/tested-examples.md), whole.
- `crates/ascribe-model/src/lock.rs`: `ascribe.lock` records "the hash of its bytes".
- SPEC §4.8, and `packages/astro/src/code-titles.ts`: how a snippet's title reaches a renderer.
- `packages/elements/CONTRACT.md`: how the availability element's lead-in, "Available:", comes from a stylesheet and not from the markup.
- `crates/ascribe-emit/src/json.rs`: what a block's `snippet` holds.
- `docs/content/checks/diagnostics.md`, "What to do next": the five kinds of next step.
- `docs/ascribe.toml`: the docs' own source, `code`, is the repository's root, and its snippets show configuration, test output, and source files.

## The decisions

Each had a recommendation, and the maintainer took all five, on issue #458. Each answer is under its heading as **Decided:**.

### 1. How a file is hashed

The observation is written where the code is checked out, and compared where the docs are built. A checkout on Windows may have rewritten a text file's line endings, and `ascribe.lock` hashes bytes as they are on disk.

| Option | Means |
|---|---|
| **The bytes, with CRLF read as LF** (recommended) | A file hashes the same on a Windows checkout and a Linux one. A file that really changed only in its line endings reads as unchanged, which for a test's purposes it is. One rule, stated in the observation's format. A binary file with a CR LF pair in it hashes the same either way too, consistently, which is all a comparison needs |
| The bytes as they are | The same as `ascribe.lock`. An observation made on Windows is stale on every other machine unless the repository forbids the rewrite, as Ascribe's own `.gitattributes` does |
| What git stores for the file | Exact, and what a team means. `ascribe observe` would need a repository and `git`, and an example that isn't committed yet couldn't be observed |

The format names its hash, `sha256-lf:`, so it can't be mistaken for the lock's `sha256:` and a later format can add another.

**Decided:** the bytes, with CRLF read as LF, named `sha256-lf:` (README decision 25, decision [#458a](../decisions/458a-example-hash.md)). A file then hashes the same on a Windows checkout and a Linux one, which a comparison made on another machine needs, and the rule is one line in the observation's format. Hashing the bytes as they are would make every observation made on Windows stale elsewhere, and asking git would need a repository and couldn't observe an example not yet committed. _From:_ Kyle, on issue #458, 13 October 2026.

### 2. What carries the tested line

A snippet's title is `title="…"` in the code block's info string. The tested line travels the same way (README decision 13). What's in the attribute decides what a site can restyle or translate.

| Option | Looks like | For and against |
|---|---|---|
| **The environment, without the words around it** (recommended) | `tested="Node 24.9.0 and TypeScript 5.9.2 on Linux"` | The caption reads "Tested with …", and "Tested with" comes from the stylesheet, as "Available:" does, so a site translates it once. With no stylesheet, the caption still reads as a fact about the block |
| The finished sentence | `tested="Tested with Node 24.9.0 and TypeScript 5.9.2 on Linux."` | Nothing for a renderer to add. English in the markup of every site |
| One attribute for each fact | `tested-node="24.9.0" tested-os="linux"` | A site can lay it out any way. Names a project chose become attribute names, and both renderers have to assemble a sentence |

A block with a tested line and no title still gets a caption. A block with both has the title, then the tested line. The plain output writes the full sentence in English, on a line under the block, since nothing styles it.

**Decided:** the environment, without the words around it: `tested="Node 24.9.0 and TypeScript 5.9.2 on Linux"`, with "Tested with" from the stylesheet (README decision 26, decision [#458b](../decisions/458b-tested-attribute.md)). A site translates "Tested with" once, as it does "Available:", and with no stylesheet the caption still reads as a fact about the block. The finished sentence would put English in every site's markup, and an attribute for each fact would turn names a project chose into attribute names and leave both renderers assembling a sentence. _From:_ Kyle, on issue #458, 13 October 2026.

### 3. What the JSON output says

On a block's `snippet`, beside its address, path, and lines:

```json
"tested": {
  "status": "current",
  "observations": [
    { "name": "default", "environment": { "os": "linux", "arch": "x64", "node": "v24.9.0" } }
  ]
}
```

| Question | Recommended |
|---|---|
| The key | `tested`, absent when the file isn't an example (decision 5) |
| `status` | `current`, `stale`, or `unobserved` |
| `observations` | The current ones only, each with its name and its environment as the observation has it: the kept first line, not the shown number, so a reader of the JSON parses versions its own way |
| Labels | Not here. They're in the config, and a later key can carry them |

**Decided:** as recommended (README decision 27, decision [#458c](../decisions/458c-tested-json.md)): `tested` on a block's `snippet`, absent when the file isn't an example; `status` is `current`, `stale`, or `unobserved`; `observations` lists the current ones only, each with its name and its environment as the observation has it; no labels. The kept line, not the shown number, lets a reader of the JSON parse versions its own way, and labels are already in the config, where a later key can carry them. _From:_ Kyle, on issue #458, 13 October 2026.

### 4. The three diagnostics

| What | Recommended name | Next step |
|---|---|---|
| The file changed after its last test run | `example-stale` | `outside` |
| The file is an example and no observation lists it | `example-unobserved` | `outside` |
| An observation lists a file that isn't there | `observation-file-missing` | `outside` |

All three are advice, in a new area, `examples`. The first two are reported at the `@snippet` line, the third at the observation.

**The next step is `outside` for all three,** wherever the code is. The proposal has it differ: a command to run for code in the same repository, `outside` for code in another. But the five kinds are about what the source can do, and in both cases nothing in a page fixes it: someone runs the tests. The message says which: `ascribe observe` in the named folder, or the next `ascribe sources update`. A sixth kind for "run a command" would be one more thing for every reader of the report to learn, for one check.

**Decided:** `example-stale`, `example-unobserved`, and `observation-file-missing`, advice in a new area, `examples`, each with the next step `outside` (README decision 28, decision [#458d](../decisions/458d-example-diagnostics.md)). The first two are reported at the `@snippet` line, the third at the observation. The five kinds say what the source can do, and in neither case does anything in a page fix it: someone runs the tests. The message names which, `ascribe observe` in the named folder or the next `ascribe sources update`, so no sixth kind is needed for one check. _From:_ Kyle, on issue #458, 13 October 2026.

### 5. What a snippet of a file that isn't an example reports

Ascribe's own docs show configuration files, test output, and source code through one source. Most of those aren't examples, and none has ever been observed.

| Option | Means |
|---|---|
| **Nothing, unless the source has a config and the file matches its `include`** (recommended) | A source with no `ascribe.examples.toml` reports nothing and shows no tested line, so a project that never adopts this sees no change. With a config, a file `include` matches is an example: tested, stale, or unobserved. A file it doesn't match is just a file a page shows |
| Every snippet is an example | Simple. Every project gets advice on every snippet the day it upgrades |
| A page says which snippets are examples | `@snippet {tested=true}`. Exact, and the code's owners no longer decide what counts |

**Decided:** nothing, unless the source has an `ascribe.examples.toml` and its `include` matches the file (README decision 29, decision [#458e](../decisions/458e-what-is-an-example.md)). A project that never adopts tested examples sees no change, and the code's owners, who write the config, decide what counts as an example. Treating every snippet as one would give every project advice on every snippet the day it upgrades, and marking them in the page would move that choice away from the code. _From:_ Kyle, on issue #458, 13 October 2026.

## Tasks

1. The maintainer reads the five and answers each.
2. Record each answer in this file, under its heading, as **Decided:** with the reason, and in the README's decisions, numbered from 25.
3. Record them in `project-docs/decisions/`, as the repository does.
4. Add to the proposal what the answers change in it: the next step, and the proposal's sentence that Ascribe runs `git` and nothing else.

## Out of scope

- The tap's name and the install script's address. Phase 6 opens with those.
- Which repositories phase 7 makes, beyond the two the proposal names.

## Acceptance criteria

- Each of the five has an answer and a reason.
- Nothing in a later phase file contradicts one.
- Phase 2 can be started without asking the maintainer anything.

## Commits

1. "Record the decisions the tested examples plan ships"
