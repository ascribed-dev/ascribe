---
section: The look
---

**What Ascribe says it is lives in `design/words.toml`: the headline, the sentence, its short form, the keywords, and the maker.** Every surface that says what Ascribe is takes them from there or is compared with them by `scripts/design/words.test.ts`: the README, getting started, the landing page, the social card, the npm packages and the Marketplace listing, the command's help, and the plugin's manifests. The binary takes the short form from `ascribe-cli`'s crate description, which the test compares. _Why:_ the words were written a different way on each surface; one home, in a file the scripts and tests can read, keeps them the same. _From:_ [the website plan](https://github.com/ascribed-dev/ascribe/blob/58afc492/project-docs/website/README.md#decisions), decision 12, carried out in #815.
