# Start a project, after phase 5

Part of [Page templates](../README.md), [phase 5](../phase-5-project.md): the acceptance criterion "Getting started's steps, followed in an empty folder, work as written", tested as the surface audit's [journey 1](../../surface-audit/journeys/start-a-project.md) was, against #410's pull request's preview. The steps followed by hand first, then the session's notes, in the journey's parts.

## The steps, by hand

In an empty folder, with `@ascribed/cli@next` at `0.3.1-next.19` (built from `779ff65`, after #409), and the page's npm arm at each step:

1. `npm install --save-dev @ascribed/cli@next`, then `npx ascribe --version`: `ascribe 0.3.1-next.19 (779ff6539e7f)`.
2. `npx ascribe init`: `wrote ascribe.toml`, `made docs/, the folder the pages go in`, and the Astro guide's permalink; exit 0. The file is the page's `#init` snippet, byte for byte (diffed).
3. The page's `#added` snippet appended, `.ascribe/` in `.gitignore`, and `docs/install.md` from the page.
4. `npx ascribe check`: the one `link-target-missing` on `configure.md` the page shows, at `docs/install.md:30:22`; exit 1.
5. `docs/configure.md` with a title and a description: `0 errors, 0 warnings, 1 advice`, the advice `page-orphan` on `install.md`, which no page links to; exit 0. The page said "with a `title`", which `init`'s model doesn't accept alone, so it now says "with the same frontmatter fields as `install.md`".
6. `npx ascribe build`: `site/`, `plain/`, and `json/`, two pages each; exit 0.
7. `npx ascribe fmt --check`: nothing to change; exit 0.

## The task

> Follow https://pr-433--ascribe-docs.netlify.app/docs/getting-started/ to start a documentation project in this folder, write a first page about making tea, and publish it as a website I can open on this machine.

Journey 1's sentence, with the preview's address for the page's. The page itself, not its `/go/` permalink, since the preview serves permalinks as a page that refreshes to the address, which `WebFetch` may not follow, and that's the preview's, not the page's.

## The session

Journey 1's command, flag for flag, in an empty folder that isn't a git repository:

```shell
claude -p "<the task>" \
  --setting-sources project \
  --allowedTools "Read,Edit,Write,Glob,Grep,Bash,WebFetch" \
  --no-session-persistence --include-hook-events \
  --output-format stream-json --verbose \
  > transcripts/start-a-project.jsonl
```

## Notes

- **When, with what:** 12 October 2026, 17:40 UTC. Claude Code 2.1.296, model `claude-sonnet-5-5` (the CLI's default here, not journey 1's), on Linux x64 in a cloud container. Node.js 22.22.0 and `npm` 10.9.4: older than the Node.js 24 the page asks for. No `ascribe` on the path, and no plugin: the first event lists no tool, skill, or command of Ascribe's. `npm view @ascribed/cli dist-tags`: `latest: '0.3.0'`, `next: '0.3.1-next.19'`. Standard error was empty.
- **Transcript:** [`transcripts/start-a-project.jsonl`](transcripts/start-a-project.jsonl).
- **Steps:**
  1. `WebFetch` of the page: "Prerequisites: Node.js 24 or later … Install the CLI as a dev dependency with `npm install --save-dev @ascribed/cli`".
  2. `npm init -y`, `npm install --save-dev @ascribed/cli`, `npx ascribe --version`: `npm warn EBADENGINE`, `ascribe 0.3.0`.
  3. Without `init`, which 0.3.0 doesn't have, the route by hand: `spec = "0.1"`, `docs/`, `.gitignore`, and `docs/making-tea.md`, with a `title` and a `description`, then `npx ascribe check`: "`[ASC047] Error: \`description\` isn't a field of the content type \`page\``".
  4. "Version 0.3.0 differs from the guide; I'll drop the description and the H1." `npx ascribe check`: `0 errors, 0 warnings, 0 advice`; `npx ascribe build`: three outputs.
  5. `WebFetch` of a guessed `/docs/publishing/astro/`: 404. Then the page's links with `curl`, and the Astro guide from its `.md` address.
  6. `[consumer]` with `profile`, `site`, `base-path`, and `trailing-slash`, `docs/index.md`, Astro installed, and the integration's command: "wrote src/content.config.ts … wrote src/nav.ts".
  7. `npx astro build`, then `npx astro preview` in the background, and `curl` of both pages: 200.
- **Tried first:** the hand-written route, since the page's first step installs the release, and `ascribe init` is marked as in main. It kept the page's `description`, which the route by hand says to leave out; the check said why in one error, and it dropped it. An address for the Astro guide, guessed, before the page's own link.
- **What the hook or the plugin's tools said:** nothing; there was neither.
- **Doubts:** "Version 0.3.0 differs from the guide": the page says so, in the line that opens "Write it by hand", and the `WebFetch` summary the agent worked from left that line out.
- **Time:** 80 seconds by the clock (17:40:17 to 17:41:37); `duration_ms` 79106, 10 turns.
- **Result:** "The tea site is built and running on this machine at http://localhost:4321/." Checked afterwards: `npx ascribe check` exits 0 with `0 errors, 0 warnings, 0 advice`; `dist/making-tea/index.html` has the title "Making tea"; `http://localhost:4321/making-tea` answered 200.
- **Seen on the way:**
  - The two routes ask for different frontmatter: `init`'s default type has a `description`, and the one-line `ascribe.toml` has no field but `title`. An agent on the release that copies the page's example meets `frontmatter-unknown-key` once. That ends when a release has `init`, and the route by hand is for that release only.
  - The agent wrote `:::note`, which isn't Ascribe's, and `ascribe check` said nothing about it.
  - On Node.js 22, the install warns (`EBADENGINE`) and everything works.
