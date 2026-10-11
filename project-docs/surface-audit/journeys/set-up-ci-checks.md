# Set up checks in CI

Part of [The surface audit](../README.md), [phase 3](../phase-3-journeys.md), journey 6. The script first; then the session's notes.

## The task

> Set up GitHub Actions for this repository, so a pull request that breaks the docs fails its checks.

That sentence is the whole prompt. It doesn't say which commands the workflow should run, or how the runner gets Ascribe. What a check can't confirm is the workflow: `ascribe check` says nothing about a file under `.github/`, and the result that matters is a workflow that fails when it should.

## Starting point

A fresh copy of `examples/quill`, without its `README.md`, as a git repository with one commit. It has no `.github/` folder and no `package.json`, so nothing in it says how Ascribe is installed: the getting-started page's workflow runs `npm ci` and `npx ascribe check`, which needs a `package.json` and a lockfile this project doesn't have.

## The session

One, with the plugin:

```shell
claude -p "Set up GitHub Actions for this repository, so a pull request that breaks the docs fails its checks." \
  --setting-sources project \
  --plugin-dir <checkout>/plugins/ascribe \
  --allowedTools "Read,Edit,Write,Glob,Grep,Bash,mcp__plugin_ascribe_ascribe__*" \
  --no-session-persistence --include-hook-events \
  --output-format stream-json --verbose \
  > <checkout>/project-docs/surface-audit/journeys/transcripts/set-up-ci-checks-plugin.jsonl
```

