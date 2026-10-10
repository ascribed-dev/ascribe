# The agents pass: checks by hand

Part of [Agents](README.md). Everything left to try with real agents, in the order to run it: the hand checks from phases 8, 9, and 10, and phase 11's runs. They need Claude Code, VS Code with GitHub Copilot, Copilot's cloud agent, and Codex or Cursor, so they're run on your machine and your GitHub account. Record each result in [pass-results.md](pass-results.md). Nothing here blocks a merge.

Never use this repository or its pull requests for these: work in scratch repositories.

## 0. Set up

1. Build or install `ascribe` from `main`, and put it on your path (`cargo build --release -p ascribe-cli`, or `npm install -g @ascribed/cli@next`). Check `ascribe --version`.
2. Make three scratch git repositories, outside this one, each with one commit:
   - **quill-bare**: a copy of `examples/quill`, with nothing else.
   - **quill-full**: a copy of `examples/quill`, then `ascribe agents sync --target claude --target copilot --target codex --with-hook`, committed.
   - **monorepo-full**: a copy of `examples/monorepo`, then the same `sync` in each of `docs/`, `handbook/`, and `handbook/pages/security/`, committed.
3. Push **quill-full** to a new private GitHub repository for step 4.

## 1. Claude Code (phase 9)

In **quill-full**, unless it says otherwise.

1. In Claude Code's terminal, run `/plugin marketplace add <checkout>/plugins/ascribe`, then `/plugin install ascribe@ascribe`. Restart, open a project, and confirm `/ascribe:check` appears. (Use a copy of **quill-bare** for the plugin, since quill-full's own hooks and the plugin together report each problem twice.)
2. Ask it to link to `missing.md` in a page. The error should appear once, after the edit.
3. Leave the error and let it finish. It should be sent back once, then allowed to stop.
4. Repeat 2 and 3 in the Claude Code extension for VS Code.
5. Take `ascribe` off your path and start a session. You, not the model, should see a hook error.
6. Note how much context one error takes in the transcript.

## 2. Codex (phase 9)

7. In **quill-full**, trust the project in Codex, and repeat 1.2 and 1.3. The hooks are in `.codex/hooks.json`.

## 3. VS Code with Copilot (phases 8 and 10)

Open **quill-full** in VS Code with the Ascribe extension and Copilot, in agent mode.

8. The **Ascribe** MCP server appears in VS Code's list of MCP servers and starts.
9. `#ascribe_editor_problems`, `#ascribe_review_threads`, and `#ascribe_review_changes` are in chat's `#` menu and run (VS Code asks to confirm each, as it does for any extension's tool). With review off, the review tools say so in one line.
10. Copilot lists the skill, and the slash commands `new-page`, `fix`, and `review` from the MCP server.
11. Ask Copilot to break a link in a page without opening it. The problem is reported back to it.
12. On Windows, with `@ascribed/cli` installed in the project (a `.cmd` shim in `node_modules/.bin`), the MCP server starts.

## 4. Copilot's cloud agent (phase 10)

In the GitHub copy of **quill-full**.

13. Run `ascribe agents sync --target copilot --with-hook --cloud`, merge the result to the default branch, and paste the printed MCP JSON into **Settings → Copilot → MCP servers**.
14. Assign Copilot an issue: "Add a page about offline sync, linked from the quickstart." Check that the setup steps' log shows `ascribe --version`, that the agent runs the check, and that the stop hook holds it while errors remain.
15. Remove the MCP JSON from the settings, run `sync` again with `--agent`, merge `.github/agents/ascribe-docs.md`, and assign another issue to the **ascribe-docs** agent. The same three things hold.

## 5. Phase 11's runs

Seven tasks, each run twice with Claude Code and twice with Copilot in VS Code: once in a fresh copy of **quill-bare**, once in a fresh copy of **quill-full** (task 5 in **monorepo-full**, and a bare monorepo copy). A fresh copy for every run. Don't explain Ascribe's syntax to the agent; if you have to step in, count it.

16. **Task 1.** "Add a guide page about offline sync, a made-up feature, and link it from the quickstart."
17. **Task 2.** Seed one page with: a broken link, a missing `title`, `@note {type=danger}`, an unclosed `@note:`, `{producta}`; and write "Quill" in prose where `{product}` exists. "`ascribe check` reports problems; fix them." Judge the literal "Quill" by reading the result, in its own column.
18. **Task 3.** "In `install-agent.md`, add a third deployment, Quill Edge, to the section that differs by deployment." (It needs a new value in `ascribe.toml`'s `deployment` dimension, and an arm.)
19. **Task 4.** "Rename the 'Rotate keys' heading in `keys.md` to 'Rotate your API key', and keep every link to it working."
20. **Task 5.** In the monorepo: "Add a page to the security handbook about reporting a lost laptop." It should meet `handbook/pages/security/ascribe.toml`'s rules, not the handbook's or the docs project's.
21. **Task 6.** Rebuild the review fixture (`scripts/review-fixture/setup.ts`), leave three review comments on its pull request, open it in VS Code's review, and start from **Prompt agent: all open**.
22. **Task 7.** Seed a few hundred warnings across many pages, of five or six kinds (an image without alt text, an unknown directive, a heading with a phrase and no `@id`, two headings with the same text, `{{product}}`, an undeclared phrase). "Clean up the warnings; start from `ascribe check --summary`." Record whether it works rule by rule or file by file, and whether it stops and asks when it stops making progress.
23. For each run, record: whether `ascribe check --deny-warnings` passes at the end, for every build; how many times you stepped in; whether the agent ran the check itself; and anything it got wrong that a tool could have told it. The Claude Code hook reports the editor's build's errors only, so a run where it was quiet can still fail.

## 6. An agent with only `sync`'s files

24. In Codex, or Cursor outside the VS Code extension, run tasks 1 and 2 once, in a fresh copy of quill with only `ascribe agents sync` run (no hooks, no MCP server).

## 7. After the runs

25. Fill in the tables in [pass-results.md](pass-results.md). A failure a small change fixes (a tool's description, a line in the instructions, a prompt's wording) gets a pull request; one that needs design gets an issue, listed there.
