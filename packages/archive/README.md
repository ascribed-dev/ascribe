# @ascribed/archive

The search script of an archive that `ascribe versions archive` writes: the box at the top of each page's sidebar, which searches the archive's `search-index.js` from a folder opened from a disk, with nothing fetched from anywhere else. It's private, and isn't published: the binary carries a bundled copy, `crates/ascribe-emit/src/archive/search.js`, which every archive one version of Ascribe writes has.

- `src/search.ts`: which pages and headings a query finds, best first, and the words around each match.
- `src/index.ts`: the box, drawn where the sidebar's `<script>` is, and the index, loaded the first time the box is used.

After changing either, rewrite the crate's copy:

```sh
pnpm --filter @ascribed/archive embed
```

`test/embedded.test.ts` fails while the copy differs. The archive contract's section 6, `docs/content/contracts/archive.md`, says what the index holds and how a query is answered, and decisions 75 to 77 in [the Versions plan](../../project-docs/versions/README.md) why.
