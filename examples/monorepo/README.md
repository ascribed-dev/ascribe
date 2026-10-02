# A repository with several projects

A made-up repository for Lantern, a feature-flag service, laid out the way a real one often is: code, and documentation beside it, in more than one Ascribe project. It's for trying the VS Code extension with several projects in one workspace: **Run and Debug → Extension: several projects** opens it (see [packages/vscode/DEVELOPMENT.md](../../packages/vscode/DEVELOPMENT.md#running-it-locally)).

```
service/                         Go code: no ascribe.toml, so no project
docs/                            the public docs, a project
  ascribe.toml                   guides and reference pages, platform and edition
                                 variants, features, a glossary, two builds
  content/
    index.md, getting-started.md
    guides/                      flags, rollouts, self-hosting
    reference/                   CLI, configuration, glossary
    _fragments/                  included by several pages
handbook/                        the engineering handbook, a project
  ascribe.toml                   pages with a required `owner`, a `policy` note
                                 type, the `team-contact` widget
  pages/
    onboarding/, engineering/, on-call.md, _fragments/
    security/                    the security handbook, a project nested in
      ascribe.toml               the handbook's content root: policy pages, a
      content/                   case-sensitive glossary, its own phrases
```

Each project has no errors or warnings under any of its builds, and its files are in canonical form; tests in `crates/tessera-cli` keep it that way.

## Things to try

- **Servers start on demand.** Open the workspace and nothing runs until you open a page. Open `docs/content/index.md`, then `handbook/pages/index.md`, and watch each project's output channel appear (**Ascribe: Show Server Output**).
- **Each file has one project.** A file belongs to the nearest `ascribe.toml` above it. The handbook skips `pages/security/`, because that folder holds its own `ascribe.toml`. So a security page gets only the security project's diagnostics, completions, and phrases. Hover `{report}` in `security/content/_fragments/report.md`, then try typing `{` in a handbook page: each completes its own project's phrases.
- **Links don't cross projects.** In `handbook/pages/index.md`, link to `security/content/incident-response.md`: it's reported as missing, because for the handbook it isn't a source.
- **The preview follows the active file.** Preview `docs/content/getting-started.md` and switch its build to `self-hosted`: the edition tabs reduce to the self-hosted arm. Then preview a handbook page: the build picker has only its one build, and the docs page keeps its choice when you come back.
- **Files outside a project.** Preview this README or `docs/README.md`: one isn't in any project, the other is in the docs project but outside its content root, and the preview says which.
- **Each project's own model.** Remove `owner:` from a handbook page, or set `review: monthly` in a security policy, and see each project's schema at work.
