@memo: Generated from builtin_schemas() in crates/ascribe-core/src/schema.rs by crates/ascribe-cli/src/docs/builtins.rs; edit the schema, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli docs` to rewrite it.
| Directive | Forms | Primary | Purpose |
|---|---|---|---|
| [`@id`](../writing/directives.md#id) | line | an id | Give a heading a stable id |
| [`@include`](../writing/directives.md#include) | line | a path | Transclude a file or a region |
| [`@variant`](../writing/directives.md#variant) | container, grouped | none | Mark alternative content |
| [`@available`](../writing/directives.md#available) | line | an availability spec or feature key | Declare where content applies |
| [`@note`](../writing/directives.md#note) | line, container | text, optional | Callout |
| [`@steps`](../writing/directives.md#steps) | line | none | Mark an ordered list as a procedure |
| [`@details`](../writing/directives.md#details) | line, container | none | Collapsible content |
| [`@snippet`](../writing/directives.md#snippet) | line | an address | Take a code example from a file |
| [`@intended`](../writing/directives.md#intended) | line | a reason | Acknowledge a check's problem in the next block, or at the section's heading |
| [`@memo`](../writing/directives.md#memo) | line, container | text | Text for whoever edits the page next, never published |
| [`@term`](../writing/directives.md#term) | line | none | Make a glossary page's heading a term |
