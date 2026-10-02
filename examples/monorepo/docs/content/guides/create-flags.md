---
title: Create feature flags
description: Choose a flag type, name it well, and set safe defaults.
---

A feature flag has a key, a type, and a default value. The key is how your code asks for the flag, so it can't change after you create the flag.

## Flag types
@id: flag-types

| Type | Values | Use it for |
|---|---|---|
| `boolean` | `true`, `false` | Turning a feature on or off. |
| `string` | One of a list you declare | Choosing between variants, such as button copy. |
| `number` | Any number | Tuning a value, such as a timeout or a page size. |

@note {type=tip}
Most flags should be booleans. A flag with many values is harder to reason about, and harder to remove once the feature ships.

## Name a flag
@id: name-a-flag

Flag keys are lowercase words joined by hyphens, such as `new-checkout` or `search-ranking-v2`. Name the flag after the change, not the team or the ticket: the key outlives both.

## Create a flag
@id: create-a-flag

@steps
1. Create the flag with its type and default:

   ```shell
   lantern flags create checkout-copy --project checkout --type string --values control,short,friendly --default control
   ```

2. Check that it exists everywhere it should:

   ```shell
   lantern flags get checkout-copy --project checkout
   ```

The default is what every user gets until a rollout says otherwise, and what the SDK returns if it has never reached {product}.

.Show the full flag definition
@details:
`lantern flags get --output json` prints the whole definition:

```json
{
  "key": "checkout-copy",
  "type": "string",
  "values": ["control", "short", "friendly"],
  "default": "control",
  "environments": {
    "production": { "enabled": false },
    "staging": { "enabled": true }
  }
}
```
@end

## Remove a flag
@id: remove-a-flag

When a feature has shipped to everyone, remove the flag from your code first, then archive it:

```shell
lantern flags archive new-checkout --project checkout
```

@note {type=warning}
Archiving a flag that your code still reads makes the SDK return its default. Search your code for the key before you archive it.
