---
title: Releases
description: How a change gets from main to customers.
owner: platform
---

We deploy `main` to production several times a day. Risky changes ship behind a feature flag, turned off, and roll out with {product} itself. The public docs explain rollouts at {docs}guides/rollouts/.

## Deploy a change
@id: deploy

@steps
1. Merge your change to `main`. CI runs the tests and builds an image.
2. The image deploys to staging on its own. Check your change there.
3. Promote it to production from the deploy dashboard, or with:

   ```shell
   just promote
   ```

@note {type=policy}
Don't deploy after 15:00 on a Friday, or during an incident, unless the deploy fixes the incident.

## Roll back
@id: roll-back

If a deploy causes errors, roll back first and investigate after:

```shell
just rollback
```

Rolling back takes about two minutes. Turning off a feature flag takes seconds, so try that first if the change is behind one.

## Release notes
@id: release-notes

Changes customers can see get a line in the release notes. Write it in the pull request description, under **Release note**, and the release bot collects it.
