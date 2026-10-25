---
title: Users
---

Create a user:

```json request="../api/openapi.yaml#createUser"
{ "name": 42 }
```

It answers with the user:

```json response="../api/openapi.yaml#createUser 201"
{
  "id": "u_1",
  "nickname": "Ada",
  "joined": "2026-10-25"
}
```
