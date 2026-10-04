---
title: Elements
---

@note: A one-line note.

@note {type=tip}
A note on the paragraph after it.

.A titled note
@note {type=warning}:
A container note.

With two paragraphs.
@end

@steps
1. Do this.
2. Then this.

.More detail
@details:
Hidden until opened.
@end

@available: cloud
A cloud-only paragraph.

## A section for some
@available: self-managed 3.5

Only some see this.

@quill-labspace {lab=first-sync}

@quill-aside: An aside in one line.

.An aside
@quill-aside:
An aside around a paragraph.
@end

.Before
@quill-compare:
Old way.

.After
@quill-compare:
New way.
@end

@variant {pm=npm}:
Use npm.
@variant {pm=pnpm}:
Use pnpm.
@end
