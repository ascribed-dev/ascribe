---
section: The binary
---

**The install script lives in this repository, at `site/public/install.sh`, and is served with the docs site at `https://ascribed-dev.com/install.sh`.** `curl -fsSL https://ascribed-dev.com/install.sh | sh` installs the latest release on Linux and macOS, and it changes only by a pull request here, reviewed like any change. _Why:_ it's an address people pipe into a shell, so it's served from the one domain Ascribe already has, and reviewed where the code is; a raw file on GitHub has a longer address that names a branch. _From:_ the maintainer, 13 October 2026, in the tested examples plan's phase 6 ([#463](https://github.com/ascribed-dev/ascribe/issues/463)), taking the phase's recommendation.
