---
title: Users
---

The pattern is left out, and the rest is still checked:

```json request="../api/openapi.yaml#createUser"
{ "handle": "ada", "age": "36" }
```

A schema in another file isn't checked:

```json request="../api/openapi.yaml#createTeam"
{ "name": 1 }
```
