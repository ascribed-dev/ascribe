# Spec questions

[SPEC.md](../SPEC.md) is normative, but it has gaps. This file records every place where an implementing agent found the spec ambiguous or silent, so that a human can resolve it. The protocol is in [phases/README.md](phases/README.md#when-the-spec-is-unclear):

1. **Don't guess silently.** Add an entry below with the section, the ambiguity, the options, and your proposed resolution.
2. **Implement the most conservative option**: the one that reports an error or keeps content rather than dropping it.
3. **Mark the code** with a `// SPEC-QUESTION(Qn)` comment pointing at the entry, and tag any conformance case that depends on it `provisional`, listing the entry in the case's `questions` (see [tests/conformance/README.md](../tests/conformance/README.md)).
4. **Keep going.** Don't edit SPEC.md. A human resolves the question, updates the spec, and records the resolution here; then the `provisional` tags come off.

Contract changes (phase 02) also go through this file: a change to a contract needs an entry approved by a human, listing every phase it affects.

## Entry format

Number entries in order (`Q1`, `Q2`, …) and never reuse a number.

```markdown
### Q<n>: <short title>

- **Section:** SPEC §<n.n>
- **Raised by:** phase <NN>
- **Status:** open | resolved (<date>) | withdrawn
- **Ambiguity:** what the spec doesn't settle, with a minimal example.
- **Options:** the plausible readings, with their consequences.
- **Proposed resolution:** which option, and why. Note which one is implemented now.
- **Affects:** code (`SPEC-QUESTION(Q<n>)` locations), conformance cases, contracts, phases.
- **Resolution:** filled in by a human.
```

## Questions

_None yet._
