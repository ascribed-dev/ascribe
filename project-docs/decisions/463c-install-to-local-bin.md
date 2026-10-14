---
section: The binary
---

**The install script puts `ascribe` in `~/.local/bin` and never asks for `sudo`.** `--to <folder>` chooses another folder, and the script says how to add the folder to `PATH` when it isn't there. _Why:_ a script that writes to `/usr/local/bin` needs a password on most machines, and one piped from the web shouldn't ask for it. _From:_ the maintainer, 13 October 2026, in the tested examples plan's phase 6 ([#463](https://github.com/ascribed-dev/ascribe/issues/463)), taking the phase's recommendation.
