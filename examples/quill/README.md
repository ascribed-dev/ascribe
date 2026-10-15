# Quill

A complete, valid Ascribe project: the documentation set for Quill, a made-up docs-sync tool. Its centerpiece is the page from [SPEC Appendix B](../../SPEC.md#appendix-b-complete-example), `docs/install-agent.md`, which is byte for byte the page in the specification.

```
ascribe.toml                       the content model (SPEC Appendix B, plus image attributes)
docs/
  install-agent.md                 the Appendix B page
  quickstart.md                    has the `try-in-browser` id the page links to
  keys.md                          has the `rotate-keys` id the page links to
  webhooks.md                      a page only self-managed 3.5 has, linked from a section of keys.md marked the same
  playground.png                   an image beside quickstart.md
  _fragments/
    prerequisites.md               included by the page
    prerequisites.png              an image beside the fragment, used inside it
```

The project has no errors under any of its builds (`site`, `cloud`, and `self-managed-3.3`). The `site` build is versioned: `[releases.self-managed]` has two live lines, so it's an output for each, `site` for 3.5, the current line, and `site@3.4` for 3.4, under `/3.4/`, without the webhooks page. The conformance suite has a copy as the project case `projects/quill`, and a test keeps the two identical (`tests/conformance/tests/quill_example.rs`).
