---
section: The language
---

**A file in a `.git` folder is never an asset.** A link or image that names one, directly or through a symbolic link, and in any case, is reported as `link-target-missing` or `image-source-missing` with the `git` message, whether or not the file is there, and no output copies it. The asset contract's §2 step 5 and SPEC §9.4 say so. _Why:_ the boundary let a page reference any file in the project, so a link to `../.git/config` copied the checkout's `.git/config` into the site, the HTML report, and an archive, and on a CI runner that file holds the job's token. _From:_ the maintainer, in the security review's phase 6 and [issue #1222](https://github.com/ascribed-dev/ascribe/issues/1222), 29 October 2026.
