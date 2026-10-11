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

User-agent: *
Disallow: /docs/drafts/

Sitemap: https://docs.example.com/sitemap.xml
```

A project that says nothing gets no such rules.

## Context

- README decisions 6, 7, and 8, and the proposal's "Managing how crawlers treat pages".
- The research's notes on crawlers, in `project-docs/research_notes/SEO and AEO for technical docs/`: which operators run which bots, for which purpose, and what each has said about honoring `robots.txt`. Check each against the operator's own page before writing it down; the notes are from October 2026 and this changes.
- `[crawlers]` and `lines`, as phase 2 built them, and phase 2's check.
- How Ascribe ships data that changes between releases and is read by the binary: the Vale preset is the nearest, and `tests/conformance/diagnostics.toml` the best-known.
- RFC 9309, for how a `robots.txt` group is read. The rule this phase turns on (README decision 29): a crawler obeys only the most specific group that names it, and the `*` group doesn't apply to it at all.

## Design

### The purposes

Three, each `allow`, `disallow`, or absent:

- **`training`:** gathering text to train a model.
- **`search`:** building the index an AI search product answers from.
- **`on-request`:** fetching a page because a person asked an assistant about it.

**`disallow` writes one group** naming every token of that purpose, with `Disallow: /`. Nothing else has to be in it: a crawler told to fetch nothing has no use for a project's paths.

**`allow` writes nothing,** and neither does saying nothing. The reason is how a group is read. A group that named those crawlers and said `Allow: /` would take them out of the `*` group, and with it out of every path the project's own `lines` disallow: "allowed" would quietly mean "allowed everywhere, drafts included". Leaving them unnamed keeps them under the `*` group, where the project's paths apply to them as to everyone. So `allow` is a project writing its choice down, and the reference says in a sentence why it adds no lines.

A project that wants a kind of crawler to see less than everyone else, or more, writes that crawler's group itself, whole, in `lines`.

### The tokens are data

One file in the repository lists each token Ascribe knows, with its operator, its purpose, the address of the operator's page about it, and the date that page was read. The binary carries it.

A test fails on a token with no source or no date: what a commit can break. How old a date is isn't a test's to judge, since a test that fails on an anniversary turns `main` red with no change. It's a line in the weekly Report workflow instead, which already opens an issue for what it finds: a token whose source was read over a year ago, or no longer answers.

A token whose purpose an operator doesn't state isn't in the list. A project names it in `lines`.

### What's written

- A group for each purpose a project disallowed, then the `User-agent: *` group with its lines for every crawler, then any groups it wrote whole, then the sitemap.
- **A token is in one group.** A project's own group that names a token one of its purposes already covers is an error at the line: two groups for one crawler read as one, and the result is rarely what either was written to say.
- **A project's lines never reach a crawler it disallowed by purpose,** and don't need to. They don't reach a crawler it named in a group of its own either, which is why that group has to be whole, and the reference says so where `lines` is described.
- In the record, `robots` gains the groups beside `lines`, and changes nothing phase 2 wrote.
- With several projects in one site, the purposes must agree across them, since there's one file. Two that differ are an error that names both.

### What's said

In the reference, at each purpose, in a sentence each: what disallowing it costs (a site blocked from a search bot isn't in that product's answers), which operators say their bot of that kind honors `robots.txt` and which say it may not, and that a host's or a CDN's own bot settings can override the file. Each with the date it was true.

### The check, widened

Phase 2's check read the group for `*`. It now reads, for each search engine's crawler, the one group that applies to it: its own if a purpose or the project named it, and `*` otherwise. So a page that isn't listed and is disallowed for the crawler that has to read the rule is reported, whichever group does it.

A second, advice: a project that disallows `search` or `on-request` for a site whose pages are otherwise listed. It isn't wrong. It's the choice with a cost people don't expect, and the message says what the cost is. It's acknowledged like any other.

## Tasks

1. The data file, its test, how the binary carries it, and the Report workflow's line for a source that's old or gone.
2. The three keys in `[crawlers]`, with validation and conformance cases.
3. The groups in the record's `robots`, and `robots.txt` from them, with tests: each purpose disallowed alone; all three; none; `allow`, which writes nothing; with `lines` for every crawler; with a group the project wrote; and a token named twice, which is the error. One test reads the written file as a crawler would, for a token of each purpose, and checks which paths it may fetch.
4. Several projects: agreeing, and the error.
5. The checks, in the registry, with conformance cases.
6. The reference, with each operator's statement dated and linked; the Astro guide; `CHANGELOG.md`; the map of the surface; the decision.
7. `RELEASING.md`: a step to reread the tokens' sources.

## Out of scope

Rules by path for a purpose. A crawl delay. Content signals and any "no AI" tag or file: no operator has committed to one. Blocking by a host's or a CDN's settings, which aren't Ascribe's.

## Acceptance criteria

- A project with no `[crawlers]` gets a `robots.txt` with the sitemap's address and nothing else.
- `training = "disallow"` writes one group naming every training token in the data, and no other.
- With `search = "allow"` and `lines = ["Disallow: /drafts/"]`, a search crawler read against the file may not fetch `/drafts/`. That's the case a group saying `Allow: /` would get wrong.
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
