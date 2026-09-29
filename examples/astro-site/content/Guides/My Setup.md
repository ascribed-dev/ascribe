---
title: Set up Loom
description: Install and configure Loom.
available: cloud, self-managed preview 3.4
---

This guide sets up {product}. Back to the [documentation home](../index.md).

## Requirements
@id: requirements

@include: ../_fragments/requirements.md

## Install
@id: install

@variant {pm=npm}:
```shell
npm install -g @loom/cli
```
@variant {pm=pnpm}:
```shell
pnpm add -g @loom/cli
```
@variant {pm=yarn}:
```shell
yarn global add @loom/cli
```
@end

## Connect
@id: connect

@variant {deployment=cloud}:
Sign in to {product} Cloud and copy your API key.
@variant {deployment=self-managed}:
Point the CLI at your own server.
@end

## Weave configuration
@id: weave-config

Set the `weave` options in `loom.yaml` (here is a [sample config](<../downloads/loom.yaml>)). Every option is listed in the [options reference](../Reference/Options.md).

## Streaming
@id: streaming
@available: self-managed preview 3.4

Streaming pushes changes as you save.
