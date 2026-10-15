---
description: "Write a new page from a request: the page type that fits it, where its file goes, the guide to read first, and the type's template to start from."
argument-hint: "<what the page is for>"
---

The page to write: $ARGUMENTS

1. Run `ascribe model --section types`, and choose the page type that fits the request. Choose the page's title, and, when the request says where the page goes, a file or folder for it.
2. Run `ascribe agents prompt new-page --arg type=<type> --arg title=<title> [--arg path=<path>]`, and do what the prompt it prints asks.

Without a shell, call the MCP server's tool `ascribe_model` for the page types, then use its prompt `new-page`.
