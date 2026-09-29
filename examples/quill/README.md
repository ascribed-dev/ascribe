# Quill

A complete, valid Ascribe project: the documentation set for Quill, a made-up docs-sync tool. Its centerpiece is the page from [SPEC Appendix B](../../SPEC.md#appendix-b-complete-example), `docs/install-agent.md`, which is byte for byte the page in the specification.

```
ascribe.toml                       the content model (SPEC Appendix B, plus image attributes)
docs/
  install-agent.md                 the Appendix B page
  quickstart.md                    has the `try-in-browser` id the page links to
  keys.md                          has the `rotate-keys` id the page links to
  playground.png                   an image beside quickstart.md
  _fragments/
    prerequisites.md               included by the page
    prerequisites.png              an image beside the fragment, used inside it
```

The project has no errors under any of its builds (`site`, `cloud`, and `self-managed-3.3`). The conformance suite has a copy as the project case `projects/quill`, and a test keeps the two identical (`tests/conformance/tests/quill_example.rs`).
