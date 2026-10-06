---
title: Sign in
description: How the client gets a session.
---

# Sign in

The client signs in with your token and gets a session, which lists the flags that are on for you:

@snippet: api:src/auth.ts#login

The session lasts until your program exits.
