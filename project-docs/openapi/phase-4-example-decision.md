# Phase 4: Decide the example marker

Part of [OpenAPI](README.md). Needs phase 1. A spike, then a decision by the maintainer; no shipped code.

## Goal

The maintainer settles how a JSON block names its operation, from a spike of the validator, so phase 5 builds a language change and a dependency that are already chosen.

## The spike

In a scratch crate outside the workspace, never merged:

- **The validator:** the `jsonschema` crate, and one alternative if it costs too much. What it adds to the release binary's size and to a clean build's time, measured against `main`'s `ascribe`; its licence; whether it builds for every target the release builds.
- **Positions:** a JSON parser that keeps each value's span, or a span map built beside `serde_json`, so a validator's instance path (`/user/name`) becomes a line and column in the block. Try it on the spikes' specs' examples.
- **The translation:** OpenAPI 3.0's and Swagger 2.0's schemas into JSON Schema, `nullable` among the rest, on the corpus's schemas: how many validate after it, and what's left over.
- **Components:** a schema's `$ref`s into the same file's components, resolved for the validator.

Recorded in this phase's pull request, which changes only `project-docs/`.

## The recommendation

The proposal's open question, with its recommendation: an attribute in the info string, as `title` and `tested` are:

````markdown
```json request="../api/openapi.yaml#createUser"
{ "name": "Ada" }
```

```json response="../api/openapi.yaml#createUser 201"
{ "id": "u_1", "name": "Ada" }
```
````

- **The value** is a link to an operation, either form phase 2 allows, so it's checked, completed, and renamed as a link is; a response names its status after a space, and its media type after another when there's more than one.
- **The info string** keeps the block a plain fenced block that every Markdown viewer still shows. The alternative, a directive around the block, adds a container for one attribute.
- **What's checked:** the block's JSON against the request body's or the response's schema, for its media type, `application/json` by default.

The spike may change the recommendation: if positions can't be had, phase 5 reports a disagreement at the block, not at its place, and says so.

## Also settled here

- The diagnostics' names and levels: an example that disagrees, one that isn't JSON, and a marker naming an operation, status, or media type the spec doesn't have.
- The dependency, and its decision.

## Output

A decision file in `project-docs/decisions/`, numbered by this phase's issue, with the marker, what's checked, the dependency, and the spike's numbers. Phase 5 starts from it.
