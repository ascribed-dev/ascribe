# Phase 6: Following their releases

Part of [Page templates](README.md). Requires phase 3. A workflow, the script, and one command.

## Goal

Two kinds of following:

- **Ascribe follows them.** A new release of The Good Docs Project arrives as a pull request with the regenerated set, for a person to read before it ships in Ascribe's.
- **A project can look.** `ascribe templates status` says what's changed in Ascribe's set since the project made its copy, for a team that wants to know.

Neither changes a project's files. Decision 6 holds: a team's templates are its own.

## Context

- The proposal: "The conversion", its last paragraph, and "The starter set gets in by being copied".
- `examples/docs-repository/.github/workflows/update-sources.yml`: a scheduled job that follows another repository and opens a pull request.
- This repository's workflows, and `project-docs/outside.md`: what each job may do, with which token, and who can change it. A job that opens a pull request is listed there.
- Their repository: how a release is tagged, and `index.json`'s date for each template.
- The pinned release's file, the script, and the rule file.
- `ascribe sources status`, in the command reference: the nearest command that says how a copy stands against where it came from.

## Design

### The job

Scheduled, weekly. It reads their repository's tags, at `gitlab.com/tgdp/templates` (README decision 25): GitLab's tags answer without signing in, and its releases answer 403. When the newest release tag is newer than the pin:

1. it moves the pin, runs the script, and runs the tests;
2. it opens a pull request with the new copies, or updates the one that's open;
3. the description says what a reader needs: which of the six changed, from the diff between the two tags, with a link to that comparison on GitLab. Not from the index's `last_updated`, which doesn't follow releases: the how-to's is 15 December 2022.

- **A rule that no longer finds its place stops the script,** and the job opens the pull request anyway, with the failure as its description and nothing regenerated. That's decision 3 doing its work: a person decides what the rule should now say.
- **A new template in their Core pack** isn't added. The description names it.
- **Nothing merges itself.** Their words ship in Ascribe's binary, so a person reads the diff.

### What a project's copy remembers

`templates/templates.lock`, which `ascribe templates eject` has written since phase 2 (README decision 23): each file, their tag, Ascribe's version, a hash of the file as written, and the type's summary. This phase reads it.

The hash is what lets `status` tell "you changed this" from "this changed upstream", which are different things to be told. A project whose lock is gone, or that has a template with no entry, is told so plainly, and `status` says what it can from the files alone.

### `ascribe templates status`

For each template a project took from the set:

- unchanged upstream since the copy, or changed, with how (their release, and a short list of what's different by section);
- whether the project has changed its copy;
- a template in the set the project doesn't have.

It reads the binary's set and the project's files, and nothing else: no network. So it says what's changed as of this `ascribe`, and says which release that is.

- `--format json` for tools, with a schema.
- It exits 0 whatever it finds, and 1 with `--exit-code` when something upstream changed, by the checklist's rule for a command that reports.
- It shows a difference and applies none. A team that wants the new text copies it. A `--diff` that prints the upstream change to one template is enough to do that by hand.

## Tasks

1. The job, with its permissions and its entry in `project-docs/outside.md`. Tried once against a pin deliberately set back a release, with the pull request it opened linked.
2. The failure path, tried by breaking a rule on purpose.
3. Reading `templates.lock`, with what `status` says when it's missing, when a file has no entry, and when an entry's file is gone.
4. `ascribe templates status`, with tests: unchanged; changed upstream; changed by the project; both; a file deleted; a copy with nothing remembered.
5. The command reference, the page on templates, `RELEASING.md` if a release should check the pin, `CHANGELOG.md`, and the map of the surface.

## Out of scope

Updating a project's templates for it. Merging. Following any repository but theirs. Telling a project in `ascribe check` that its templates are behind: it isn't a problem with the project.

## Acceptance criteria

- With the pin set back, the job opens a pull request whose diff is the set regenerated and whose description names what changed.
- With a rule broken, it opens one that says which rule and where, and changes no template.
- `ascribe templates status` in a project whose copies are untouched, against a binary with a newer set, lists exactly the templates that differ, and exits 0.
- It makes no request: the test runs it with the network blocked.

## Verify

```sh
cargo test --workspace --locked
pnpm test
```

## Commits

1. "Open a pull request when The Good Docs Project releases"
2. "Say what's changed in the templates since a project's copy"
