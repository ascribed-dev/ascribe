# An agent asked for a how-to

Part of [Page templates](../README.md), [phase 4](../phase-4-start.md): its acceptance criterion that an agent with the plugin, asked for a how-to, reads the guide and writes a page with no placeholder left, in the surface audit's form ([journey 1's flags](../../surface-audit/journeys/fix-a-broken-link.md#what-changed-in-these-two-commands-and-why)). The session ran twice, once on a draft of [#406](https://github.com/ascribed-dev/ascribe/issues/406)'s prompt and once on what the pull request ships, since the first changed what the prompt does with `path`. Both transcripts are here.

## The session

```shell
claude -p "Add a how-to page on checking a release's signature before installing it." \
  --setting-sources project \
  --plugin-dir <checkout>/plugins/ascribe \
  --allowedTools "Read,Edit,Write,Glob,Grep,Bash,mcp__plugin_ascribe_ascribe__*" \
  --no-session-persistence --include-hook-events \
  --output-format stream-json --verbose \
  > new-page-how-to-plugin-<n>.jsonl
```

**Starting point:** a fresh copy of `examples/templates`, without its `README.md`, as a git repository with one commit: the six types, their templates, guides, and notes in `templates/`, and one filled page of each type about Quill, a made-up tool. `ascribe check` reported nothing in it, advice included. The request is the preparer's, and asks for a page the copy has facts for only in part: the concept page says a signature is a file beside each release that anyone with the public key can check, and no page names the command that checks it.

**When, with what:** 12 October 2026, Claude Code 2.1.296, `claude-sonnet-5-5`, in a cloud container. `ascribe` on the path was the debug build of the branch, `ascribe 0.3.0+dev`. The plugin was loaded from the branch's `plugins/ascribe`, with its MCP server connected; standard error was empty both times.

## First, on the draft (`new-page-how-to-plugin-1.jsonl`)

The draft's `path` still named only the project, as it had before.

1. The agent ran the plugin's `/ascribe:new-page` with the request.
2. `ascribe model --section types`, and it chose `how-to`.
3. `ascribe agents prompt new-page --arg type=how-to --arg title="Check a release's signature before installing" --arg path=how-to/check-release-signature.md`. The plugin's command tells it to pass a file or folder for the page as `path`; the prompt took it as a place in the project and put the page at `docs/how-to/check-a-releases-signature-before-installing.md` instead.
4. `cat templates/how-to.guide.md templates/how-to.ascribe.md`: it read the guide and Ascribe's notes, by the paths the prompt gave, before writing.
5. It read the concept, troubleshooting, and reference pages, then wrote the page at the prompt's path, from the template: the overview, before you start, four steps under `@steps` with a warning `@note`, and see also. No placeholder or instruction of the template was left, and `ascribe check docs` reported only that no page linked to it.
6. It linked the page from `docs/index.md`; `ascribe check docs` then reported nothing. Ten turns, 29 seconds.

It said plainly that step 3 of its page names no command, since the docs don't say how a signature is checked, and asked for the command rather than make one up.

**What changed because of it:** a `path` in the content root is now where the page goes, a file or a folder, from the current folder or, when nothing is there, from the content root, so what the plugin's command asks the agent to pass is what the prompt uses (decision 46 in the plan's README).

## Then, on the pull request's prompt (`new-page-how-to-plugin-2.jsonl`)

1. `/ascribe:new-page`, then `ascribe model --section types`, as before.
2. `ascribe agents prompt new-page --arg type=how-to --arg title="Verify a release signature before installing" --arg path=how-to/verify-release-signature.md`. The prompt answered `Where: docs/how-to/verify-release-signature.md.`, the path it was given, from the content root, and sent it to the guide first.
3. `cat templates/how-to.guide.md templates/how-to.ascribe.md`: it read both.
4. It read the how-to, concept, and troubleshooting pages, the reference, and `ascribe.toml`, and searched the copy for a verify command, a signature file's name, and where the public key is.
5. It wrote nothing, and said why: the docs don't name the command that checks a signature, the signature's file, or where the key is published, and the guide asks for steps that are tested and accurate. It named the path and title it would use and the pages it would link from, and asked for the command or the code. Eight turns, 20 seconds. `ascribe check` afterwards reported nothing, and `git status` was clean.

## What it shows

Both times the agent went to the plugin's command, which sent it to the type and then the prompt, and both times it read the guide and the notes, at the paths the prompt named, before anything else. The first wrote a page with nothing of the template left. The second, given the same request, stopped short of writing because the facts weren't in the copy, which the first had worked around by leaving its third step without a command. Neither left a placeholder behind, and the request, not the prompt, is why the second has no page.
