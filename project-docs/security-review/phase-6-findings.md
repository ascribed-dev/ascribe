# Phase 6: Findings

Part of [Security review](README.md). Needs phases 1 to 5.

## Goal

The maintainer reads every finding and says which are fixed, in what order, and which are accepted, so the fixes can start.

## What's read

- **The draft advisories,** each with its input, what it achieves, and a severity (CVSS, or critical, high, medium, low with a sentence saying why).
- **The phases' notes,** each hardening note with its recommendation and size.

## Decisions here

- For each advisory: fix now, in a private fork the advisory creates, released before the advisory is published; or accept, with the reason recorded in the advisory.
- For each hardening note: an issue on the board, or accepted, with the reason in the notes file.
- Whether the fuzz job and a JavaScript audit become required checks or stay reports.

## Output

The advisories updated, an issue per hardening note that's kept, and `notes/summary.md`: what was reviewed, how many findings of each severity, what's fixed, and what's accepted, without detail of anything not yet released. Each fix is then its own pull request with its regression test (README decision 5).
