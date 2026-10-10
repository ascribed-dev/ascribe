# Directives

A directive is a line that starts with `@`. Each built-in one, with its syntax:

- `@id: <id>` on the line under a heading: an id for links to it that doesn't change with its text
- `@include: <file>.md`, or `@include: <file>.md#<id>` for one section: another file's content, in place
- `@variant {<dimension>=<value>}:` before each arm, and one `@end` after the last: content that differs by a dimension
- `@available: <spec>` under a heading or above a block: where it applies, as targets with an optional state and version (`cloud, self-managed preview 3.4`), or a feature's key
- `@note {type=<type>}: <text>`, or `@note:` … `@end` around several blocks: a callout
- `@steps` on the line above an ordered list: the list is a procedure
- `.<Title>` on its own line, then `@details:` … `@end`: collapsible content; the title is required
- `@snippet: <source>:<path>#<region>`: a code example from a file outside the pages
- `@intended {check=<check>}: <reason>` above a block: a review check's problem in it is intended; write one only when the user says the problem is intended

A title line, `.<Title>`, goes on the line above a directive that takes one. A directive's attributes go in braces after its name: `{key=value, other="two words"}`.

A project can declare its own directives, widgets, whose names have a hyphen: `ascribe model --section widgets` lists them, with their attributes. Which note types, dimensions, and sources a project has: `ascribe model`.

The full reference, with examples: https://ascribed-dev.com/reference/directives/
