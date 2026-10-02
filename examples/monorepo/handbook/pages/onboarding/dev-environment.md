---
title: Set up your development environment
description: Run the Lantern service and its tests on your laptop.
owner: platform
---

The {product} service is in `service/` at the root of the repository. You run it, and its database, with Docker.

## Install the tools
@id: install-tools

```shell
brew install go@1.22 docker just
```

## Run the service
@id: run-the-service

@steps
1. Start the database:

   ```shell
   just db-up
   ```

2. Run the service with live reload:

   ```shell
   just dev
   ```

   The service listens on `http://localhost:8080`.

3. Run the tests:

   ```shell
   just test
   ```

@note {type=tip}
`just --list` shows every task. The `justfile` is the source of truth for how we build and test.

.If the database won't start
@details:
Another PostgreSQL is probably using port 5432. Stop it, or start ours on another port:

```shell
LANTERN_DB_PORT=5433 just db-up
```
@end
