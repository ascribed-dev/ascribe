---
title: Install the Quill agent
description: Install and configure the Quill agent to sync your docs to Quill Cloud or a self-managed Quill server.
available: cloud, self-managed preview 3.3
---

The {product} agent watches your docs repository and syncs changes to {product}. This page covers installing the agent with a package manager, configuring it, and connecting it to {cloud} or a self-managed server. If you only want to try {product}, see [Try {product} in the browser](quickstart.md#try-in-browser).

.Try it without installing
@note {type=tip}
You can run {product} in the browser at play.quill.dev with no local setup.

## Prerequisites
@id: prerequisites

@include {heading=false}: _fragments/prerequisites.md

## Install the agent
@id: install-agent

@steps
1. Install the agent package:

   @variant {pm=npm}:
   ```shell
   npm install -g @quill/agent
   ```
   @variant {pm=pnpm}:
   ```shell
   pnpm add -g @quill/agent
   ```
   @variant {pm=yarn}:
   ```shell
   yarn global add @quill/agent
   ```
   @end

2. Verify the install:

   ```shell
   quill --version
   ```

   The command prints the installed version, {version}.

   @note
   The agent needs write access to your repository's `.quill/` directory.

3. Create `quill.yaml` at the root of your repository:

   ```yaml phrases=true
   agent:
     version: {version}
     watch: docs/
   ```

## Connect to {product}
@id: connect

@variant {deployment=cloud}:
Sign in to {cloud} and copy an API key from **Settings → Keys**, then add it to `quill.yaml`:

```yaml
cloud:
  api_key: ${QUILL_KEY}
```

@variant {deployment=self-managed}:
Point the agent at your server. Self-managed servers must run {product} Server 3.3 or later.

```yaml
server:
  url: https://quill.internal.example.com
```
@end

## Streaming sync
@id: streaming-sync
@available: cloud, self-managed preview 3.4

Streaming sync pushes changes as you save, instead of on each commit.

{cloud}'s streaming sync is enabled by default for new {cloud}-hosted workspaces.

For event formats, see the [streaming API reference]({api}streaming).

## Troubleshooting
@id: troubleshooting

@note {type=warning}:
If the agent exits immediately, check the log at `~/.quill/agent.log`.

A common cause is an expired API key. Generate a new key, then restart the agent. See [](keys.md#rotate-keys).
@end
