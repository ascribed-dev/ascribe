---
section: The binary
---

**A file under a `.git` folder is never an asset.** A link or an image whose file is inside the project but under a `.git` folder, such as a link from a page to the checkout's `config` file under `.git`, is reported as `link-target-missing` or `image-source-missing` with the `git` message, and no build, report, preview, or archive copies it; `Layout::is_in_git` in `crates/ascribe-resolve/src/layout.rs` holds it, beside the boundary check, and the assets contract's step 5 says so. _Why:_ the contract let a page reference any file inside the project root, and a checkout's `.git/config` holds the job's token on a CI runner that keeps its credentials, so a fork's page could publish it. _From:_ the maintainer, after the security review of October 2026, in [#1222](https://github.com/ascribed-dev/ascribe/issues/1222), 29 October 2026.
