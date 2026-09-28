# Output-layout contract

This contract says where `tessera build` writes each build's output, how it records what it wrote, and how it replaces a previous build's output without ever deleting a file it didn't write. Phase 18 implements it for every emitter; phase 20 adds the site emitter through the same code; the Astro integration (phases 21 and 22) reads the output it describes.

## 1. Directory layout

```
<output-dir>/                   [project] output-dir, default .tessera/build
  .lock                         held while a build writes (§4)
  .staging/                     work in progress (§4)
  <build>/
    <emitter>/                  one emitter's output: the emitter root
    <emitter>.manifest.json     what that emitter root holds (§3)
```

- `<build>` is the build's name from `tessera.toml`, as written (`site`, `cloud`, `self-managed-3.3`). Build names are unique ignoring case (content-model.md, `model-build-name-case`), and never start with `.`, so they can't clash with `.lock` or `.staging`.
- `<emitter>` is `site`, `plain`, or `json`.
- Each emitter root is complete on its own: its pages, and a copy of every asset they use ([asset contract](assets.md)). Nothing is shared between emitter roots or builds.

### 1.1 Inside an emitter root

Pages mirror their source paths relative to the content root:

| Emitter | Page `guides/install.md` is written to |
|---|---|
| `site` | `guides/install.md` |
| `plain` | `guides/install.md` |
| `json` | `guides/install.json` |

Only pages the build publishes are written (SPEC §9.3); fragments never are.

Assets go where the [asset contract](assets.md) §3 puts them: at their mirrored path, or, for links in the site output, under `_tessera/files/`.

**`_tessera/` is reserved** in every emitter root for files Tessera places other than pages and assets inside the content root: assets from outside the content root (`_tessera/up/`), the site output's published files (`_tessera/files/`), and generated files an emitter adds, such as the site emitter's Zod schema (phase 20 chooses its name under `_tessera/`).

## 2. What Tessera owns

Tessera owns exactly these, and touches nothing else under `<output-dir>`:

- the files listed in a manifest (§3);
- the manifests themselves;
- `<output-dir>/.staging/` and everything in it;
- `<output-dir>/.lock`.

Everything else under `<output-dir>` belongs to the user, even inside an emitter root. A user's file is never deleted, moved, or overwritten, and a build that would need to overwrite one fails instead (§4). A directory is removed only when a build empties it by removing files it owned.

## 3. The manifest

Each emitter root has a manifest beside it, `<output-dir>/<build>/<emitter>.manifest.json`, listing every file in the root that Tessera wrote. It's JSON:

```json
{
  "format": "tessera-manifest",
  "version": 1,
  "build": "site",
  "emitter": "site",
  "files": [
    { "path": "_tessera/files/downloads/quill.yaml", "kind": "asset", "source": "downloads/quill.yaml",
      "url": "/docs/_tessera/files/downloads/quill.yaml" },
    { "path": "_tessera/up/shared/logo.png", "kind": "asset", "source": "../shared/logo.png" },
    { "path": "guides/img/settings.png", "kind": "asset", "source": "guides/img/settings.png" },
    { "path": "guides/install.md", "kind": "page", "source": "guides/install.md" }
  ]
}
```

| Field | Meaning |
|---|---|
| `format` | Always `"tessera-manifest"`. A file at a manifest's path without it isn't a manifest, and isn't Tessera's (§4). |
| `version` | The manifest format's version: `1`. A reader rejects versions it doesn't know. |
| `build`, `emitter` | Which output this is. |
| `files` | Every file in the emitter root that Tessera wrote, sorted by `path`. |
| `files[].path` | The file's path relative to the emitter root, `/`-separated. |
| `files[].kind` | `"page"`, `"asset"`, or `"generated"` (anything else Tessera writes, such as a schema). |
| `files[].source` | For a page or asset, its source path relative to the content root, starting with `..` for a file outside it. Absent for generated files. |
| `files[].url` | For a site-output asset under `_tessera/files/`, the URL pages use for it, which the consumer must serve it at (asset contract §3.2). Absent otherwise. |

The manifest lists files only. Directories are implied by their paths.

## 4. Replacing a previous build's output

A build writes to a staging directory and replaces the previous output only when it has succeeded. Each build-and-emitter pair is replaced separately: building one build, or one emitter, doesn't touch the others.

1. **Lock.** Take an exclusive advisory lock on `<output-dir>/.lock` (creating it if needed), held until the build ends and released by the operating system if the process dies. If another build holds it, fail: "another tessera build is writing to `<output-dir>`".
2. **Read the previous manifest**, if there is one. A file at the manifest's path that isn't a manifest (§3) fails the build: Tessera can't tell what it owns.
3. **Emit into staging:** `<output-dir>/.staging/<build>/<emitter>/`, removing any leftover staging for that pair first. If emitting fails, or the build has errors (SPEC §8.2: "a build MUST fail on errors"), remove the staging directory and stop. The previous output is untouched.
4. **Check before touching anything.** Fail, naming every problem, and leave the output untouched, if:
   - a file would be written where a file exists that the previous manifest doesn't list (a user's file is in the way);
   - a file would be written where a directory exists, or a directory is needed where a file exists;
   - two new files have the same path, or paths that differ only in case (they'd be one file on macOS and Windows).
5. **Record ownership first.** Write the manifest listing every file in the previous manifest *and* every new file. Write each manifest by writing a temporary file in `.staging/` and renaming it into place, so it's never half-written.
6. **Move the new files into place**, replacing the previous build's files at the same paths. A file whose content hasn't changed may be left as it is, so watchers such as Astro's dev server see only real changes.
7. **Remove stale files:** those in the previous manifest that this build didn't produce. Then remove any directory that this step emptied.
8. **Write the final manifest**, listing only this build's files, the same way as in step 5.
9. **Clean up:** remove the staging directory and release the lock.

**The invariant:** at every moment, every file Tessera has written into an emitter root is listed in the manifest on disk. So if a build is interrupted, the root holds either the previous output or a mix of old and new files that are all listed, and the next build removes what it no longer produces. A user's file is never at risk, because Tessera only deletes files a manifest lists.

A file listed in a manifest that the user has since edited is still Tessera's: the next build overwrites or removes it. The output directory is generated; edit the source instead.

## 5. Why this shape

- **One root per build and emitter**, with nothing shared, keeps each output self-contained (SPEC §9.4) and lets a consumer point at exactly one directory, such as an Astro content collection at `.tessera/build/site/site/`.
- **Mirrored paths** make the output's structure the source's structure, keep relative references between pages and assets valid, and make collisions between different files impossible (asset contract §3.1).
- **The manifest beside the root, not in it**, keeps the root holding only output, so a consumer loading every file in it loads nothing else.
- **Failing on a user's file** instead of overwriting it follows the phase plan's rule: output never destroys what Tessera didn't create.
