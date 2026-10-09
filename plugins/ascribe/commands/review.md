---
description: "Review what this branch does to its pages, as readers see them, and report what reads wrongly."
argument-hint: "[path] [base]"
---

Run `ascribe agents prompt review [--arg path=<path>] [--arg base=<base>]`, and do what the prompt it prints asks.

Take the arguments from this, in this order, and leave out an optional one that isn't given: $ARGUMENTS

- `path`: A file or folder in the project; by default, the current directory.
- `base`: The git revision to compare with; by default, the default branch.

Without a shell, use the prompt `review` of Ascribe's MCP server instead.
