---
name: review
description: "Review a pull request for a card that is In review on Ascribe's project board, then approve it with the Approve workflow and merge it once nothing blocking is left. Use when a thread is started to review a card, or asked to review a pull request from the board."
---

# Reviewing a card

A reviewer gets one card in **In review** and owns its last move: it merges, and GitHub moves the card to Done. Nobody else merges an agent's pull request, and the reviewer merges only what it approved.

[The `board` skill](../board/SKILL.md) says how to approve and merge.

## Steps

1. **Find the pull request:** the one that closes the card's issue. Read the issue, the description, and the whole diff.
2. **Check it against [AGENTS.md](../../../AGENTS.md)** and the checklist in [project-docs/checklists.md](../../../project-docs/checklists.md) that fits, and against what the issue asked for. Run what you need to believe it: the tests it adds, the command it changes.
3. **Write one review,** with a comment on each line that needs one. Mark each comment as blocking or not. Blocking means wrong, unsafe, against a rule in AGENTS.md, or missing what the issue asked for. A preference isn't blocking; say it once, or not at all.
4. **Wait for the implementor** when anything is blocking. Subscribe to the pull request; each push wakes you. Review the new commits, and resolve each thread the push fixed.
5. **Approve and merge** when nothing blocking is left, CI is green on the head, and there's no merge conflict:
   - run Approve with the head commit you reviewed, and check that its run passed;
   - merge with a merge commit, giving the same commit, so a push since your review stops the merge.

If the merge is refused because the head moved, review the new commits and approve again.

## Judgment

- Review what the pull request says it does, not the pull request you'd have written.
- A change to a contract, an output, or `project-docs/outside.md` that the issue didn't ask for is blocking until the maintainer agrees to it, in the issue or the pull request. Ask them there, and don't merge until they answer.
- Don't push to the implementor's branch. Say what to change.
