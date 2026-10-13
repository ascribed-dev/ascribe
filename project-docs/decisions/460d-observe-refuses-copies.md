---
section: The binary
---

**`ascribe observe` refuses, with exit 2, a folder inside `sources/<name>/` of a folder above it whose `ascribe.lock` pins a source named `<name>`; when that lock can't be read, everything under its `sources/` is refused.** It reads only the lock, never `ascribe.toml`, and says which source the folder is a copy of and where to run it instead. _Why:_ README decision 18 has `ascribe observe` never run a config that arrived as a copy, and the phase knows a copy "from the lock"; a lock that can't be read is still a sign the folder is Ascribe's, and refusing costs nothing there, since a config is never meant to be run from a docs project's copies. _From:_ the implementation of [issue #460](https://github.com/ascribed-dev/ascribe/issues/460), for Kyle's review in its pull request, 13 October 2026.
