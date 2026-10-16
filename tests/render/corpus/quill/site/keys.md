<!--ascribe-anchor tag="h2" source="keys.md:7-7"-->
## Create a key <ascribe-attributes id="create-key"></ascribe-attributes>

<!--ascribe-anchor tag="p" source="keys.md:10-10"-->
Sign in to Quill Cloud and open **Settings → Keys**.

<!--ascribe-anchor tag="h2" source="keys.md:12-12"-->
## Rotate keys <ascribe-attributes id="rotate-keys"></ascribe-attributes>

<!--ascribe-anchor tag="p" source="keys.md:21-21"-->
Generate a new key, put it in `quill.yaml`, and restart the agent. To install the agent first, see [Install the agent](/install-agent/#install-agent).

<!--ascribe-anchor tag="h2" source="keys.md:23-23"-->
## Sign webhook events <ascribe-attributes id="sign-webhooks"></ascribe-attributes>

<ascribe-availability scope="section" data-ascribe-source="keys.md:25-25">
<ascribe-availability-target target="self-managed" dimension="deployment" states="ga" versions="3.5">self-managed (GA, 3.5+)</ascribe-availability-target>
</ascribe-availability>

<!--ascribe-anchor tag="p" source="keys.md:27-27"-->
A key also signs the events the agent sends to your [webhooks](/webhooks/).
