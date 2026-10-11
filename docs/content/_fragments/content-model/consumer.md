The consumer profile ([SPEC §9.5]({repo}/blob/main/SPEC.md#95-consumer-profile)): how the site output fits a specific consumer. Spec 0.1's toolchain implements one profile, `astro` ([SPEC §9.6]({repo}/blob/main/SPEC.md#96-astro)). The profile sets defaults for every other key; a key given here overrides them, within the values the profile supports.

```toml
[consumer]
profile = "astro"
site = "https://docs.quill.dev"
base-path = "/"
trailing-slash = "always"
slugger = "github"
```
