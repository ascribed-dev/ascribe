# What can be done where

Part of [The surface audit](README.md), [phase 2](phase-2-inventory.md). Every thing a user wants to do, in a user's words, against the surfaces someone does it on. Written by hand from the generated [map of the surface](../surface.md), so each name in a cell is a row of the map.

It's the audit's answer to question 3. It doesn't judge: a gap is listed, and what to do about it is a finding, in [findings.md](findings.md). [Phase 5](phase-5-findings.md) finished it against phase 3's sessions: every cell that was open is settled, and five cells the sessions showed to be less than they said are corrected, each with the session it's from.

## How to read it

The rows come from the proposal's eight journeys, the tasks the guides teach, and what the agents plan asked agents to do. The columns are the five places someone acts:

- **Command line:** a person or an agent with a shell, and CI.
- **VS Code:** a person, with the extension.
- **MCP tools:** an agent with no shell, or one whose host approves tools one at a time. The plugin and the extension both offer this server, so the column is theirs too.
- **In VS Code, for an agent:** the three tools only the editor has.
- **Site preview:** `astro dev`, with `@ascribed/astro`.

A cell holds what does the job there, or one of these:

| Mark | Means |
|---|---|
| by hand | It's written into the page or `ascribe.toml`, and nothing on this surface helps |
| no (D1) | It isn't there, on purpose. The number is a reason in [Why not](#why-not) |
| **gap** | It isn't there, and nothing found says that's on purpose |

## Starting

| A user wants to | Command line | VS Code | MCP tools | In VS Code, for an agent | Site preview |
|---|---|---|---|---|---|
| Start a project | **gap**: `ascribe.toml` is written by hand, from getting started | **gap**: the walkthrough's "Open a project" opens one that exists | no (D1) | no (D2) | no (D3) |
| Start a new page of a type | `ascribe agents prompt new-page`, which writes a prompt for an agent. For a person, **gap** | **gap** | The prompt `new-page` | no (D2) | no (D3) |
| See what the project allows: its types, phrases, features, builds | `ascribe model`. Not its sources: `ascribe model --section sources` is `error: invalid value 'sources'` ([journey 3](journeys/show-an-example-from-code.md)) | The **Content model** view; completion; hover | `ascribe_model`; the resource `ascribe://model/{+project}` | no (D2) | no (D3) |
| Learn the directives | `ascribe agents skill references/directives.md` | Completion; hover; the actions bar (`Cmd+K A`) | The resource `ascribe://directives` | no (D2) | no (D3) |

## Writing

| A user wants to | Command line | VS Code | MCP tools | In VS Code, for an agent | Site preview |
|---|---|---|---|---|---|
| Add a note, steps, or collapsible details | by hand | **Wrap in a note**, **Insert a note**, **Make the list steps**, **Insert steps**, **Wrap in collapsible details**, and five more | by hand | no (D2) | no (D3) |
| Add a section that differs by platform | by hand | **Insert content that varies**, **Add a variant**, **Remove this variant**, **Make the page one variant** | by hand | no (D2) | no (D3) |
| Say where something is available | by hand | **Mark where it's available**, **Set where the page is available**, **Change a feature's availability** | by hand | no (D2) | no (D3) |
| Link to a page or a heading, and know the link works | `ascribe link`; `ascribe outline` for a page's ids | **Insert a link**, **Link the selected text**, **Change where the link goes**; completion; the title shown for an empty link | `ascribe_link`; `ascribe_outline` | no (D2) | no (D3) |
| Give a heading an id that stays | by hand | **Give the heading a stable id**; **Copy a link to this section** | by hand | no (D2) | no (D3) |
| Reuse text: a phrase, a fragment, a glossary term | by hand | **Insert a phrase**, **Make this a phrase**, **Include a fragment**, **Add to the glossary** | by hand | no (D2) | no (D3) |
| Show an example from the code | by hand: `[sources.<name>]` and `@snippet` | **Insert a code snippet** | by hand | no (D2) | no (D3) |
| Add an image | by hand | **Insert an image**, **Set the image's width**, **Change the image's description** | by hand | no (D2) | no (D3) |
| Format pages | `ascribe fmt`, and `--check` | The setting `ascribe.formatOnSave`; VS Code's Format Document | `ascribe_format`, which returns the edits | no (D2) | no (D3) |

## Checking and fixing

