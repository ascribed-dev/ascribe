---
type: tutorial
title: "Publish your first release"
description: "Publish a signed release of a project with Quill, from an empty folder to a release a reader can check."
---

## Overview

In this tutorial, you'll learn how to publish a signed release with Quill. It's for someone who has installed Quill and hasn't published with it yet.

By the end of this tutorial, you'll be able to:

* Make a signing key.
* Publish a release signed with it.

## Before you start

Before you start the tutorial, you should have a project folder with a `quill.toml` in it.

## Publish the release

To get started, open a terminal in the project's folder.

@steps
1. Run `quill keys new` to make a signing key.

2. Run `quill publish` to publish the release, signed with the key.

## Summary

In this tutorial, you learned how to make a signing key and publish a release signed with it.

## Next steps

Read [Release signing](../concept/signing.md) to learn what the signature tells a reader.
