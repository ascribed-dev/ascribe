# Security policy

Ascribe parses documentation written by other people. `ascribe check`, `ascribe build`, and the language server all read untrusted Markdown, and the language server and the VS Code extension run inside your editor. Reports about crashes, hangs, or unbounded memory on crafted input, path escapes out of a project's content root, and anything in the extension that runs project code unexpectedly are all in scope.

## Supported versions

Security fixes go into the latest release. Ascribe is pre-1.0, so there are no backports.

## Reporting a vulnerability

Please report it privately. Don't open a public issue.

Use [GitHub's private vulnerability reporting](https://github.com/ascribed-dev/ascribe/security/advisories/new) on this repository. Include the Ascribe version, your platform, and the smallest input that reproduces the problem.

You'll get a reply within a week. Once a fix is released, the advisory is published with credit to you, unless you'd rather stay anonymous.
