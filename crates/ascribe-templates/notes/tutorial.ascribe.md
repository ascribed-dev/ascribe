# Writing a tutorial in Ascribe

Ascribe's notes on the tutorial, to read after its guide: only what's particular to writing one in an Ascribe project. They're yours to change, as the template is.

- **The code a reader types comes from a project that runs.** A reader can't recover from a step that doesn't work. Keep the tutorial's finished project in your repository, where CI builds or runs it, and take each file into the page by region:

  ```markdown
  @snippet: code:examples/first-sync/quill.yaml#remote
  ```

  The page then shows what the project has, and a change that breaks the project breaks its CI, not the reader.
- **Each list of steps is a procedure.** Write `@steps` directly above the numbered list, so the site and agents read it as one.
- **One path, with the platform as tabs.** A tutorial follows one path. Where only a step's command differs by platform, write a variant group in that step, and the site shows the arms as tabs, keeping the reader's choice on every page.
- **Versions are phrases.** Write `You need {product} {version} or later`, with both in `[phrases]`, and the next release changes one line in `ascribe.toml` rather than every page.
- **Output to expect comes from a run too.** When your tests record a command's output, take it with `@snippet` rather than pasting it.

The directives are in the [directives reference](https://ascribed-dev.com/docs/reference/directives/).
