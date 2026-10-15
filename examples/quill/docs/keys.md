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

Generate a new key, put it in `quill.yaml`, and restart the agent. To install the agent first, see [](install-agent.md#install-agent).

## Sign webhook events
@id: sign-webhooks
@available: self-managed 3.5

A key also signs the events the agent sends to your [webhooks](webhooks.md).
