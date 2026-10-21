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
