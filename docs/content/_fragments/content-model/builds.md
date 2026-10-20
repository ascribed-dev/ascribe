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

@include: ../../_generated/content-model-builds.md

**Rules.** Build names MUST be unique ignoring case (`model-build-name-case`), since they become directory names and some file systems ignore case. A versioned build whose target has no live line, or that also filters, is an error (`model-build-versions`). A build that filters for a target its own selection excludes (for example, selecting `deployment = "cloud"` and filtering for `self-managed 3.3`) is legal but almost certainly a mistake, so the loader warns (`model-build-filter-excluded`).
