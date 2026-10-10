# Phase 3: The journeys

Part of [The surface audit](README.md). Requires phase 1, for the script's form. Mostly the maintainer's time, and agents' sessions.

## Goal

Notes from the other seven tasks, each walked by everyone who'd do it, in the form phase 1 settled.

A list of the surface shows what exists. A walk shows what someone can find, in what order, and where two parts that each make sense alone don't meet.

## Context

- Phase 1's script, its notes, and the note on what to change about the method.
- README decisions 5 and 6: a walk is done as written and noted as it went, and an agent's is a fresh session with its transcript kept.
- The guides a new user would start from: `docs/content/getting-started.md`, and those under `docs/content/guides/`.
- `examples/quill`, `examples/monorepo`, and `examples/docs-repository`: starting points, each in a fresh copy.

## The journeys

Each has a goal a user would recognize, a starting point, and who walks it. The maintainer's are marked.

| | Task | Starts from | Walked |
|---|---|---|---|
| 1 | Start a project and publish a first page | An empty folder, and the getting-started page | The maintainer, as an author. The maintainer, as a site builder with an Astro site already |
| 2 | Add a section that differs by platform | `examples/quill` | The maintainer, in the editor. An agent |
| 3 | Show an example from the code, and keep it current | `examples/quill`, with a code file to draw from | The maintainer. An agent |
| 4 | Rename a phrase everywhere | `examples/quill` | The maintainer, in the editor. An agent |
| 5 | Review a pull request's docs | A branch with changes, as the review fixture makes one | The maintainer: in the editor, in the site preview, and from the report |
| 6 | Set up checks in CI | A project with no workflow | The maintainer |
| 7 | Arrive in an unfamiliar project and make a correct edit | `examples/monorepo`, and a one-line request | An agent with the plugin. An agent with nothing but what `ascribe agents sync` wrote. An agent with nothing |

That's ten walks for the maintainer and seven sessions for agents.

Notes on three of them:

- **Journey 1 is the one to do first and coldest.** It can only be walked as a newcomer once. Follow the published getting-started page exactly, on a machine or in a folder with nothing set up, and note every place the page and reality differ.
- **Journey 5** uses the review fixture (`scripts/review-fixture/`), or the demo pull request if it's still open, so there are real threads to read and answer.
- **Journey 7's third walk** is the baseline: what an agent does with no help shows what the help is worth, and what an agent guesses Ascribe's commands are called.

## How a walk is noted

As phase 1 settled it. At least: each step with what was typed or clicked and what came back, quoted; each moment of doubt, with what was expected; the time taken; and, for an agent, what it tried before what worked.

A walk that can't be finished is noted to where it stopped, with why. That's a result.

## Tasks

1. A script for each journey, in `journeys/`.
2. The maintainer's ten walks. They don't have to be done together, or in order after the first.
3. The agents' seven sessions, with transcripts in `journeys/transcripts/`.
4. For each journey, a short list under its notes of what the walks had in common and where they differed. No findings yet beyond what was noted as it happened: phase 5 writes those.

## Out of scope

Fixing what a walk turns up; walking a task twice to see if it goes better; journeys for surfaces nobody has asked about.

## Acceptance criteria

- Every walk in the table has notes, or a line saying why it wasn't done.
- Every note of doubt quotes what was on the screen.
- Each agent's transcript is kept whole.

## Verify

```sh
pnpm test
```

## Commits

One for the scripts, then one for each journey's notes as it's finished.
