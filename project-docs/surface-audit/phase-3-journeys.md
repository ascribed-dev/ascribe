# Phase 3: The agents' journeys

Part of [The surface audit](README.md). Requires phase 1, for the script's form. Agents' sessions, prepared, run, and recorded by an agent; no code, and none of the maintainer's time.

## Goal

A transcript and notes from an agent doing each of the other seven tasks in a fresh session, with what it tried first, so the findings about what an agent can find and do rest on recorded runs.

The maintainer's walks are gone from this phase (README decision 9). Phase 1's two showed that the maintainer's surface is the editor with an agent in it, and that timing a person isn't what this audit is for. The agents' sessions stay because they're what's automatable: each runs unattended and records itself.

## Context

- Phase 1's script, [`journeys/fix-a-broken-link.md`](journeys/fix-a-broken-link.md): the form a script and its notes take, how a copy is prepared, and how a session is run and recorded (`claude -p` with a clean profile and `--output-format stream-json`).
- README decisions 5, 6, and 9: a session is run as written and noted as it went; it's a fresh session with nothing but the project and what Ascribe put in it, and its whole transcript is kept.
- The guides a newcomer starts from: `docs/content/getting-started.md`, and those under `docs/content/guides/`.
- `examples/quill`, `examples/monorepo`, and `examples/docs-repository`: starting points, each in a fresh copy.
- The agents plan's pass ([`pass-results.md`](https://github.com/ascribed-dev/ascribe/blob/4dff94ea9d05abbd77e2bf553560aabacf659cc3/project-docs/agents/pass-results.md) at its last commit): seven tasks run with `claude -p`, and what they found. These sessions aren't a repeat of it: they're the same kind of run, on the audit's tasks, kept whole.

## The sessions

Each has a goal a user would recognize, a starting point, and the setups to run it in.

| | Task | Starts from | Sessions |
|---|---|---|---|
| 1 | Start a project and publish a first page | An empty folder, and the getting-started page's address | An agent, with nothing else |
| 2 | Add a section that differs by platform | `examples/quill` | An agent with the plugin |
| 3 | Show an example from the code, and keep it current | `examples/quill`, with a code file to draw from | An agent with the plugin |
| 4 | Rename a phrase everywhere | `examples/quill` | An agent with the plugin |
| 5 | Review a pull request's docs | A branch with changes, as the review fixture makes one | An agent, from the prompt `ascribe diff --format prompt` writes |
| 6 | Set up checks in CI | A project with no workflow | An agent with the plugin |
| 7 | Arrive in an unfamiliar project and make a correct edit | `examples/monorepo`, and a one-line request | Three: with the plugin; with only what `ascribe agents sync` wrote; with nothing |

Nine sessions, after phase 1's two. Each is a fresh session with a clean profile, in a copy that holds the project and what its row adds, with the task's sentence as the whole prompt, recorded whole to `journeys/transcripts/<journey>-<setup>.jsonl`. When it stops, the preparer runs `ascribe check` in the copy and notes the exit code and the diff.

Notes on three of them:

- **Journey 7's third session is the baseline.** What an agent does with no help shows what the help is worth, and what an agent guesses Ascribe's commands are called.
- **Journey 5** uses the review fixture (`scripts/review-fixture/`), so there are real changes to review; the agent gets the prompt the report gives a reviewer, and nothing else.
- **Journey 1 is the coldest.** The agent is given the getting-started page's address and an empty folder, and every place the page and what happens differ is a note.

## How a session is noted

As phase 1 settled it: the parts under "Notes" in [`journeys/fix-a-broken-link.md`](journeys/fix-a-broken-link.md). The steps are a numbered list, one line per step with what the agent ran or asked and what came back, quoted from the transcript; then each place it went wrong or hesitated, with what it tried before what worked; the time taken; what the hook or the tools said; and the transcript's path.

A session that doesn't finish is noted to where it stopped, with why. That's a result.

## Tasks

1. A script for each journey, in `journeys/`, in phase 1's form: the task's sentence, the setups, the notes' parts, and the preparer's section.
2. The nine sessions, recorded.
3. For each journey, a short list under its notes of what its sessions had in common and where they differed. No findings yet beyond what was noted as it happened: phase 5 writes those.

## Out of scope

Walks by the maintainer; fixing what a session turns up; running a session twice to see if it goes better; journeys for surfaces nobody has asked about.

## Acceptance criteria

- Every session in the table has notes and a transcript, or a line saying why it wasn't run.
- Every note of doubt quotes the transcript.
- Each transcript is kept whole.

## Verify

```sh
pnpm test
```

## Commits

One for the scripts, then one for each journey's notes and transcripts as it's finished.
