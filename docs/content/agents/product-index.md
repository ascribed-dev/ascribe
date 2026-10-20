---
type: how-to
title: Index your docs for your product's agents
description: "Write a compact index of your docs into your product's `AGENTS.md`, so the agents of the people who use your product read them."
available: ascribe 0.3.0
---

Everything above is for the agents that write your docs. The agents of the people who use your product read them too, and `ascribe agents index` writes something for those: a compact index of the docs, one line a page, small enough to go in your product's own `AGENTS.md`. With `--pages`, it also copies each page's Markdown version into a folder, and the index points into it, so a package can ship the docs for the version it installs, with no server and nothing to fetch:

```shell
ascribe agents index --build site --into packages/sdk/AGENTS.md --pages packages/sdk/docs
```

It writes between markers, as `sync` does, so you keep the rest of the file, and running it again, in CI or before a release, replaces only the index. Write what the agent should do with it above the markers: Vercel's version said "Prefer retrieval-led reasoning over pre-training-led reasoning". The index holds to 8 KB, leaving out descriptions before pages, and says what it left out. Its pages come as `llms.txt`'s do, in the [navigation](../navigation/index.md)'s order: the content root's under **Start**, then a heading for each top section. See [`ascribe agents index`](commands.md#cli-agents-index) for each option.

The evidence for it is one vendor's. Vercel put an index of about 8 KB, of paths to local docs files, in `AGENTS.md`, and its evals on Next.js APIs newer than its model's training passed 100% of the time, against 53% with no docs; a skill that offered the same docs wasn't used in 56% of cases ([Vercel](https://vercel.com/blog/agents-md-outperforms-skills-in-our-agent-evals)). It's one framework, on evals whose number and model weren't published, and nobody has repeated it. The reason it makes sense is simpler than the number: an agent uses what's in front of it, and an index in `AGENTS.md` always is.
