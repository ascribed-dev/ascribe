# Start a project and publish a first page

Part of [The surface audit](../README.md), [phase 3](../phase-3-journeys.md), journey 1. The script first; then the session's notes.

## The task

> Follow https://ascribed-dev.com/getting-started/ to start a documentation project in this folder, write a first page about making tea, and publish it as a website I can open on this machine.

That sentence is the whole prompt. It gives the getting-started page's address and nothing about what's on it. "A website I can open" is the result a check can't confirm: `ascribe check` passing says nothing about whether there's a site with the page in it.

## Starting point

An empty folder: no files, and not a git repository, since there's nothing in it for a commit to hide. This is the coldest of the journeys. Every place the page and what happens differ is a note.

## The session

One, with no plugin, and with `WebFetch` allowed so the agent can read the page it's given:

```shell
claude -p "Follow https://ascribed-dev.com/getting-started/ to start a documentation project in this folder, write a first page about making tea, and publish it as a website I can open on this machine." \
  --setting-sources project \
  --allowedTools "Read,Edit,Write,Glob,Grep,Bash,WebFetch" \
  --no-session-persistence --include-hook-events \
  --output-format stream-json --verbose \
  > <checkout>/project-docs/surface-audit/journeys/transcripts/start-a-project-nothing.jsonl
```

