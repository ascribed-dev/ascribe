---
title: Users
---

A trailing comma isn't JSON:

```json request="../api/openapi.yaml#createUser"
{
  "name": "Ada",
}
```

Nor is a YAML block, which isn't checked:

```yaml request="../api/openapi.yaml#createUser"
name: Ada
```

Inside a list, the place is still the block's:

- Send:

  ```json title="Create" request="../api/openapi.yaml#createUser"
  { "name": "Ada" } trailing
  ```
