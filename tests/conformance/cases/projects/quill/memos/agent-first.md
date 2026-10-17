---
about: install-agent.md#install-agent
quote: Install the agent package
---

The agent goes first because its installer writes the config file that
`quill init` reads. The other way round, `init` fails with an error that
doesn't mention the agent.
