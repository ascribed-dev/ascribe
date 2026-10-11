Settings for the authoring environment (SPEC §10).

@snippet: code:examples/content-models/full.toml#editor

| Key | Type | Default | Description |
|---|---|---|---|
| `build` | string | see below | The build whose page-level diagnostics the language server reports by default ([SPEC §8.1]({repo}/blob/main/SPEC.md#81-validation-levels) checks pages once per build). It MUST name a declared build. |

**Default.** If only one build exists, that build. Otherwise, the build named `site`, if there is one. Otherwise the key is required (`model-editor-build-required`).
