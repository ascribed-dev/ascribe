```toml
[from."../../docs/ascribe.toml"]
take = ["releases", "dimensions.pm", "phrases.version", "consumer.site"]
```

The key after `from` is the other file's path, relative to the folder `ascribe.toml` is in. A project can take from more than one file, each a table of its own.

@include: ../../_generated/content-model-from.md

What's taken is read from the other file as if written in this one, and every rule of this reference holds it there. A source's `path` is the one exception: it's relative to the folder of the file it's written in, so a taken source is the same folder from either project. The other file's own `[from]` isn't followed: what's taken is always written in the file named.
