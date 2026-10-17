---
type: page
title: API keys
description: Create and rotate the API keys the Quill agent uses.
---

## Create a key
@id: create-key

Sign in to {cloud} and open **Settings → Keys**.

## Rotate keys
@id: rotate-keys
@memo:
Rotation comes before revocation: support found that people who revoke
first lock out their own agents. See [the incident](notes/2026-08.md).

Keep the {product} name out of the heading; search shows it already.
@end

Generate a new key, put it in `quill.yaml`, and restart the agent. To install the agent first, see [](install-agent.md#install-agent).

## Sign webhook events
@id: sign-webhooks
@available: self-managed 3.5

A key also signs the events the agent sends to your [webhooks](webhooks.md).
