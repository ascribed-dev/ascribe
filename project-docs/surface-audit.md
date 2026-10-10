# Proposal: an audit of Ascribe's surface

A proposal, not a plan. It says what the audit would look at, how, and what it would hand back, so the idea can be judged before the work starts. The plan for doing it is in [surface-audit/](surface-audit/README.md).

## The goal

Find where Ascribe's surface has stopped being one thing: where two parts name the same idea differently, do the same job twice, or leave a gap between them. Then say what to change.

A lot was added quickly. The agents work, the editor UI, review, and drift each landed as a plan with its own decisions, and each made sense alone. Nobody has yet looked at all of it at once, as the person or the agent meeting it does.

It's worth doing now for two reasons:

- **Four proposals are about to add more:** permalinks, content checks, tested examples, and versions each bring commands, configuration, and files. They should land on a surface with settled conventions.
- **Nothing has to be kept.** Ascribe is before 1.0 and has no users, so a rename or a removal costs only the work. That won't stay true.

## What "the surface" is

Everything a person or an agent touches. A count taken from `main` on 10 October 2026, to show the size, not to be the inventory:

| Surface | For | What's there |
|---|---|---|
| The command line | Both | 15 commands, with 8 more under `agents` and `sources`; seven output formats between them (`text`, `json`, `concise`, `prompt`, `summary`, `html`, and each command's own text) |
| The language | Authors | The directives, attributes, phrases, and frontmatter keys of `SPEC.md` |
| `ascribe.toml` | Authors | 20 sections |
| Diagnostics | Both | 141, each with a message and a fix |
| The VS Code extension | People | 63 commands, 35 of them actions; 9 settings; 4 sidebar views; a status bar item; the actions bar and its key; the preview, review, and the build lens; a walkthrough of 6 steps |
| The language server | Editors, the extension | The standard requests, and 9 of Ascribe's own (`ascribe/context`, `ascribe/edit`, `ascribe/preview`, and so on) |
| The MCP server | Agents | 9 tools, with resources and prompts |
| VS Code's agent tools | Agents in VS Code | 3 language model tools, and the MCP server registered for Copilot |
| The plugin | Agents | A skill, 3 commands, a hook, the language server, the MCP server |
| What `ascribe agents sync` writes | Agents | Rules in `AGENTS.md`, and the skill's folder |
| `@ascribed/astro` | Site builders | The integration's options, the dev toolbar's review app, what it writes for the editor |
| `@ascribed/elements`, `@ascribed/review` | Site builders | Custom elements and their CSS properties; the review package's API |
| The outputs | Site builders, tools | The site, plain, and JSON outputs; the commands' JSON; the HTML report |
| Files Ascribe puts in a project | Both | `ascribe.lock`, `sources/`, `.ascribe/`, a block in `AGENTS.md`, the skill folder |
| The docs | Both | The guides and references on the site, each command's `--help`, the READMEs |

Three kinds of user meet this, and the audit takes each one's side in turn: an author writing docs, an agent writing docs for one, and someone building the site that publishes them.

## What it looks for

Seven questions, each asked across every surface.

1. **One word for one thing.** Is a build always a build? Are "problems", "diagnostics", and "issues" three things or one? Does a command's name match the tool, the action, and the docs heading for the same job?
2. **The same thing, the same way.** Do commands agree on `--format`, on `--build`, on how a page is named, on exit codes, on what `json` contains? Do the extension's commands follow one pattern for titles?
3. **What can be done where.** A table of every capability against every surface. A gap is either deliberate (and then said somewhere) or an oversight. So is a capability offered three ways.
4. **Whether each thing can be found.** Starting from nothing, how does someone learn that `ascribe refs` exists, or the build lens, or the hook? What does `--help` lead to, what does the palette show, what does an agent try first?
5. **Where a choice is made.** A setting can live in `ascribe.toml`, in VS Code's settings, in the integration's options, or in a flag. Is each in the right one, and is any in two?
6. **What a problem says to do next.** Across the editor, the command line, and what an agent reads, does the same problem lead to the same next step?
7. **What's left over.** Commands, settings, options, and files that an earlier design needed and the current one doesn't.

## How

### 1. An inventory, generated

Most of the surface is already declared somewhere a script can read: the commands in the binary, the extension's manifest, the diagnostics registry, the MCP server's tool list, the content model's reference. The inventory is built from those, not typed.

That makes it repeatable, and it's useful after the audit: a generated map of the surface, kept current by a test, is how the next plan sees what it's adding to.

### 2. Journeys

A list is where inconsistencies hide; a task is where they show. So the audit walks a small set of real tasks end to end, once as each kind of user who would do it, and writes down every surface touched and every place the path was unclear:

| Task | As |
|---|---|
| Start a project and publish a first page | An author; a site builder |
| Add a section that differs by platform | An author in the editor; an agent |
| Fix a broken link | An author in the editor; an author at the command line; an agent through the hook; an agent through MCP |
| Show an example from the code, and keep it current | An author; an agent |
| Rename a phrase everywhere | An author; an agent |
| Review a pull request's docs | A reviewer in the editor; in the site preview; from the CI report |
| Set up checks in CI | An author |
| Arrive in an unfamiliar Ascribe project and make a correct edit | An agent, in a fresh session |

The agent's journeys are run with real agents in fresh sessions, with what they tried recorded: which command they guessed first, what they read, where they went wrong. The agents plan's last phase did a pass like this and has a method to reuse.

### 3. The seven questions, against the inventory

With the inventory and the journeys in hand, go through the questions one at a time. The capability table in question 3 is the largest single piece of work, and the most likely to find something.

### 4. Findings

Each finding says what it is, where, and what it costs someone, and is one of five kinds:

- **A contradiction:** two parts disagree.
- **A duplicate:** two parts do one job.
- **A gap:** something is missing where a neighbor has it.
- **A naming problem.**
- **Friction:** it works, and takes more than it should.

Each comes with one recommendation (rename, merge, remove, add, document, or leave as it is, with the reason) and a rough size.

## What it hands back

- **The generated inventory,** and the test that keeps it current.
- **The capability table.**
- **A report of findings,** in order of what they cost a user, each with its recommendation.
- **A short set of conventions** for whatever is added next: how a command group is named, what `--format` values mean, where a kind of setting lives, how an action's title is worded. These go where plans will read them, in `project-docs/checklists.md`.
- **Issues** for the changes worth making, grouped so that related renames happen together.

It doesn't hand back fixes. The audit finds and recommends; each change is its own work, decided on its own.

## What it leaves out

- How Ascribe looks. The visual design work covered that.
- How fast it is, and how the code is arranged.
- Whether a feature should exist. The audit asks whether what exists fits together, not whether it was a good idea, except where something is plainly left over.
- The language's design. Its directives are in scope for naming and for consistency with everything else, not for redesign.

## Settled

- **It's done once.** The generated inventory makes a later one cheap, if it's ever wanted.
- **The maintainer walks the people's journeys.** Nobody else is set up to test yet.
- **The docs site is in scope,** as the way people find the surface and as the first site built with Ascribe.
- **The command line's conventions come first,** from the plan's first phase, so the next plan to add a command group doesn't wait for the whole audit.
