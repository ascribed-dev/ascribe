---
title: Install the Quill agent
linkTitle: Install
description: Install and configure the Quill agent to sync your docs to Quill Cloud or a self-managed Quill server.
keywords: quill, agent, install, sync
weight: 20
---

{{< summary-bar feature_name="Quill agent" >}}

The Quill agent watches your docs repository and syncs changes to Quill. This page covers installing the agent with a package manager, configuring it, and connecting it to Quill Cloud or a self-managed server. If you only want to try Quill, see [Try Quill in the browser](/manuals/quill/quickstart.md#try-in-browser).

> [!TIP]
>
> You can run Quill in the browser at play.quill.dev with no local setup.

## Prerequisites {#prerequisites}

{{% include "quill-prerequisites.md" %}}

## Install the agent {#install-agent}

1. Install the agent package:

   {{< tabs group="package-manager" >}}
   {{< tab name="npm" >}}

   ```console
   $ npm install -g @quill/agent
   ```

   {{< /tab >}}
   {{< tab name="pnpm" >}}

   ```console
   $ pnpm add -g @quill/agent
   ```

   {{< /tab >}}
   {{< tab name="yarn" >}}

   ```console
   $ yarn global add @quill/agent
   ```

   {{< /tab >}}
   {{< /tabs >}}

2. Verify the install:

   ```console
   $ quill --version
   ```

   The command prints `{{% param "quill_version" %}}`.

   > [!NOTE]
   >
   > The agent needs write access to your repository's `.quill/` directory.

3. Create `quill.yaml` at the root of your repository:

   ```yaml
   agent:
     version: {{% param "quill_version" %}}
     watch: docs/
   ```

## Connect to Quill

{{< tabs group="deployment" >}}
{{< tab name="Quill Cloud" >}}

Sign in to Quill Cloud and copy an API key from **Settings → Keys**, then add it to `quill.yaml`:

```yaml
cloud:
  api_key: ${QUILL_KEY}
```

{{< /tab >}}
{{< tab name="Self-managed" >}}

Point the agent at your server. Self-managed servers must run Quill Server 3.3 or later.

```yaml
server:
  url: https://quill.internal.example.com
```

{{< /tab >}}
{{< /tabs >}}

## Streaming sync

{{< summary-bar feature_name="Quill streaming sync" >}}

Streaming sync pushes changes as you save, instead of on each commit.

Quill Cloud's streaming sync is enabled by default for new workspaces.

For event formats, see the [streaming API reference](https://api.quill.dev/v3/streaming).

## Troubleshooting

> [!WARNING]
>
> If the agent exits immediately, check the log at `~/.quill/agent.log`.
>
> A common cause is an expired API key. Generate a new key, then restart the agent.

For more information, see [Rotate API keys](/manuals/quill/keys.md#rotate-keys).
