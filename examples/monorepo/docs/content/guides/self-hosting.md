---
title: Run {product} on your own servers
description: Install and upgrade the self-hosted {product} server.
available: self-hosted 2.4
---

The self-hosted server is the same service as {cloud}, packaged as one container. It needs PostgreSQL 15 or later and about 2 GB of memory.

## Install the server
@id: install-server

@steps
1. Create a database and a user for {product}:

   ```sql
   CREATE DATABASE lantern;
   CREATE USER lantern WITH PASSWORD 'change-me';
   GRANT ALL PRIVILEGES ON DATABASE lantern TO lantern;
   ```

2. Start the server:

   ```shell phrases=true
   docker run -d -p 8080:8080 \
     -e LANTERN_DATABASE_URL=postgres://lantern:change-me@db:5432/lantern \
     ghcr.io/lantern/server:{version}
   ```

3. Open `http://localhost:8080` and create the first admin account.

@note {type=important}
Put the server behind TLS before you connect production apps to it. SDKs send user attributes with every check.

## Configure the server
@id: configure-server

The server reads `lantern.yaml` from its working directory. See [](../reference/configuration.md) for every setting.

## Upgrade
@id: upgrade

Upgrades run database migrations on startup. Back up the database first, then start the new version:

```shell
docker pull ghcr.io/lantern/server:latest
```
