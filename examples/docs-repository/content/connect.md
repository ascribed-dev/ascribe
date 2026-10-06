---
title: Connect
description: What the client does when it starts.
---

# Connect

`connect` signs in, retrying a few times if the network fails, and keeps the session for every flag you read after:

@snippet: api:src/client.ts#connect

Call it once, when your program starts.
