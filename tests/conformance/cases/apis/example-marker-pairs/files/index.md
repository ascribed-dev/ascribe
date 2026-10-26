---
title: Users
---

A marker in any order beside a title:

```json title="Create a user" request="../api/openapi.yaml#createUser"
{ "name": 42 }
```

A marker's value can be a token:

```json request=../api/openapi.yaml#createUser
{ "name": 42 }
```

A marker inside a title's value isn't one, and neither is a marker with spaces around its `=`:

```json title="see request=../api/openapi.yaml#createUser"
{ "name": 42 }
```

```json request = "../api/openapi.yaml#createUser"
{ "name": 42 }
```
