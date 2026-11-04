---
type: troubleshooting
title: Troubleshooting
description: "What to do when `ascribe` won't install or start, a check in CI compared nothing, `ascribe diff` finds no merge base, the Astro build or `astro dev` fails, or an agent doesn't hear from Ascribe."
---

The problems you can meet installing Ascribe, checking in CI, building with Astro, and setting up an agent, by what you see. Each entry gives the text you see, its cause, and the fix. For a problem with review, see [Troubleshooting review](review/troubleshooting.md).

## `ascribe` can't find its native package
@id: native-package

`npx ascribe` or `pnpm exec ascribe` stops with:

```text
@ascribed/cli could not find its native package @ascribed/cli-linux-x64 (…). Reinstall @ascribed/cli for linux/x64, and ensure optional dependencies are enabled.
```

**Cause.** `@ascribed/cli` installs the binary for your platform as an optional dependency, and your package manager left it out. That happens with `--no-optional` or `--omit=optional`, with `optional=false` in `.npmrc`, or with a lockfile written on another platform. It also happens on a platform Ascribe has no binary for ([Platforms](install.md#platforms)).

**Solution.**

@steps
1. Turn optional dependencies back on: remove the flag or the setting that leaves them out.
2. Reinstall `@ascribed/cli` on the machine that runs it.
3. If that's a platform Ascribe doesn't support, install it [on your path](install.md#without-node) on one that is, such as a CI job on Linux.

## `ascribe` won't start on Linux: `GLIBC` not found
@id: glibc

On Linux, `ascribe` stops before it runs, with a message that a version of `GLIBC_` wasn't found.

**Cause.** Ascribe's Linux binaries need glibc {glibc} or later, and the system is older. A distribution built on musl, such as Alpine, has no glibc at all.

**Solution.** Run `ascribe` on a system with glibc {glibc} or later. In CI or a container, use a newer image, such as a current Debian or Ubuntu one. The build images of Netlify, Vercel, Cloudflare Pages, and AWS Amplify all have it ([Platforms](install.md#platforms)).

## ASC257 in CI: the addresses weren't compared
@id: addresses-base-unreadable

`ascribe check` in CI warns at `ascribe.addresses`, and `--deny-warnings` fails the job:

```text
the addresses weren't compared with the base: …
```

**Cause.** `ascribe check` compares the addresses the docs publish with the base's, and it couldn't read the base. The checkout has no history, as `actions/checkout` gives by default, or the base isn't there, as after a force-push ([`addresses-base-unreadable`](checks/diagnostics/project-checks.md#ASC257)).

**Solution.**

@steps
1. Give `actions/checkout` `fetch-depth: 0`, so the job has the history.
2. Name the base with `--base`: on a pull request, the branch it targets; on a push, the commit before the push.

[Keep the addresses you've published](checks/ci.md#keep-the-addresses-youve-published) has the workflow.

## `ascribe diff`: no merge base
@id: no-merge-base

`ascribe diff`, or `ascribe drift`, exits with `2` and one of these:

```text
error: this clone doesn't have enough history to find where the branch left `main`: fetch more of it (in GitHub Actions, `fetch-depth: 0` on actions/checkout), or pass --base-exact to compare with `main` itself
error: `main` and HEAD share no history, so there's no merge base to compare with; pass --base-exact to compare with `main` itself
```

**Cause.** `ascribe diff` compares with the point where your branch left the base, its merge base. A shallow clone doesn't have that commit, and two unrelated histories have none ([Exit codes](review/diff.md#diff-exit-codes)).

**Solution.** Do what the message says:

- **A shallow clone:** fetch the history. In GitHub Actions, give `actions/checkout` `fetch-depth: 0`.
- **No shared history:** pass `--base-exact`, to compare with the base itself.

## `astro build`: `site` doesn't match
@id: astro-site-mismatch

`astro build` or `astro dev` stops with:

```text
@ascribed/astro: ascribe.toml and astro.config disagree:
- [consumer] site is "https://example.com", but Astro's `site` is "https://docs.example.com".
```

The list can name `base-path`, `trailing-slash`, or Astro's `build.format` too.

**Cause.** Ascribe writes every link with `[consumer]`'s settings in `ascribe.toml`. A site whose routes differ would have broken links, so the build fails ([What the integration does](astro/reference.md#what-the-integration-does)).

**Solution.** Write each setting once, in `[consumer]`:

@steps
1. Remove `site`, `base`, and `trailingSlash` from `astro.config.mjs`. The integration gives Astro the values from `[consumer]`.
2. If `astro.config.mjs` must keep one, set it to the value the message names.

## `astro dev` answers 503
@id: astro-dev-503

Every page in `astro dev` says:

```text
Ascribe build failed; fix the source and save to retry.
```

**Cause.** The last rebuild failed. Until a save builds cleanly again, pages answer with HTTP 503, so you never see a stale page ([In `astro dev`](astro/reference.md#in-astro-dev)).

**Solution.**

@steps
1. Read the diagnostics in Astro's terminal, or run `ascribe check`.
2. Fix them, and save. The pages come back after the next clean build.

## The host doesn't list Ascribe's MCP tools
@id: mcp-tools-missing

Your agent's host lists no `ascribe_` tools, or says the `ascribe` server failed to start.

**Cause.** The host couldn't start `ascribe mcp`. Usually:

- The host hasn't read the new entry: it reads its servers when a session starts.
- `ascribe` isn't on the host's path. The entry `sync` writes runs the `ascribe` on the path.
- With `npx --no-install`, `@ascribed/cli` isn't in the `package.json` of the folder the host starts in, or of a folder above it.
- The host asks you to approve a project's servers, and you haven't.

**Solution.**

@steps
1. Start a new session.
2. Approve the `ascribe` server if the host asks.
3. Run `ascribe --version` in a terminal. If it isn't found, install it [on your path](install.md#without-node), or use [`@ascribed/cli` in the project](agents/mcp.md#with-ascribedcli-in-the-project).

[Set up the MCP server](agents/mcp.md) has the entries for each host.

## The hook says nothing
@id: hook-says-nothing

The agent writes a page with an error, and never hears about it.

**Cause.** The hook reports some things and not others ([What the agent hears](agents/hooks.md#what-the-agent-hears)). It says nothing:

- When the file isn't a page or a fragment of a project.
- About warnings, or advice that has no safe fix.
- When the check takes longer than 2 seconds after an edit, or 20 before the agent finishes: it's given up.
- Before the agent finishes, about changes the agent already committed, or when the agent is already continuing because of a stop hook.
- When no entry runs it: the project has no entries, or, in Codex, you haven't trusted the project's hooks.

**Solution.**

@steps
1. Run `ascribe agents sync --check` to see that the entries are there and current, and `ascribe agents sync --target claude --with-hook` to write them ([Add the hooks](agents/hooks.md#add-the-hooks)).
2. In Codex, trust the project's hooks when it asks.
3. Run `ascribe check <file>` on the page, to see what the hook would have found.
4. If checks time out, put the release's binary on your path. The npm package's launcher adds about 60 ms to each run.

## For more information

- [Install](install.md): the platforms, and every way to install `ascribe`.
- [Checks in CI](checks/ci.md): the workflow, and why it fetches the history.
- [`ascribe diff`](review/diff.md#cli-diff): what it compares.
- [The Astro integration](astro/reference.md): its options, and what it checks.
- [Set up the MCP server](agents/mcp.md) and [Check an agent's edits with hooks](agents/hooks.md): Ascribe for agents.
- [Troubleshooting review](review/troubleshooting.md): the page preview, the site preview, and the HTML preview.
