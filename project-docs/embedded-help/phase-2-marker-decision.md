# Phase 2: Decide the section summary

Part of [Embedded help](README.md). Needs phase 1's measurements. A decision by the maintainer; no code.

## Goal

The maintainer settles how a section has a summary, so phase 3 can build it. Phase 3 changes the language, which is a contract (SPEC §4, Appendix A), so the syntax is settled first, not chosen while building.

## What the measurements say

Phase 1 ([#885](https://github.com/ascribed-dev/ascribe/pull/885)) built the docs' `help.json`: 326 entries. With the first-paragraph rule:

- **Diagnostics, 240 entries:** every summary is the metadata line ("Error · file level · next step: choose · SPEC §3.3"). 155 have a **When** paragraph (median 72 characters, 7 over 300); 85 have none, since `diagnostics.toml` gives them no `when`.
- **Long openings, 37 entries:** the first paragraph is over 300 characters (the longest 867), all command and guide sections. Their first sentence would do: median 185, 5 still over 300.
- **Lead-ins, 7 entries:** the first paragraph ends in ":" and introduces a list or code, as quill's `prerequisites` does ("Before you install the agent, make sure you have:").
- **No summary, 2 entries:** one section opens with a heading, one with a table.

So no rule over the prose picks well: the right summary is in the wrong paragraph for diagnostics, inside a paragraph for long openings, and spread over two blocks for lead-ins. A summary is something a writer, or an agent, decides.

## The recommendation

**A section summary, `@summary`, for every section, with `@note`'s two bindings** (SPEC §4: "self (with primary), else following block"):

```markdown
## `ascribe versions release`
@id {permalink=true}: cli-versions-release

@summary
Releases a line.

It rewrites each mark's `planned` for its target to the line's version, …
```

```markdown
## ASC149 `page-size`
@id {permalink=true}: ASC149
@summary: A page's Markdown, as a build writes it for agents to read, is as long as the limit or longer.
```

- **`@summary` with no primary** makes the next block the summary, and the block renders where it is: a writer picks a sentence the page already shows, and nothing is written twice. To pick a first sentence, the writer makes it a paragraph of its own.
- **`@summary: text`** gives the summary as its own text, which the page's body doesn't show, as a page's `description` isn't in its body: for when nothing on the page reads well alone, as for a lead-in or a diagnostic.
- **General, not help's:** any section may have one, with or without a permalink, as any page may have a description. Embedded help is its first reader; the JSON output, the editor's hover on a link to the section, search results, and a site's link previews can read it later.
- **Optional:** nothing requires one, except where a project asks: phase 4's check reports a permalink with no summary, as advice unless the project raises it.
- **No guessing:** from phase 3, `help.json`'s `summary` is the section's `@summary` or absent. Phase 1's first-paragraph rule goes.

The alternatives the measurements rule out: a container, `@summary:` … `@end`, which holds a lead-in and its list but adds a form no reader needs once the line form exists; an attribute on the heading naming a paragraph by its place (`summary=2`), which breaks when a paragraph is added; and an attribute, `{render=true}`, that switches whether the summary shows, which makes one attribute change both where the text comes from and whether it shows, where the two bindings say each in their own spelling.

## Also settled here

- **One summary per section.** A second is an error, at the second.
- **Where it goes:** the line form at the top of its section, after the heading and its `@id`, as `@id` is; the following form anywhere in the section, before the block it marks.
- **What the following form may mark:** a paragraph. Recommended: a paragraph only, since a summary is shown as a sentence or two, and a lead-in takes the line form.
- **What the line form's text may hold:** inline content, as `@note`'s text does: code spans, links, emphasis, and phrases.
- **A summary in a fragment** is the summary of the section it lands in, on each page that includes it, as a fragment's heading permalink is cut from its recorded page (README decision 7).
- **A summary inside a variant arm** is an error until phase 6, which can give each arm its own.
- **What each output writes:** the line form writes nothing in the site and plain outputs' bodies; the following form's block is written as it would be without it, with no element, and review's source anchors are its own (a sentence in `site-render.md`). The JSON output carries the section's summary on its heading: its text, and which form gave it.
- **The diagnostics:** the generator writes `@summary:` from each diagnostic's `when`, and each of the 85 with none gets a `when` in `diagnostics.toml`. A `**When:**` label never reaches a summary, so no rule for dropping one is needed.
- **The check:** a section with a permalink and no summary is advice, configurable in `[checks]`, so a project that only links to its permalinks isn't asked for summaries, and one that ships embedded help raises it.

## Output

A decision file in `project-docs/decisions/`, numbered by this phase's issue, with the form, each rule above, and why. Phase 3 starts from it. Nothing else changes.
