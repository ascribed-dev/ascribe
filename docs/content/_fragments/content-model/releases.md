The release lines of a versioned target, by the target's name. A **line** is a version as the list writes it, and holds every version that starts with its parts: `3.4` holds `3.4`, `3.4.2`, and `3.4.2.1`. A project chooses how fine its lines are by how it writes them: `["3.4", "3.5"]` for a line for each minor release, `["0.2.0"]` for one for each patch release. The newest live line is the target's **current line**; it isn't written down, so it can't disagree with the list.

@snippet: code:examples/content-models/full.toml#releases

The target is a versioned value of a declared dimension, such as `self-managed`, or a name no dimension has. `[releases.<target>]` declares a name no dimension has as a value of the **implicit dimension** `release`, which exists only while it has a value. `[releases] label` is the dimension's label, `Release` unless set, and `[releases.<target>] label` is the target's, its name unless set:

```toml
[releases.ascribe]
label = "Ascribe"
live = ["0.4.1"]
```

Everything that names a dimension names `release` as it names a declared one: `@variant {release=ascribe}`, a switcher's `sync`, a build's `variants`, `dimension:` targets in `ascribe refs`, and the elements' `dimension` attribute. It comes after the declared dimensions in their order.
