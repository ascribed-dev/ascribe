---
title: Secrets
owner: security
review: yearly
effective: 2025-09-01
---

Secrets are passwords, API keys, tokens, and private keys: anything that grants access if someone else has it.

@note {type=policy}
Secrets never go in source code, chat, or tickets. Store them in the secrets manager, and read them at runtime.

## If a secret leaks
@id: if-a-secret-leaks

Treat it as an incident, at least SEV2. Rotate the secret first, then report it to {channel}: a leaked secret that's already rotated is a much smaller problem.

## Rotation
@id: rotation

Rotate every secret at least once a year, and whenever someone who knew it leaves the team.
