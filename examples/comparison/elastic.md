---
navigation_title: Install the agent
description: Install and configure the Quill agent to sync your docs to Quill Cloud or a self-managed Quill server.
applies_to:
  deployment:
    ess: ga
    self: preview 3.3
---

# Install the {{quill}} agent [install-quill-agent]

The {{quill}} agent watches your docs repository and syncs changes to {{quill}}. This page covers installing the agent with a package manager, configuring it, and connecting it to {{quill-cloud}} or a self-managed server. If you only want to try {{quill}}, see [Try {{quill}} in the browser](/get-started/quickstart.md#try-in-browser).

:::{admonition} Try it without installing
You can run {{quill}} in the browser at play.quill.dev with no local setup.
:::

## Prerequisites [prerequisites]

:::{include} _snippets/prerequisites.md
:::

## Install the agent [install-agent]

::::::{stepper}

:::::{step} Install the agent package

::::{tab-set}
:::{tab-item} npm
```shell
npm install -g @quill/agent
```
:::

:::{tab-item} pnpm
```shell
pnpm add -g @quill/agent
```
:::

:::{tab-item} yarn
```shell
yarn global add @quill/agent
```
:::
::::

:::::

:::::{step} Verify the install

```shell
quill --version
```

The command prints the installed version, {{quill-version}}.

:::{note}
The agent needs write access to your repository's `.quill/` directory.
:::

:::::

:::::{step} Create the configuration file

Create `quill.yaml` at the root of your repository:

```yaml subs=true
agent:
  version: {{quill-version}}
  watch: docs/
```

:::::

::::::

## Connect to {{quill}} [connect]

::::{applies-switch}

:::{applies-item} ess: ga
Sign in to {{quill-cloud}} and copy an API key from **Settings → Keys**, then add it to `quill.yaml`:

```yaml
cloud:
  api_key: ${QUILL_KEY}
```
:::

:::{applies-item} self: preview 3.3
Point the agent at your server. Self-managed servers must run {{quill}} Server 3.3 or later.

```yaml
server:
  url: https://quill.internal.example.com
```
:::

::::

## Streaming sync [streaming-sync]

{applies_to}`ess: ga` {applies_to}`self: preview 3.4+` Streaming sync pushes changes as you save, instead of on each commit.

{{quill-cloud}}'s streaming sync is enabled by default for new workspaces.

For event formats, see the [streaming API reference]({{quill-api}}streaming).

## Troubleshooting [troubleshooting]

:::{warning}
If the agent exits immediately, check the log at `~/.quill/agent.log`.

A common cause is an expired API key. Generate a new key, then restart the agent. See [Rotate API keys](/manage/keys.md#rotate-keys).
:::
