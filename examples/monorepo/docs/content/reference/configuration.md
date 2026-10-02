---
title: Server configuration
description: The settings in a self-hosted server's lantern.yaml.
since: "2.4"
---

A self-hosted server reads `lantern.yaml` from its working directory. Environment variables override the file: `server.port` is `LANTERN_SERVER_PORT`.

```yaml
server:
  port: 8080
  public-url: https://lantern.internal.example.com
database:
  url: postgres://lantern:change-me@db:5432/lantern
  pool-size: 20
```

## server
@id: server

| Setting | Default | Description |
|---|---|---|
| `port` | `8080` | The port the server listens on. |
| `public-url` | none | The URL SDKs and the dashboard use to reach the server. |

## database
@id: database

| Setting | Default | Description |
|---|---|---|
| `url` | required | The PostgreSQL connection URL. |
| `pool-size` | `20` | The most connections the server opens. |

## audit
@id: audit
@available: audit-log

| Setting | Default | Description |
|---|---|---|
| `retention-days` | `90` | How long audit entries are kept. |
| `export` | none | A URL to send each entry to, as JSON. |
