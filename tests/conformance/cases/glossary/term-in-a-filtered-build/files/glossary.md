---
title: Glossary
---

## Streaming sync
@term
@available: self-managed 3.5

Sending each change to a project's data to every connected client as soon as it's written, instead of waiting for the client's next poll. A client that loses its connection gets the changes it missed when it reconnects, in the order they were made, so two clients that edit the same record always end up agreeing on its value.

## Workspace
@term

@variant {deployment=cloud}:
A workspace Quill hosts for you: the projects, members, and settings of one team, kept in the region you choose when you sign up. Quill backs it up every hour, upgrades it without downtime, and keeps it within the limits of your plan, and you reach it from the web app, the command line, or the API with a key from Settings.
@variant {deployment=self-managed}:
A workspace on your own servers.
@end

## Relay
@term

@include: _relay.md
