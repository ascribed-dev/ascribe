Two projects, `docs/` and `api/`, whose sources share a folder, `code/`:
each has `[sources.code]` with `path = "../code"`, and pages that show
`greet.py`'s region `greet`. Only `docs/` shows `farewell.py`'s region
`farewell` and the whole of `notes.md`, a Markdown file. The integration
suite `shared` makes the workspace a git repository, so the sources may be
outside each project.
