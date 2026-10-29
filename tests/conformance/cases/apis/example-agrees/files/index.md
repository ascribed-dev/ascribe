---
title: Users
---

Swagger 2.0, where `id` is read-only and so not sent, and a body is in every
media type the operation lists:

```json request="../api/swagger.yaml#createUser"
{ "name": "Ada", "nickname": null, "age": 36 }
```

```json response="../api/swagger.yaml#createUser 201 application/vnd.users+json"
{ "id": "u_1", "name": "Ada" }
```

OpenAPI 3.0, through `$ref`s to a request body, a response, and a schema
whose name has a space, where `password` is write-only and so not returned:

```json request="../api/openapi-3.0.json#replaceUser"
{ "name": "Ada", "password": "s3cret", "team": null, "role": null }
```

```json response="../api/openapi-3.0.json#replaceUser 200"
{ "name": "Ada", "manager": { "name": "Grace", "password": "x" }, "role": "admin" }
```

OpenAPI 3.1, by method and path, the response's status a range, with one JSON
media type among others:

```json response="../api/openapi-3.1.yaml#get/users/{id} 2XX"
{
  "name": "Ada",
  "email": null,
  "joined": "2026-10-25T09:00:00Z",
  "tags": ["admin", "staff"]
}
```

OpenAPI 3.1, where a `$ref`'s siblings are keywords, so `id`, read-only
beside its `$ref`, isn't sent:

```json request="../api/openapi-3.1.yaml#makeUser"
{ "name": "Ada" }
```

A block with no marker isn't checked:

```json
{ "anything": true }
```
