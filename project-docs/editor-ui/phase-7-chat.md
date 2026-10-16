# Phase 7: Chat, drop, and paste

Part of [The editor's views](README.md). Needs nothing.

## Goal

VS Code's chat has Ascribe's skill in a project that never synced it, and the docs say how dropped images land.

In VS Code, open a project that has never run `ascribe agents sync`. Chat lists Ascribe's skill. Run `ascribe agents sync`, and chat lists one copy, the project's. Drop an image from the Finder into a page: VS Code's Markdown extension writes `![…](…)`, and `ascribe check` reports nothing new, or the plan's later drop handler is due.

## Context

- README decisions 7, 8, and 16.
- The contribution point `chatSkills`, which takes a `path` to a `SKILL.md` and an optional `when`.
- The skill: what `ascribe agents skill` prints, and where `@ascribed/cli` ships it (`skills/ascribe/`). Its one home, and how the extension's copy is made from it at build time, so it's never written twice.
- `ascribe agents sync` and its targets: `.agents/skills/ascribe/` and `.claude/skills/ascribe/`.
- Which folders VS Code's chat loads skills from, and the setting that names them, in the VS Code version the extension requires (`engines.vscode`).
- VS Code's Markdown extension: drop and paste into a Markdown file, `markdown.editor.drop.*`, `markdown.editor.filePaste.*`, and `markdown.copyFiles.destination`.
- The image and link checks `ascribe check` runs on what's written: relative paths, links to files, and alt text.
- The docs: `docs/content/agents/index.md` (agents in VS Code, and the skill), and the editor page's writing sections.

## Design

### The skill

- **The extension ships the skill** as a file it builds from the skill's one home, and declares it under `chatSkills` with `when: ascribe.active && !ascribe.agentsSynced`. A test fails when the shipped copy and the one home differ.
- **`ascribe.agentsSynced`** is true when the synced `SKILL.md` exists in a folder chat loads skills from, for any workspace folder, and is set again when such a folder is created or deleted. If neither of `sync`'s folders is one chat reads, the key is never true, and the docs say chat uses the extension's copy.
- **Nothing is written** to the project.

### Drop and paste

A test drops and pastes, through the Markdown extension's commands, an image and a page into a page of a fixture, with the settings at their defaults, and runs `ascribe check` on the result. Where it reports nothing new, the editor's page says how to set `markdown.copyFiles.destination` for a project's images folder. Where it reports something, the phase records which diagnostic in the README's "Later" and stops there: the drop handler is a plan of its own.

## Tasks

1. Which folders chat loads skills from, checked in the required VS Code version, and written in the pull request.
2. The shipped skill, built from its one home, its test, and the `chatSkills` contribution.
3. `ascribe.agentsSynced`, and its file watching, with unit tests.
4. The drop and paste test, and its result.
5. The docs: the agents page's VS Code section (the skill in chat, when it's the extension's copy and when the project's); the editor page's drop and paste, with the setting, if the test passes; `changes/agents/skill-in-chat.md`; and the surface map.

## Out of scope

- Shipping the rules, which are a project's own (decision 16).
- A chat participant.
- A drop or paste handler of Ascribe's own.

## Acceptance criteria

- In an integration test, `ascribe.agentsSynced` is false in a fixture with no synced copy and true in one with a copy in a folder chat reads. By hand, chat lists the skill once in each, and the pull request says so.
- The shipped `SKILL.md` is byte for byte the one home's.
- The drop and paste test's result is in the editor's page or in the README's "Later".

## Verify

```sh
pnpm --filter ascribe-vscode test
```

The integration tests run in CI.

## Commits

1. "Give VS Code's chat Ascribe's skill where the project has none"
2. "Say where a dropped image lands"
