---
type: troubleshooting
title: "Troubleshooting signing"
description: "What to do when Quill can't sign a release, or a signature doesn't check."
---

This page covers the problems Quill reports while it signs a release.

## Quill says there's no signing key

`quill publish` stops with `no signing key`.

### Cause

The project has no key that isn't retired.

### Solution

Run `quill keys new`, then `quill publish` again.

### For more information

* [Rotate keys](../how-to/rotate-keys.md)
