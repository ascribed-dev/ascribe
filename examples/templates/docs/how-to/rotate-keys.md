---
type: how-to
title: "Rotate keys"
description: "Replace the keys Quill signs releases with, without a release going unsigned."
---

## Overview

This guide explains how to replace the keys Quill signs releases with.

## Before you start

Before you rotate the keys, ensure:

* You can publish a release.
* You have the old key's passphrase.

## Rotate the keys

@steps
1. Run `quill keys new`.

2. Run `quill keys retire` with the old key's name.

## See also

* [Quill](../index.md)
