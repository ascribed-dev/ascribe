---
section: Process
---

**Triage moves only issues opened by an account listed in `.github/triage-authors.txt`, or labeled `triage approved` by the maintainer; any other open Backlog issue is labeled `needs human` and left to the maintainer.** `board.yml` gives each open issue's `author` and `approved` in `board.json` and applies and removes the label; the `triage` skill doesn't read, comment on, or move an unapproved issue, and lists it in its report for the coordinator to pass on. An account is added to the list by a pull request. _Why:_ an issue's text is what an implementor builds, so work from an account nobody vouched for gets a person's look before an agent acts on it. _From:_ Kyle, in the project thread that asked for it (pull request #1399), 19 November 2026.
