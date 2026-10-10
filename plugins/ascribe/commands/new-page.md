---
description: "Write a new page of a type: the frontmatter the type requires, and where its file goes."
argument-hint: "<type> <title> [path]"
---

Run `ascribe agents prompt new-page --arg type=<type> --arg title=<title> [--arg path=<path>]`, and do what the prompt it prints asks.

Take the arguments from this, in this order, and leave out an optional one that isn't given: $ARGUMENTS

- `type`: The page type, as `ascribe model` lists them.
- `title`: The page's title.
- `path`: A file or folder in the project; by default, the current directory.

Without a shell, use the prompt `new-page` of Ascribe's MCP server instead.
