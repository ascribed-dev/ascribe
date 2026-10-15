## Create a key <ascribe-attributes id="create-key"></ascribe-attributes>

Sign in to Quill Cloud and open **Settings → Keys**.

## Rotate keys <ascribe-attributes id="rotate-keys"></ascribe-attributes>

Generate a new key, put it in `quill.yaml`, and restart the agent. To install the agent first, see [Install the agent](/install-agent/#install-agent).

## Sign webhook events <ascribe-attributes id="sign-webhooks"></ascribe-attributes>

<ascribe-availability scope="section">
<ascribe-availability-target target="self-managed" dimension="deployment" states="ga" versions="3.5">self-managed (GA, 3.5+)</ascribe-availability-target>
</ascribe-availability>

A key also signs the events the agent sends to your [webhooks](/webhooks/).