The flags are [phase 1's](fix-a-broken-link.md#what-changed-in-these-two-commands-and-why), flag for flag, and the reasons for each are there. `<checkout>/target/release` is first on the path, so `ascribe` is the release build of `main` (`ascribe 0.2.0`, with the `permalinks` commands). Standard error goes to a file in the scratch folder, and the notes say what was in it. The session is run once, with ten minutes at most.

The path is a departure from "an empty folder, and nothing else", made because every session and check in this phase uses the one binary: a bare `ascribe` in this session is the release build of `main`, while `npx ascribe`, which the page tells a reader to type, is whatever `npm` installed. The notes say which the agent ran, and what version each was.

## What the preparer checks afterwards

- The files the session left, listed, without `node_modules`.
- `ascribe check` in the folder, with its exit code; and `npx ascribe --version`, for what `npm` installed.
- Whether there's a built site, and whether the page about tea is in it: the built HTML is read for the page's title and text.
- Whether what the agent said to open does open: the file exists, or the address answers.
- Each step of the getting-started page, beside what the agent did for it.

## Notes

One section per session, with phase 1's parts, in this order:

- **When, with what:** the date and time, Claude Code's version and model, the binary, the copy, and what the session's first event listed of the plugin's (its tools, its skill, its commands, the language server), with whether a synced skill or an account's connector was used.
- **Transcript:** its path.
- **Steps:** a numbered list, one line per step: what the agent ran or asked, then what came back, quoted from the transcript.
- **Tried first:** the commands it guessed, what it read, and where it went wrong.
- **What the hook or the plugin's tools said:** quoted.
- **Doubts:** each quoting the transcript.
- **Time:** by the clock, and as the session's last event reports it.
- **Result:** what the agent said it had done, then what the preparer's checks said.
- **Seen on the way:** one line per thing that looks like a finding, each quoting its evidence. Phase 5 writes the findings.

### With nothing

- **When, with what:** 10 October 2026, 9:57 PM. Claude Code 2.1.296, model `claude-opus-5-5`, started from zsh on macOS in the empty folder `start-a-project-nothing`, with `target/release` first on the path. No plugin: the session's first event listed no tool, skill, or command of Ascribe's. The user's synced skills and the account's connectors (`claude.ai Claude Docs` connected, three more `needs-auth`) were listed, and none was used. Standard error was empty. Before the session, `https://ascribed-dev.com/getting-started/` answered 200, and `npm view @ascribed/cli dist-tags` said `latest: '0.2.0'`, `next: '0.2.1-next.13'`. Node.js was 24.21.0 and `npm` 11.19.0.

  The agent ran `npx ascribe` every time, as the page says to, so every Ascribe command in this session was the release from `npm`, `ascribe 0.2.0`. The binary on the path was never run.

- **Transcript:** [`transcripts/start-a-project-nothing.jsonl`](transcripts/start-a-project-nothing.jsonl).

- **Steps:**

  1. `ToolSearch`, `select:WebFetch`. Claude Code's own step: it loads a tool's definition before the first call.
  2. Bash: `ls -la`. An empty folder.
  3. `WebFetch`, `https://ascribed-dev.com/getting-started/`, asking for "the full getting-started instructions verbatim: every command, install step, file name, config content, directory layout, and how to build, serve, or publish the site. Include all code blocks exactly." Came back: "I can't reproduce the full guide verbatim, but here's a concise summary of the setup steps.", then nine numbered steps in prose with no code, such as "**7. Publish with Astro.** In an Astro project, install `@ascribed/astro`, register the integration in `astro.config.mjs` with your site URL and the `site` build, then define a content collection, a route, and a layout. The Astro guide walks through these."
  4. Bash: `curl -sL https://ascribed-dev.com/getting-started/ -o fetched/gs.html`, into the session's scratchpad, then a Python script that cut the tags out of `<main>`; and `node -v; npm -v`. Came back: 39,446 bytes of HTML, and the page's text, whole. Its second line: "For AI agents: the documentation index is at [/llms.txt] llms.txt, and this page is available as [/getting-started.md] Markdown."
  5. Bash: `curl -sL https://ascribed-dev.com/guides/astro.md`. Came back: the Astro guide as Markdown, 29.6 KB, too long for the tool's answer and saved to a file.
  6. Bash: `npm init -y >/dev/null && npm install --save-dev @ascribed/cli 2>&1 | tail -5 && npx ascribe --version`. Came back: `added 2 packages, and audited 3 packages in 1s`, `ascribe 0.2.0`.
  7. `Read`, the saved Astro guide.
  8. Said: "Setting up the project now: the Ascribe CLI is installed (0.2.0), and I've read the getting-started and Astro guides." Bash, one command that wrote `ascribe.toml` (with `spec = "0.1"`, a dimension `tea` with `black`, `green`, and `herbal`, and `[consumer]` with `profile = "astro"` and `site = "http://localhost:4321"`), `.gitignore`, and `docs/index.md`, "Make a cup of tea", with a `@note {type=tip}:`, three `@variant {tea=…}:` arms, and `@steps`; then `npx ascribe check; npx ascribe fmt --check; echo "fmt exit $?"`. Came back: `checked 1 file: 0 errors, 0 warnings`, `fmt exit 0`.
  9. Bash, one command: `npm install astro @ascribed/astro sharp`; `npm ls astro @ascribed/astro @ascribed/cli --depth=0`; wrote `astro.config.mjs`, `src/content.config.ts`, `src/pages/[...slug].astro`, and `src/layouts/Docs.astro`; set the package's scripts; `npx astro build 2>&1 | tail -25`. Came back: `@ascribed/astro@0.2.0`, `@ascribed/cli@0.2.0`, `astro@7.3.8`; `[@ascribed/astro] running ascribe build for "site"`, `[@ascribed/astro] checked 1 file: 0 errors, 0 warnings`, `[@ascribed/astro] built site/site: 1 page, 0 assets`; and `[build] 1 page(s) built in 3.83s`, `[build] Complete!`
  10. Bash: `lsof -nP -iTCP:4321 -sTCP:LISTEN | tail -1`. Nothing was listening.
  11. Bash, in the background: `npx astro preview --port 4321`.
  12. Bash: a loop of `curl` until `http://localhost:4321/` answered, then the page's `<body>`. Came back: `200`, and `<h1>Make a cup of tea</h1>`, `<ascribe-note type="tip" label="Tip">`, `<ascribe-tabs sync="tea">` with three `<ascribe-tab>` elements.
  13. Bash: the preview's output, and the port again. Came back: `Preview server running at http://localhost:4321 (pid 50483)`, `200`.
  14. Answered: "The documentation project is set up in this folder and the tea page is live at **http://localhost:4321/**. The server is running in the background (pid 50483) and answered 200 with the rendered page when I checked."

- **Tried first:** `WebFetch` on the address, which is what the task gave it. It got a summary without a command in it, and went round: `curl` for the HTML and its own script to read it. From the page's "For AI agents" line it learned pages have a Markdown form, and fetched the second page that way, `guides/astro.md`, in one step.

  No Ascribe command was guessed: `npx ascribe --version`, `check`, and `fmt --check` are the page's. It never ran `npx ascribe build`, `--help`, or `explain`, and never saw a diagnostic.

- **What the hook or the plugin's tools said:** nothing; the session had neither.

- **Doubts:**
  - What "publish" means on one machine. The page's last step of publishing is a sentence, and the agent decided for itself that a built site served by `astro preview` was the answer, and said how to stop it: "**Stop the server:** `npx astro preview stop`".
  - What address to give the site. "The site URL is set to `http://localhost:4321` in both `ascribe.toml` and `astro.config.mjs`; the two must agree, so change both when you have a real host." It had that from the Astro guide, and wrote it into both files as a comment.
  - What to leave out, listed at the end: "I skipped the guide's optional steps: the VS Code extension, the CI workflow (this folder isn't a git repository), and `agents sync`, which isn't released yet."

- **Time:** 97 seconds by the clock; the session reports 95.8 seconds, fourteen turns. The two installs and the Astro build are most of it.

- **Result:** a project, a page, and a built site that opened. The agent knew from `npx ascribe check`, the Astro build's log, and a `200` from the server it started. The preparer ran, in the folder:
  - The files, without `node_modules`: `ascribe.toml`, `.gitignore`, `docs/index.md`, `package.json`, `package-lock.json`, `astro.config.mjs`, `src/content.config.ts`, `src/layouts/Docs.astro`, `src/pages/[...slug].astro`; and, built, `.ascribe/build/site/`, `.astro/`, and `dist/index.html` with one stylesheet.
  - `npx ascribe --version`: `ascribe 0.2.0`. `npx ascribe check`: exit 0, `checked 1 file: 0 errors, 0 warnings`. The binary from `main`, `ascribe check`: the same, exit 0.
  - `dist/index.html`: `<title>Make a cup of tea</title>`, the six steps ("Fill the kettle with fresh water, and heat it."), the note, the tabs, and the steps as Ascribe's elements, and the elements' script in the page.
  - `curl http://localhost:4321/`: `200`, with the session over. `lsof` showed `node` listening, pid 50483. So what the agent said to open did open.
  - Then `npx astro preview stop`, to leave nothing running: `Stopped preview server (pid 50483).`

  Each step of the getting-started page, beside what the agent did:

  1. **Node.js 24 or later.** `node -v`: `v24.21.0`.
  2. **Install the command,** `npm install --save-dev @ascribed/cli` and `npx ascribe --version`. Done as written, after an `npm init -y` the page doesn't mention.
  3. **Install the editor.** Skipped.
  4. **Create `ascribe.toml`,** and add `.ascribe/` to `.gitignore`. Done, with the page's shape and its own dimension. It left out `[phrases]` and added `profile = "astro"` under `[consumer]`, from the Astro guide.
  5. **Write a page,** `docs/install.md`, which links to a page that doesn't exist. It wrote `docs/index.md` about tea instead, with the page's three directives and no link.
  6. **Check it,** where the page shows `ASC036` for the missing page and says to create it. `npx ascribe check` passed at once: the agent never met the error the page teaches with.
  7. **Build it,** `npx ascribe build`. Not run. The Astro integration ran the build itself.
  8. **Format it,** `npx ascribe fmt`. It ran `npx ascribe fmt --check`.
  9. **Publish it with Astro.** "In an Astro project (`npm create astro@latest` makes one)": it didn't run that, and wrote the files by hand. Step 1, `npm install @ascribed/astro`: done, with `astro` and `sharp` in the same command. Step 2, `astro.config.mjs`: as the page has it, with its own address. Step 3, "Define the content collection, a route, and a layout that loads the elements. Astro walks through each file.": done from the other page.
  10. **Check it in CI.** Skipped: "this folder isn't a git repository".
  11. **Work with an AI agent,** `npx ascribe agents sync`. Skipped: "which isn't released yet".

- **Seen on the way:**
  - The getting-started page, read through Claude Code's `WebFetch`, came back as a summary with every command taken out: "I can't reproduce the full guide verbatim, but here's a concise summary of the setup steps." The agent had to download the HTML and strip it. That's the client's tool, not the site. The site's answer to it is the line the agent found only afterwards, "this page is available as [/getting-started.md] Markdown", which is in the page and so was in what `WebFetch` summarized away.
  - The page is titled for a published site and stops one step short of one. Its last instruction for publishing is "Define the content collection, a route, and a layout that loads the elements. [Astro] walks through each file.", and nothing on the page builds, serves, or opens the site: checked by the preparer, the page's Markdown has no `astro dev`, `astro build`, or `astro preview`. The Astro guide, 29.6 KB, has `npx astro dev` and `npx astro build`. The agent needed both pages and chose `astro preview` itself.
  - The page says an Astro project comes from `npm create astro@latest`, and the agent didn't run it: it made the Astro project out of the four files the Astro guide gives. The page's two routes to a site, a new Astro project with Ascribe added or an Ascribe project with Astro added, aren't told apart, and the agent took the second.
  - `ascribe.toml`'s `[consumer]` `site` and Astro's `site` have to be the same, and the getting-started page shows each (`https://docs.example.com`, twice) without saying so. The agent got it from the Astro guide and wrote it down beside both: "the integration fails the build if they disagree."
  - The page teaches `ascribe check` with a link that's broken on purpose. An agent writing its own first page doesn't make that link, so it never sees a diagnostic, a code, or `explain`: the first check is `checked 1 file: 0 errors, 0 warnings`.
  - The published page marks its last section "Ascribe (in main, not yet released)", three times in that section, and the agent read it rightly and skipped it. Checked by the preparer: `npx ascribe agents --help` with the release the page installs answers `error: unrecognized subcommand 'agents'`. So the page a newcomer is sent to describes a command their install doesn't have, under a label that says so.
  - `npx ascribe build`, the page's own step, wasn't needed: `[@ascribed/astro] running ascribe build for "site"`. The page has it as a step before publishing and doesn't say the integration does it.
  - The session left a server running after it ended, `node` on port 4321, because the task asked for something that opens. The preparer stopped it with the command the agent gave.

## What the session showed

One session, so nothing to compare it with but phase 1's. It's the only one of the nine that ran the released Ascribe, and the only one that started from the docs.

- **It finished, in 97 seconds, with a site that opened.** From an empty folder and an address, the agent got to a served page with Ascribe's tabs, note, and steps in it, without one failed command.
- **The getting-started page was half of what it needed.** Everything up to the check came from that page. The site came from the Astro guide, which the page hands off to in its last publishing step.
- **Reading the page was the hardest part.** `WebFetch` gave a summary; the agent downloaded and stripped the HTML; only then did it find the page's Markdown form, which it used for the second page.
- **It took the page as a pattern, not a script.** It wrote its own content model and its own page with the page's directives, and skipped the steps that didn't serve the task: the editor, the broken link, `ascribe build`, CI, and the unreleased command.
- **It never needed help from Ascribe.** No `--help`, no `explain`, no diagnostic. What it would do with one, phase 1's sessions and this phase's other journeys show.

## Preparing a copy

For the preparer.

```shell
mkdir <scratch>/start-a-project-nothing
```

Nothing else. Before the session, see that `https://ascribed-dev.com/getting-started/` answers 200, and note what `npm view @ascribed/cli dist-tags` says, since the page's commands install the `latest` release and the binary on the path is `main`'s.