The flags are [phase 1's](fix-a-broken-link.md#what-changed-in-these-two-commands-and-why), flag for flag, and the reasons for each are there. `<checkout>/target/release` is first on the path, so `ascribe` is the release build of `main` (`ascribe 0.2.0`, with the `permalinks` commands). Standard error goes to a file in the scratch folder, and the notes say what was in it. The session is run once, with ten minutes at most.

## What the preparer checks afterwards

- `ascribe check`, with its exit code; `git status --short` and `git diff`.
- The workflow, read whole and quoted in the notes: what starts it, how it installs Ascribe and at which version, which commands it runs, and whether its YAML parses.
- Whether it would run as written: an action or a package it names that doesn't exist, an `npm ci` with no lockfile, a version that isn't published (`npm view @ascribed/cli versions`).
- Its commands, run by hand in a copy of the session's copy, twice: as the session left it, where they should pass, and with a link broken (`keys.md` renamed, as in phase 1), where they should fail. GitHub isn't involved: the workflow itself isn't run.
- Beside it, the workflow the getting-started page gives, and where the agent's differs.

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

### With the plugin

- **When, with what:** 10 October 2026, 9:56 PM. Claude Code 2.1.296, model `claude-opus-5-5`, started from zsh on macOS with `target/release` first on the path (`ascribe 0.2.0`). The plugin from the checkout, `ascribe@inline` version 0.2.0. Copy `set-up-ci-checks-plugin`, in a scratch folder, prepared by the agent that ran it. The session's first event listed the plugin's nine tools (`mcp__plugin_ascribe_ascribe__ascribe_check` and the eight others), `LSP`, the skill `ascribe:ascribe`, and the commands `ascribe:check`, `ascribe:new-page`, and `ascribe:review`; the server `plugin:ascribe:ascribe` was `connected`. The user's synced skills and the account's connectors (`claude.ai Claude Docs` connected, three more `needs-auth`) were listed too, and none was used. Standard error was empty. `npm view @ascribed/cli dist-tags` before the session: `latest: '0.2.0'`, `next: '0.2.1-next.13'`.

  **The session left its copy.** Its third step was `cd /Users/kylerollins/Documents/GitHub/ascribe`, the checkout the binary and the plugin are in, and it took its workflow from two files there. That isn't "nothing but the project and what Ascribe put in it" (README decision 6). It's noted as it went and wasn't run again: the session is run once, and what sent the agent there is a result. See "Seen on the way".

- **Transcript:** [`transcripts/set-up-ci-checks-plugin.jsonl`](transcripts/set-up-ci-checks-plugin.jsonl).

- **Steps:**

  1. Bash: `git ls-files | head -100; echo ---; cat ascribe.toml 2>/dev/null; echo ---; ls -la; ls .github 2>/dev/null; git remote -v; which ascribe; ascribe --help 2>&1 | head -40`. Came back: the seven files, the content model, no `.github`, no remote, `/Users/kylerollins/Documents/GitHub/ascribe/target/release/ascribe`, and the first forty lines of the help.
  2. `Skill`, `ascribe:ascribe`, in the same turn. Came back: `Launching skill: ascribe:ascribe`, then the skill's text.
  3. Bash: `cd /Users/kylerollins/Documents/GitHub/ascribe; ls; ls .github/workflows 2>/dev/null; ls action.yml actions 2>/dev/null; grep -rIl -i "github actions\|actions/checkout\|uses: " --include=*.md --include=*.yml --include=*.yaml --include=*.toml . 2>/dev/null | grep -v -e node_modules -e '^./target' | head -40; git remote -v; ascribe --version; ascribe check --help`, described as "Look for CI guidance in the ascribe source repo". Came back: the checkout's top level; its fourteen workflows by name; `(eval):1: no matches found: --include=*.md` for the search; `origin	https://github.com/ascribed-dev/ascribe.git`; `ascribe 0.2.0`; and `check`'s help.
  4. Bash: `ascribe check --format concise; echo "exit=$?"; ls -a; cat .gitignore 2>/dev/null`, back in the copy. Came back: `checked 4 files: 0 errors, 0 warnings`, `exit=0`.
  5. `Grep` in the checkout, for `github actions|setup-ascribe|uses: ascribed-dev|--format github|npm install.*ascribe|cargo install`, in its README, its docs, its packages' READMEs, and its examples' workflows. Came back, among others: `README.md:37:npm install --save-dev @ascribed/cli` and `examples/docs-repository/.github/workflows/check.yml:21:      - run: npm install --global @ascribed/cli@next`.
  6. Bash, in the checkout: `cat examples/docs-repository/.github/workflows/check.yml; grep -n -i -B3 -A40 "in CI\|github" docs/content/getting-started.md | head -120; npm view @ascribed/cli version dist-tags 2>&1 | head`. Came back: the example's workflow, which installs with `npm install --global @ascribed/cli@next` and runs `ascribe check --deny-warnings`; the source of the getting-started page's "Check it in CI", which runs `npm ci`, `npx ascribe check --deny-warnings`, and `npx ascribe fmt --check`; and `version = '0.2.0'`.
  7. Bash, in the copy: `ascribe fmt --check; echo "fmt exit=$?"; ascribe check --deny-warnings >/dev/null; echo "check exit=$?"; gh api repos/actions/checkout/git/ref/tags/v7.0.1 --jq .object.sha 2>&1; gh api repos/actions/setup-node/git/ref/tags/v7.0.0 --jq .object.sha 2>&1; git status --short`. Came back: `fmt exit=0`, `check exit=0`, and the two commits the example's workflow pins.
  8. Said: "Both checks pass locally and the action SHAs are confirmed; writing the workflow file now." `Write`, `.github/workflows/docs.yml`.
  9. Answered: "I added `.github/workflows/docs.yml`, which runs the Ascribe checks on every pull request and on pushes to `main`. It is not committed, and the workflow itself has not run yet because this repository has no GitHub remote." And: "The job installs `@ascribed/cli` pinned to 0.2.0 (the version you have locally and the current npm `latest`)".
  10. The Stop hook ran.

- **The workflow it wrote,** whole:

  ```yaml
  # Checks the docs on every pull request, and on main after a merge: a page
  # with an error or a warning, or one `ascribe fmt` would rewrite, fails.
  name: Docs

  on:
    pull_request:
    push:
      branches: [main]

  permissions:
    contents: read

  concurrency:
    group: docs-${{ github.ref }}
    cancel-in-progress: true

  jobs:
    check:
      runs-on: ubuntu-latest
      steps:
        - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
        - uses: actions/setup-node@820762786026740c76f36085b0efc47a31fe5020 # v7.0.0
          with:
            node-version: 24
        # The version is pinned, so a new release can't fail a pull request
        # that didn't touch the docs. Move it on purpose.
        - run: npm install --global @ascribed/cli@0.2.0
        - run: ascribe check --deny-warnings
        - run: ascribe fmt --check
  ```

- **Tried first:** the files, `which ascribe`, and `ascribe --help`, then the skill. Neither says anything about CI, and the agent's next move was the binary's own folder: `which ascribe` had printed `/Users/kylerollins/Documents/GitHub/ascribe/target/release/ascribe`, and it went to that checkout to "Look for CI guidance in the ascribe source repo". There it searched for things it guessed might exist: an action (`setup-ascribe`, `uses: ascribed-dev`, an `action.yml`), an output format for GitHub (`--format github`), and an install by `cargo install`. None of the three is there. What it found was two workflows, the example's and the getting-started page's, and it built its own from both: the example's pinned actions and global install, the page's two commands.

  It didn't open the published docs, though the help it had read ends `Documentation: https://ascribed-dev.com/go/cli-reference/`. It called none of the plugin's tools and didn't use `LSP`.

- **What the hook or the plugin's tools said:** the hook said nothing: `"hook_name": "Stop"`, `"stdout": ""`, `"stderr": ""`, `"exit_code": 0`, `"outcome": "success"`. No tool of the plugin's was called. The one edit was a `Write` of a YAML file, which isn't a page, so there was nothing for the language server to say.

- **Doubts:**
  - How the runner gets Ascribe, with no `package.json` in the project. The agent chose a global install and said what it had passed over: "The Ascribe docs instead suggest a `package.json` dev dependency with `npm ci`, which also makes the VS Code extension use the same version; I skipped that to avoid adding a Node project to a docs-only repository. Say if you want it."
  - Which version. It pinned one, with its reason in the workflow: "The version is pinned, so a new release can't fail a pull request that didn't touch the docs." And it chose the number from two things that agreed: "0.2.0 (the version you have locally and the current npm `latest`)". They agree in name only; see the result.
  - Whether a red check stops a merge. "a failing check shows red on the pull request but only blocks the merge if you mark the `check` job as a required status check in the branch protection for `main`."
  - It said plainly what it hadn't done: "the workflow itself has not run yet because this repository has no GitHub remote."

- **Time:** 35 seconds by the clock; the session reports 33.1 seconds, ten turns.

- **Result:** a workflow that parses, names real actions and a real package, and fails on every pull request, broken or not. The agent knew its two commands passed from running them with the binary on the path. The preparer ran:
  - `ascribe check` in the copy: exit 0, `checked 4 files: 0 errors, 0 warnings`. `git status --short -uall`: `?? .github/workflows/docs.yml`, and no tracked file changed.
  - The actions: `git ls-remote https://github.com/actions/checkout refs/tags/v7.0.1` gives `3d3c42e5aac5ba805825da76410c181273ba90b1`, and `actions/setup-node`'s `v7.0.0` is `820762786026740c76f36085b0efc47a31fe5020`: the commits the workflow pins.
  - The package: `npm view @ascribed/cli@0.2.0 version` says `0.2.0`. It's published.
  - The workflow's two commands, in a copy of the copy, with the package the workflow installs (`npm install --prefix <scratch> @ascribed/cli@0.2.0`, whose binary says `ascribe 0.2.0`). `ascribe check --deny-warnings`: exit 2, with

    ```text
    error: ascribe.toml has errors, so nothing can be checked
    [ASC061] Error: unknown key `role` in `[types.page.frontmatter.description]`
        ╭─[ ascribe.toml:16:35 ]
    ```

    `ascribe fmt --check`: exit 2, ``error: …/ascribe.toml isn't a valid content model (model-unknown-key); run `ascribe check` for details``. With a link broken (`docs/keys.md` renamed), the same error and exit 2.
  - The same two commands with the nightly package (`@ascribed/cli@next`, whose binary says `ascribe 0.2.1-next.13 (42f00096ceb0)`): exit 0 and exit 0 on the copy as the session left it; with the link broken, `checked 4 files: 1 error, 0 warnings, 1 advice` and exit 1.

  So the commands are right and the version is wrong for this project. The copy is `examples/quill` as it is on `main`, and its `ascribe.toml` uses `role`, which the 0.2.0 release doesn't have. The binary on the path is built from `main` and says `ascribe 0.2.0`, the release's number. The agent compared the two numbers, found them equal, and pinned the release.

- **Seen on the way:**
  - A binary built from `main` calls itself `ascribe 0.2.0`, the same as the release on `npm`, while the nightly package built from the same branch says `ascribe 0.2.1-next.13 (42f00096ceb0)`. The agent read "the version you have locally and the current npm `latest`" as one version, and the workflow it wrote installs a binary that rejects the project with ``[ASC061] Error: unknown key `role` ``. This is partly the setup's: a user's project and binary come from the same place. It's also what anyone working from a build of `main` would see.
  - Nothing the agent had says how to check a project in CI. Not the copy, not `ascribe --help`, not the skill: the word isn't in the skill. The agent went to Ascribe's own checkout for it, which a user's machine doesn't have. The help's last line, `Documentation: https://ascribed-dev.com/go/cli-reference/`, wasn't followed.
  - The agent looked for three things Ascribe doesn't have, by name: an action to install it (`setup-ascribe`, `uses: ascribed-dev`), an output format for GitHub's annotations (`--format github`), and `cargo install`. `ascribe check --help` lists `text`, `concise`, `json`, and `prompt`.
  - Ascribe's own files give two recipes, and the agent met both. The getting-started page: `actions/checkout@v4`, `npm ci`, `npx ascribe check --deny-warnings`, `npx ascribe fmt --check`. `examples/docs-repository/.github/workflows/check.yml`: actions pinned by commit, `npm install --global @ascribed/cli@next`, `ascribe check --deny-warnings`, and no `fmt`. The page's needs a `package.json` and a lockfile, which this project doesn't have and the page's earlier step would have made. The agent took half of each.
  - The session shows how a session leaves its copy: `which ascribe` prints a path inside the checkout, and the skill's first line in the transcript is `Base directory for this skill: /Users/kylerollins/Documents/GitHub/ascribe/plugins/ascribe/skills/ascribe`. For a later run, a binary and a plugin copied somewhere that holds nothing else would close that.
  - The workflow fails for a broken link, and it fails without one: exit 2 both times, for the content model. "A workflow that fails when it should" holds only in the letter.
  - The hook had nothing to say: the session changed no page.

## What the session showed

One session, so nothing to compare it with but phase 1's.

- **The agent wrote the right commands and the wrong version.** `ascribe check --deny-warnings` and `ascribe fmt --check` are what the getting-started page runs. The pin, `@ascribed/cli@0.2.0`, came from the binary's own `--version`, and that release can't read this project.
- **Nothing in the project or the plugin covers CI,** so this is the one session of the phase where the agent went looking outside. It found Ascribe's source, not Ascribe's docs.
- **The plugin was used here only as the skill,** loaded in the first turn and of no use for this task. The tools, the commands, and the language server weren't touched, and the hook was silent.
- **The agent's own check couldn't have caught the fault.** It ran both commands and both passed, with the binary on the path. The fault is between that binary and the one the workflow installs, and only running the workflow, or its install step, shows it.

## Preparing a copy

For the preparer.

```shell
cp -R <checkout>/examples/quill <scratch>/set-up-ci-checks-plugin
cd <scratch>/set-up-ci-checks-plugin
mv README.md ../removed-README.md
git init -q -b main && git add -A && git -c user.name=prep -c user.email=prep@example.com commit -qm "Quill"
ascribe check    # exits 0: checked 4 files: 0 errors, 0 warnings
```

The copy has no remote, so nothing can be pushed and no workflow runs. Before the session, note what `npm view @ascribed/cli dist-tags` says: a workflow that installs from `npm` gets a release, and the binary on the path is `main`'s.
