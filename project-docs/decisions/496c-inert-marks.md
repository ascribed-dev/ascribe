---
section: The language
---

**A mark does nothing only when no build shows its version: `available-same-in-every-line` and `available-in-no-line` look at the live and planned lines, not the archived ones, and stay quiet while any build shows the mark.** A build badging availability shows it, as does a build filtered for the target at a version before the window, and a build filtered for another target when the mark names that target or its dimension. A version older than every declared line is before the window and reports nothing, so a project needn't declare its old releases. The checks for a version's place and for an inert mark run only on a spec with no other problem. _Why:_ an archived line is read elsewhere, at its `url`, so a mark that only matters there is still not this site's to report; a site that shows badges uses every version it shows. _From:_ the implementor of [#496](https://github.com/ascribed-dev/ascribe/issues/496), in its pull request, 15 October 2026, for the maintainer's review.

Narrowed by [decision #667](667-old-marks-in-badges.md): a build that shows a mark as a badge no longer keeps `available-same-in-every-line` quiet.
