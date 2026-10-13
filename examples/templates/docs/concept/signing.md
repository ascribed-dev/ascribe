---
type: concept
title: "Release signing"
description: "What it means that Quill signs a release, and why a reader can trust a signed one."
---

This article explains what Quill's release signature is, and what it tells a reader who checks it.

A release signature is a short file published beside each release. Quill writes it with the project's signing key when the release is published, and anyone with the project's public key can check that the release is the one the project published.

## Use cases

A team that mirrors releases checks each signature before it copies a release, so a mirror never carries a release the project didn't publish.

## Related resources

To rotate the key a release is signed with, see [Rotate keys](../how-to/rotate-keys.md).
