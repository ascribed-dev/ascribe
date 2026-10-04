<!--ascribe-anchor tag="p" source="install-agent.md:7-7"-->
The Quill agent watches your docs repository and syncs changes to Quill. This page covers installing the agent with a package manager, configuring it, and connecting it to Quill Cloud or a self-managed server. If you only want to try Quill, see [Try Quill in the browser](/quickstart/#try-in-browser).

<ascribe-note type="tip" label="Tip" heading="Try it without installing" data-ascribe-source="install-agent.md:9-11">

<!--ascribe-anchor tag="p" source="install-agent.md:11-11"-->
You can run Quill in the browser at play.quill.dev with no local setup.

</ascribe-note>

<!--ascribe-anchor tag="h2" source="install-agent.md:13-13"-->
## Prerequisites <ascribe-attributes id="prerequisites"></ascribe-attributes>

<!--ascribe-anchor tag="p" source="_fragments/prerequisites.md:1-1" via="install-agent.md:16"-->
Before you install the agent, make sure you have:

<!--ascribe-anchor tag="ul" source="_fragments/prerequisites.md:3-4" via="install-agent.md:16" items="3-3 4-4"-->
- Node.js 20 or later.
- A docs repository that Quill can read.

<!--ascribe-anchor tag="p" source="_fragments/prerequisites.md:6-6" via="install-agent.md:16"-->
![Checklist of prerequisites](./_fragments/prerequisites.png)

<!--ascribe-anchor tag="h2" source="install-agent.md:18-18"-->
## Install the agent <ascribe-attributes id="install-agent"></ascribe-attributes>

<ascribe-steps data-ascribe-source="install-agent.md:21-55">

<!--ascribe-anchor tag="ol" source="install-agent.md:22-55" items="22-36 38-47 49-55"-->
1. <!--ascribe-anchor tag="p" source="install-agent.md:22-22"-->
   Install the agent package:

   <ascribe-tabs sync="pm" data-ascribe-source="install-agent.md:24-36">

   <ascribe-tab value="npm" label="npm" data-ascribe-source="install-agent.md:24-27">

   <!--ascribe-anchor tag="pre" source="install-agent.md:25-27"-->
   ```shell
   npm install -g @quill/agent
   ```

   </ascribe-tab>

   <ascribe-tab value="pnpm" label="pnpm" data-ascribe-source="install-agent.md:28-31">

   <!--ascribe-anchor tag="pre" source="install-agent.md:29-31"-->
   ```shell
   pnpm add -g @quill/agent
   ```

   </ascribe-tab>

   <ascribe-tab value="yarn" label="Yarn" data-ascribe-source="install-agent.md:32-35">

   <!--ascribe-anchor tag="pre" source="install-agent.md:33-35"-->
   ```shell
   yarn global add @quill/agent
   ```

   </ascribe-tab>

   </ascribe-tabs>

2. <!--ascribe-anchor tag="p" source="install-agent.md:38-38"-->
   Verify the install:

   <!--ascribe-anchor tag="pre" source="install-agent.md:40-42"-->
   ```shell
   quill --version
   ```

   <!--ascribe-anchor tag="p" source="install-agent.md:44-44"-->
   The command prints the installed version, 3.4.1.

   <ascribe-note type="note" label="Note" data-ascribe-source="install-agent.md:46-47">

   <!--ascribe-anchor tag="p" source="install-agent.md:47-47"-->
   The agent needs write access to your repository's `.quill/` directory.

   </ascribe-note>

3. <!--ascribe-anchor tag="p" source="install-agent.md:49-49"-->
   Create `quill.yaml` at the root of your repository:

   <!--ascribe-anchor tag="pre" source="install-agent.md:51-55"-->
   ```yaml
   agent:
     version: 3.4.1
     watch: docs/
   ```

</ascribe-steps>

<!--ascribe-anchor tag="h2" source="install-agent.md:57-57"-->
## Connect to Quill <ascribe-attributes id="connect"></ascribe-attributes>

<!--ascribe-anchor tag="p" source="install-agent.md:61-61"-->
Sign in to Quill Cloud and copy an API key from **Settings → Keys**, then add it to `quill.yaml`:

<!--ascribe-anchor tag="pre" source="install-agent.md:63-66"-->
```yaml
cloud:
  api_key: ${QUILL_KEY}
```

<!--ascribe-anchor tag="h2" source="install-agent.md:77-77"-->
## Streaming sync <ascribe-attributes id="streaming-sync"></ascribe-attributes>

<ascribe-availability scope="section" data-ascribe-source="install-agent.md:79-79">
<ascribe-availability-target target="cloud" dimension="deployment" states="ga">Quill Cloud (GA)</ascribe-availability-target>; <ascribe-availability-target target="self-managed" dimension="deployment" states="preview" versions="3.4">self-managed (preview, 3.4+)</ascribe-availability-target>
</ascribe-availability>

<!--ascribe-anchor tag="p" source="install-agent.md:81-81"-->
Streaming sync pushes changes as you save, instead of on each commit.

<!--ascribe-anchor tag="p" source="install-agent.md:83-83"-->
Quill Cloud's streaming sync is enabled by default for new Quill Cloud-hosted workspaces.

<!--ascribe-anchor tag="p" source="install-agent.md:85-85"-->
For event formats, see the [streaming API reference](https://api.quill.dev/v3/streaming).

<!--ascribe-anchor tag="h2" source="install-agent.md:87-87"-->
## Troubleshooting <ascribe-attributes id="troubleshooting"></ascribe-attributes>

<ascribe-note type="warning" label="Warning" data-ascribe-source="install-agent.md:90-94">

<!--ascribe-anchor tag="p" source="install-agent.md:91-91"-->
If the agent exits immediately, check the log at `~/.quill/agent.log`.

<!--ascribe-anchor tag="p" source="install-agent.md:93-93"-->
A common cause is an expired API key. Generate a new key, then restart the agent. See [Rotate keys](/keys/#rotate-keys).

</ascribe-note>
