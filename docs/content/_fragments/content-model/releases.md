The release lines of a versioned target, by the target's name. A **line** is a version as the list writes it, and holds every version that starts with its parts: `3.4` holds `3.4`, `3.4.2`, and `3.4.2.1`. A project chooses how fine its lines are by how it writes them: `["3.4", "3.5"]` for a line for each minor release, `["0.2.0"]` for one for each patch release. The newest live line is the target's **current line**; it isn't written down, so it can't disagree with the list.

@snippet: code:examples/content-models/full.toml#releases
