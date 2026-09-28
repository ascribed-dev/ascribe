---
title: Test
---

@variant {deployment=cloud}:
One.
@variant {deployment=self-managed}:
Two.
@end

@variant {pm=npm|pnpm}:
Three.
@variant {pm=yarn}:
Four.
@end

@variant {deployment=cloud, pm=npm}:
Five.
@variant {deployment=self-managed}:
Six.
@end
