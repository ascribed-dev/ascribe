# Writing release notes in Ascribe

Ascribe's notes on release notes, to read after their guide: only what's particular to writing them in an Ascribe project. They're yours to change, as the template is.

- **The pages say since when, and the release notes say what changed.** Mark each feature where it's documented, with its availability: `@available: quill 3.4` on its section, or a key in `[features]` when it'll change state, so a feature going from preview to GA is one edit in `ascribe.toml`. A deprecation is a state in that history, `quill (ga 3.0, deprecated 3.4)`, and the page itself is badged with it. The release notes then link to the section rather than repeating which versions have it.
- **Each entry links to its page by file:** `[](../guides/sync.md#streaming-sync)`. `ascribe check` reports the link when the page moves or the heading changes, so an old release's notes don't lead nowhere.
- **Before the release, read `ascribe report`.** Its builds section lists what each build leaves out, and why: whether what a release's readers won't see is what you meant. After it, set `[versions] current` to the release, and `ascribe check` reports each feature in `[features]` whose versions have all shipped (`availability-left-behind`), so its marks can name the release.
- **A known issue links to its fix.** Write the fix once, on a troubleshooting page, and link the release notes to its section.
- **Leave out a section with nothing in it,** heading and all, rather than keeping it empty.

Availability is in the [directives reference](https://ascribed-dev.com/docs/writing/directives/#available).
