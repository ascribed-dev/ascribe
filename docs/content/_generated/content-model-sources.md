@memo: Generated from crates/ascribe-model/settings.toml by tests/conformance/tests/docs.rs; edit the registry, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-conformance --test docs` to rewrite it.
| Key | Type | Default | Description |
|---|---|---|---|
| `path` | string (path) | **required** without `git` | The source's folder, relative to the project root. It must exist, be a directory, and be inside the git repository the project is in, when the project is in one. |
| `git` | string (URL) | **required** without `path` | Another repository, as `git` takes it: `https://`, `http://`, `ssh://`, `git://`, `file://`, or `user@host:path`. |
| `branch` | string | the repository's default branch | The branch `ascribe sources update` moves the pin to. |
| `include` | array of patterns | every file | The files a snippet may read, matched against their paths relative to the source's folder (§1.3). |
| `ignore` | array of patterns | `node_modules` and `.git` folders, always | Files left out even when `include` matches them. Every `node_modules` and `.git` folder is left out whatever the list says, before the patterns it adds. |
