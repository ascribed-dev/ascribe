Named builds ([SPEC §9.3]({repo}/blob/main/SPEC.md#93-build-modes)). Each sets a variant mode and an availability mode. These are [SPEC §9.3]({repo}/blob/main/SPEC.md#93-build-modes)'s examples, in `ascribe.toml`:

```toml
[builds.site]
variants = "switch"
availability = "badge"

[builds.cloud-pdf]
variants = { deployment = "cloud" }
availability = { filter = "cloud" }

[builds."sm-3.3"]
variants = "switch"
availability = { filter = "self-managed 3.3" }
```

`<name>` is the build name (§1.2). It names the build on the command line (`ascribe build --build cloud`) and in output paths.

| Key | Type | Default | Description |
|---|---|---|---|
| `variants` | string or table | `"switch"` | The variant mode. `"switch"` keeps every arm and page. A **selection** is a table from dimension names to a value or an array of values: `{ deployment = "cloud" }`, `{ pm = ["npm", "pnpm"] }`. Every dimension MUST be declared and every value a value of that dimension. An empty table is an error; write `"switch"`. |
| `availability` | string or table | `"badge"` | The availability mode. `"badge"` keeps everything and annotates it. `{ filter = "<target> [<version>]" }` removes content not available for the target at the version. The target MUST be a declared dimension value, not a dimension name. A versioned target MUST have a version, and a versionless one MUST NOT. The version follows the version scheme (§7). |
| `versions` | string | none | The target the build is **versioned** by, such as `"self-managed"`: one output for each of its live lines (§7.1). The target MUST have a `[releases]` table with at least one live line, and the build's `availability` MUST be `"badge"`. |

**Rules.** Build names MUST be unique ignoring case (`model-build-name-case`), since they become directory names and some file systems ignore case. A versioned build whose target has no live line, or that also filters, is an error (`model-build-versions`). A build that filters for a target its own selection excludes (for example, selecting `deployment = "cloud"` and filtering for `self-managed 3.3`) is legal but almost certainly a mistake, so the loader warns (`model-build-filter-excluded`).
