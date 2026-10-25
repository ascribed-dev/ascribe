OpenAPI specs that pages link into, by name: each one's file, and where its reference is published. A link to a declared spec with a fragment, `[Get a user](../api/openapi.yaml#getUser)`, is a link to the operation whose `operationId` is the fragment, percent-decoded (SPEC §7.11); it's checked against the spec, and written as the operation's address in the reference.

```toml
[apis.public]
spec = "api/openapi.yaml"
reference = { renderer = "redoc", at = "/api/" }

[apis.partner]
spec = "../services/partner/openapi.json"
reference = "/partner-api/#/operations/{operationId}"
```

@include: ../../_generated/content-model-apis.md

`spec` is read as YAML, or as JSON when its name ends in `.json`, the same way for Swagger 2.0 and OpenAPI 3.0 to 3.2: each operation of `paths`, with its method, path, `operationId`, `summary`, first tag, and `deprecated`. Nothing else is read, and nothing is validated. A path item that's a `$ref` to another file isn't followed. A spec outside the project's folder is read for its operations, and a link to it with no fragment is still a link outside the project (`link-spec-outside-project`).

`reference` is a table for Redoc:

@include: ../../_generated/content-model-apis-reference.md

Redoc's address of an operation is `<at>#operation/<operationId>`, which Redoc 2 and 3 both open. Any other renderer's is a template: a string with `{operationId}`, and `{tag}` for the operation's first tag when the renderer's address has it, each filled in percent-encoded. Swagger UI's is `"/api/#/{tag}/{operationId}"`, and Stoplight Elements' with its hash router `"/api/#/operations/{operationId}"`. Either address is joined to `[consumer] base-path`, unless it's absolute (`https://…` or `//…`), for a reference published on another site. It's the site's base path in every line of a versioned build: an API's reference isn't a line's.
