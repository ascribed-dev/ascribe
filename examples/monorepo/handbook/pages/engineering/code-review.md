---
title: Code review
description: How we review changes, and what reviewers look for.
owner: platform
---

Every change to `main` needs one approving review. Reviews are about the change, not the person who wrote it.

## As an author
@id: as-an-author

- Keep changes small. A reviewer can review 200 lines well; nobody can review 2,000.
- Say what the change does and why in the description, and how you tested it.
- Answer every comment, even if only to say you've done it.

## As a reviewer
@id: as-a-reviewer

- Review within one working day. If you can't, say so, so the author can find someone else.
- Say which comments block the change and which are suggestions.
- Approve when the change is better than what's there, not when it's perfect.

@note {type=policy}
Changes to authentication, billing, or data deletion need a second review from the owning team.