| A user wants to | Command line | VS Code | MCP tools | In VS Code, for an agent | Site preview |
|---|---|---|---|---|---|
| See a page's or a project's problems | `ascribe check` | The Problems panel, for the editor's build | `ascribe_check` | `#ascribe_editor_problems`, with what isn't saved yet | The build's diagnostics, in `astro dev`'s log |
| Check every build | `ascribe check`, which does unless `--build` or `--editor-build` narrows it | **gap** ([finding 5](findings.md#5-nothing-in-the-editor-checks-every-build)) | `ascribe_check` | no (D2): the editor's build | no (D3): the one build the site shows |
| Understand a problem | `ascribe explain` | Hover; the code's link to the reference | `ascribe_explain` | no (D2) | no (D3) |
| Apply a fix Ascribe already knows | **gap**: `check`'s JSON lists each fix, and no command applies one | The lightbulb's quick fixes | The fixes in `ascribe_check`'s detailed answer, for the agent to apply | no (D2) | no (D3) |
| Hand a problem to an agent | `ascribe check --format prompt` | **Prompt Agent: Fix This Problem**, **Prompt Agent to Fix This File**, **Prompt Agent to Fix This Project** | The prompt `fix` | no (D2) | no (D3) |
| Say a reported problem is intended | by hand: `@intended`, or `intended` in frontmatter | The quick fix **Mark as intended, and write why** | by hand | no (D2) | no (D3) |
| Check the prose | `ascribe check --vale`, with `[checks.vale]`; `ascribe vale eject` | Vale's alerts in the Problems panel, with `[checks.vale]` | `ascribe_check`, when `[checks.vale]` has `in-check = true` | no (D2) | no (D3) |
| Have an agent's work checked without asking | `ascribe agents hook`, set up by `ascribe agents sync --with-hook` | no (D4) | no (D5): the plugin's stop hook and its language server do this beside the tools | no (D2) | no (D3) |

## Finding and changing across the project

| A user wants to | Command line | VS Code | MCP tools | In VS Code, for an agent | Site preview |
|---|---|---|---|---|---|
| Find where something is used | `ascribe refs` | **Find All References**; the **Used by** view | `ascribe_refs` | no (D2) | no (D3) |
| See a page's headings | `ascribe outline` | The **Pages** view. For the headings of one page, VS Code's own Outline, which reads a Markdown file's headings; the server answers no request for them | `ascribe_outline` | no (D2) | no (D3) |
| Rename a phrase everywhere | **gap**: the session's rename was `sed`, with `ascribe refs` before and after ([journey 4](journeys/rename-a-phrase.md)) | **Rename this phrase everywhere**; `F2` | no (D1): `ascribe_refs` finds the uses, and the agent edits each | no (D2) | no (D3) |
| Rename a dimension's value everywhere | **gap** | **Rename a dimension value everywhere**; `F2` | no (D1) | no (D2) | no (D3) |
| Rename or move a page and keep its links | **gap**: `ascribe refs` lists the links to fix | Renaming or moving the file in the Explorer | no (D1) | no (D2) | no (D3) |
| Search the pages' text as readers see it | **gap** (the [search proposal](https://github.com/ascribed-dev/ascribe/blob/62bf76a9/project-docs/proposals/search.md)) | **gap** | **gap** | no (D2) | no (D3) |

## Seeing it as a reader

| A user wants to | Command line | VS Code | MCP tools | In VS Code, for an agent | Site preview |
|---|---|---|---|---|---|
| See a page as one build's readers do | `ascribe render --build`. A switcher's arms come one after another, each under a bold label, so "one at a time" can't be seen ([journey 2](journeys/add-a-platform-section.md)) | **Open Page Preview**, **Select Preview Build**, **Switch Build** | `ascribe_render` | no (D2) | The page itself, in the site's own layout |
| See what a build leaves out of a page | `ascribe render` for each build, compared by eye; `ascribe report builds` for the project | **Dim What the Build Leaves Out** | `ascribe_render`, for each build | no (D2) | no (D3) |
| See the page on the real site | `astro dev`, which isn't Ascribe's | **Open Site Preview** | no (D1) | no (D2) | It is this |

## Reviewing a change

| A user wants to | Command line | VS Code | MCP tools | In VS Code, for an agent | Site preview |
|---|---|---|---|---|---|
| See which pages a change touches, and how | `ascribe diff`, as `text`, `json`, or `html`. One page's changes, in the terminal: **gap**; a page is named "only with --format prompt" ([journey 5](journeys/review-a-pull-request.md)) | **Start Review**, **Changed Pages**, the marks in the preview | `ascribe_changes` | `#ascribe_review_changes` | The **Ascribe review** app in the toolbar |
| Know when an example changed and its words didn't | `ascribe drift` | **gap**: review marks what changed on a page, and nothing lists the pages whose examples moved alone | **gap**: `ascribe_changes` is `ascribe diff`, and no tool is `drift` | no (D2) | no (D3) |
| Read a pull request's comments beside the page | no (D6) | The comments in the preview | no (D6) | `#ascribe_review_threads` | The **Ascribe review** app |
| Answer, resolve, or add a comment | no (D6) | **Reply Now**, **Add to Review**, **Resolve**, **Reopen** | no (D1) | no (D2) | The **Ascribe review** app |
| Ask an agent to review a page | `ascribe diff --format prompt` | **Prompt Agent** on a changed page | The prompt `review` | no (D2) | **gap**: the Astro guide's account of the app has changes and comments, and no prompt |

## Building and publishing

| A user wants to | Command line | VS Code | MCP tools | In VS Code, for an agent | Site preview |
|---|---|---|---|---|---|
| Build the outputs | `ascribe build` | no (D7): the preview renders the page, and the site preview runs the site's own build | no (D1) | no (D2) | The integration runs the build |
| Publish with Astro | `npm install @ascribed/astro`, and its seven steps in the guide | no (D4) | no (D1) | no (D2) | It is this |
| Publish what AI agents read | by hand: `[consumer] agents = true` | by hand | no (D1) | no (D2) | The integration serves `llms.txt` and each page's Markdown |
| Give something outside the docs a link that stays | by hand, `@id {permalink=true}`; then `ascribe permalinks sync`, `list`, `verify`, `retire` | by hand; a quick fix runs the sync | no (D1) | no (D2) | The integration serves each permalink's address |
| Keep examples from another repository current | `ascribe sources fetch`, `update`, `status` | **gap** | no (D1) | no (D2) | no (D3) |

## Keeping a project healthy

| A user wants to | Command line | VS Code | MCP tools | In VS Code, for an agent | Site preview |
|---|---|---|---|---|---|
| Check in CI | `ascribe check --deny-warnings`; `ascribe fmt --check`; `ascribe agents sync --check`; `ascribe diff --exit-code`; `ascribe drift --exit-code`. Nothing a project holds says which to run: the session went looking outside it ([journey 6](journeys/set-up-ci-checks.md)) | The walkthrough's "Check in CI", which leads to the docs | no (D1) | no (D2) | no (D3) |
| Know what state the project is in: counts, what each build leaves out, outside links, delivery to agents | `ascribe report` | **gap** | **gap** | no (D2) | no (D3) |
| Give an agent the project's rules | `ascribe agents sync`; `ascribe agents rules`; `ascribe agents skill` | no (D4) | The resource `ascribe://instructions/{+project}` | no (D2) | no (D3) |

## Why not

The reasons behind each "no", and where each is said.

- **D1. The MCP tools read and return, and change nothing.** "No tool writes a file: `ascribe_format` returns edits" ([decision 56](../decisions.md)). So anything that writes, builds, fetches, or sets a project up isn't a tool. An agent does it by editing, with what the tools tell it.
- **D2. The extension's own tools are for what only the editor has:** text that isn't saved, and the review that's open ([decision 57](../decisions.md)). Everything else an agent in VS Code gets from the MCP server, which the extension offers, so this column is short by design and the MCP column applies beside it.
- **D3. The site preview shows the site.** It's a site's own dev server with a review app added, not a place to write or check.
- **D4. The editor doesn't set up what lives outside it:** an agent's files, a site's packages. Not said anywhere found; it's how the extension is drawn.
- **D5. A check nobody asked for is a hook's job, not a tool's.** The tools answer when called.
- **D6. The binary doesn't talk to GitHub** ([decision 1](../decisions.md)). Comments are read and written by the extension and the site preview's app, in TypeScript.
- **D7. The editor shows; it doesn't publish.** Building a project's outputs belongs to the command line, CI, and the site. Not said anywhere found.

## What the table shows

Each of these is a finding now, in [findings.md](findings.md).

- **The command line and the MCP tools are the same nine jobs.** Every tool answers as a command does, and the map names the command beside each. The MCP column's gaps are the command line's: neither renames (finding 51), and neither searches.
- **The editor is where the writing is.** Eight rows are "by hand" on the command line and for an agent, against thirty-five actions in VS Code. In phase 3 the agents wrote each construct by hand from the skill's reference, and got each right.
- **Three things only the command line does:** `ascribe report`, `ascribe sources`, and `ascribe build`. Two only the editor does: renaming across the project (finding 51), and applying a fix (finding 52).
- **Nothing starts a project or a page for a person** (finding 53).
- **Rows with four or five full cells:** seeing problems, seeing what the project allows, checking a link, finding uses, rendering a page, seeing what a change touches. Each is one command with a tool and an editor feature over it: one job said three ways, not three jobs. In the sessions, an agent with a shell used the command every time and the tool never (finding 17).
- **No cell is open.** The seven that were are settled: four are gaps, two are deliberate, and one is VS Code's own to do.
