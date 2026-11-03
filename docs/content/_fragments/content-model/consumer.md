Which site generator publishes the site output, and how the output fits it: the **profile** ([SPEC §9.5]({repo}/blob/main/SPEC.md#95-consumer-profile)). Ascribe has one profile for spec 0.1, `astro` ([SPEC §9.6]({repo}/blob/main/SPEC.md#96-astro)). The profile sets defaults for every other key; a key given here overrides them, within the values the profile supports.

```toml
[consumer]
profile = "astro"
site = "https://docs.quill.dev"
base-path = "/"
trailing-slash = "always"
slugger = "github"
```
