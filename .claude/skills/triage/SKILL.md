---
name: triage
description: "Triage Ascribe's project board: decide which Backlog issues are ready for an implementor, move them to Ready, and ask on the rest. Use when a thread is started to triage the board, when the board routine fires, or when asked which issues are ready to work on."
---

# Triaging the board

Triage owns one move: **Backlog → Ready**. It reads every card in Backlog, moves the ones an implementor can start on without asking anything, and leaves the rest with a question. It doesn't start implementors; the coordinator does, from what triage reports.

[The `board` skill](../board/SKILL.md) says how to read the board and move a card.

## Steps

1. **Read the board,** and list the Backlog cards that are issues. Draft items and pull requests aren't triage's to move.
2. **Read each issue** with its comments, through the GitHub MCP tools or `gh api repos/ascribed-dev/ascribe/issues/N` and `.../comments`. Skip one whose last comment is a question from triage that nobody has answered.
3. **Decide whether it's ready.** It's ready when all of these hold:
   - it's one pull request's worth of work, or says which part comes first;
   - it says what done looks like: the behavior, the output, or the page that changes;
   - nothing it needs is waiting on another open issue or pull request;
   - it doesn't ask for a decision only the maintainer can make: a change to a contract (see AGENTS.md's "Contracts don't move"), a new dependency outside the repository, or anything in `project-docs/outside.md`.
4. **Move the ready cards to Ready,** all in one run of Board.
5. **Comment on each card that isn't ready,** once, with the one thing that would make it ready, phrased as a question the maintainer can answer in a line. Leave it in Backlog.
6. **Report, in the thread:** the cards moved to Ready, oldest first, each as `#N title`, then the cards left with a question. The coordinator starts implementors from that list, so put nothing else in it.

## Judgment

- Prefer leaving a card in Backlog with a good question to moving a vague one. An implementor who has to guess builds the wrong thing, and the review costs more than the question.
- A bug report with a reproduction is ready; the fix is the deliverable.
- Don't rewrite an issue to make it ready. Ask its author.
- Don't move a card back out of Ready, In progress, or In review. Those belong to other roles.
