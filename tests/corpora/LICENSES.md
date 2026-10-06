# Licenses of the corpora

Nothing from a corpus is committed to this repository: `ascribe-corpora`
fetches each at a pinned commit into `target/corpora/` (or
`ASCRIBE_CORPORA_DIR`) when a test or benchmark needs it. This file records
what each repository's license allows, as the task asks, **before any committed
fixture uses a corpus's content**. Today no fixture does: the hand-written
fixtures in `tests/edge.rs` and `tests/conversion.rs` are ours.

| Corpus | Repository, pinned commit | License | Where it says so | May content (or a conversion of it) be committed here? |
|---|---|---|---|---|
| Astro | `withastro/docs` @ `e750d92a8e8309e5a968e3450403c8ccb884701d` | MIT | `LICENSE` at the root; the repository states no separate license for its content | Yes, with the copyright and permission notice kept alongside |
| Docker | `docker/docs` @ `3633800c79c473180d51ff64f7dffb05ccfb92c9` | Apache-2.0 | `LICENSE` at the root; `README.md`: "released under the Apache 2.0 license" | Yes, with the license, and a note of any change |
| Elastic | `elastic/docs-content` @ `651711d37722c1fcad9623d46e4a34eeeedfa008` | CC BY-NC-ND 4.0 | `LICENSE` at the root; `README.md`: "licensed under a Creative Commons Attribution-NonCommercial-NoDerivs 4.0 International License" | **No.** NoDerivatives forbids sharing adapted material, and a conversion to Ascribe is an adaptation; NonCommercial is a second restriction |

What follows from that:

- **Elastic's text and any conversion of it stay out of the repository and out
  of committed baselines.** What is committed about it is aggregate: counts per
  class (`baselines/recognition-elastic.json`), paths in nothing, and
  no excerpts. `recognize` keeps a source line in memory to help a person read a
  report; nothing writes it.
- Running the converters over Elastic's checkout on a developer's machine or
  in CI, to measure them, doesn't share anything. It uses the text as a
  reader would. If that reading is ever in doubt for CI, the Elastic tests are
  the ones to turn off (leave `ASCRIBE_CORPORA` unset); nothing else depends on them.
- If a small Astro or Docker excerpt becomes a committed fixture, copy its
  license text next to it and name the file it came from.
- This is a record of what the repositories say, not legal advice.
