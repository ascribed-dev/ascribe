# Phase 4: Crawlers by purpose

Part of [Search and answers](README.md). Requires phase 2. Rust, a data file, and `@ascribed/astro`'s `robots.txt`.

## Goal

A project says what each kind of bot may do, and `robots.txt` has the rules for every bot Ascribe knows of that kind:

```toml
[crawlers]
training = "disallow"
search = "allow"
on-request = "allow"
```

```text
User-agent: GPTBot
User-agent: ClaudeBot
User-agent: Google-Extended
Disallow: /

Sitemap: https://docs.example.com/sitemap.xml
```

A project that says nothing gets no such rules.

## Context

- README decisions 6, 7, and 8, and the proposal's "Managing how crawlers treat pages".
- The research's notes on crawlers, in `project-docs/research_notes/SEO and AEO for technical docs/`: which operators run which bots, for which purpose, and what each has said about honoring `robots.txt`. Check each against the operator's own page before writing it down; the notes are from October 2026 and this changes.
- `[crawlers]` and `lines`, as phase 2 built them, and phase 2's check.
- How Ascribe ships data that changes between releases and is read by the binary: the Vale preset is the nearest, and `tests/conformance/diagnostics.toml` the best-known.
- RFC 9309, for how a `robots.txt` group is read.

## Design

### The purposes

Three, each `allow`, `disallow`, or absent:

- **`training`:** gathering text to train a model.
- **`search`:** building the index an AI search product answers from.
- **`on-request`:** fetching a page because a person asked an assistant about it.

Absent means nothing is written for that purpose. That's different from `allow`, which writes a group that allows, and matters when a project's own `lines` disallow everything.

### The tokens are data

One file in the repository lists each token Ascribe knows, with its operator, its purpose, the address of the operator's page about it, and the date that page was read. The binary carries it. A test fails on a token with no source or a date over a year old, so the list is reread rather than trusted.

A token whose purpose an operator doesn't state isn't in the list. A project names it in `lines`.

### What's written

- Groups for the purposes a project chose, then its own `lines`, then the sitemap. A project's own lines come last and win where they're more specific, as `robots.txt` reads.
- The same token is never in two groups.
- With several projects in one site, the purposes must agree across them, since there's one file. Two that differ are an error that names both.

### What's said

In the reference, at each purpose, in a sentence each: what disallowing it costs (a site blocked from a search bot isn't in that product's answers), which operators say their bot of that kind honors `robots.txt` and which say it may not, and that a host's or a CDN's own bot settings can override the file. Each with the date it was true.

### The check, widened

Phase 2's check read the group for `*`. It now reads every group that applies to a search engine's crawler, so a page that isn't listed and is disallowed for the crawler that has to read the rule is reported, whichever line does it.

A second, advice: a project that disallows `search` or `on-request` for a site whose pages are otherwise listed. It isn't wrong. It's the choice with a cost people don't expect, and the message says what the cost is. It's acknowledged like any other.

## Tasks

1. The data file, its test, and how the binary carries it.
2. The three keys in `[crawlers]`, with validation and conformance cases.
3. The groups in the record's `robots`, and `robots.txt` from them, with tests: each purpose alone, all three, none, and with `lines`.
4. Several projects: agreeing, and the error.
5. The checks, in the registry, with conformance cases.
6. The reference, with each operator's statement dated and linked; the Astro guide; `CHANGELOG.md`; the map of the surface; the decision.
7. `RELEASING.md`: a step to reread the tokens' sources.

## Out of scope

Rules by path for a purpose. A crawl delay. Content signals and any "no AI" tag or file: no operator has committed to one. Blocking by a host's or a CDN's settings, which aren't Ascribe's.

## Acceptance criteria

- A project with no `[crawlers]` gets a `robots.txt` with the sitemap's address and nothing else.
- `training = "disallow"` writes one group naming every training token in the data, and no other.
- Every token in the data has a source that answers, checked once and said in the pull request.
- A page that isn't listed, disallowed by a purpose's group for a search crawler, is reported.

## Verify

```sh
cargo test --workspace --locked
pnpm --filter @ascribed/astro test
pnpm --filter @ascribed/example-astro-site test:e2e
```

## Commits

1. "Ship the list of crawlers Ascribe knows, by purpose"
2. "Let a project say what each kind of crawler may do"
3. "Say what disallowing a search crawler costs"
