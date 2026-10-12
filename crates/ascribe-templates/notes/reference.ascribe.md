# Writing a reference in Ascribe

Ascribe's notes on the reference, to read after its guide: only what's particular to writing one in an Ascribe project. They're yours to change, as the template is.

- **What's generated doesn't belong in a hand-written page.** A table the code already declares, such as a command's options, a file's settings, or the error codes, goes stale the day the code changes. Generate it from the code into a fragment, a file whose name starts with `_`, and include it where it goes:

  ```markdown
  @include: ../_generated/cli-sync-options.md
  ```

  A test that fails when the fragment is stale keeps it current. Write by hand only what the code can't say: what a setting is for, and when to choose it.
- **A row that differs by release is marked in its first cell:** `` | `stream` {available="quill preview 3.4"} | … | ``. A badge build marks the row, and a filter build drops it where it isn't available. So a brace at the end of a first cell is read as the row's attributes: write it `\{` when it's text.
- **An example comes from tested code.** `@snippet: code:examples/quill.yaml#timeout` puts a region of a file your tests run into the page.
- **An entry the product links to gets a permalink.** `@id {permalink=true}: config-timeout`, under its heading, gives it an address that keeps working when the page moves or the heading is reworded, so a setting's help in the product can link straight to it.

The directives are in the [directives reference](https://ascribed-dev.com/docs/reference/directives/).
