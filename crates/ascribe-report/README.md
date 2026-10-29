# ascribe-report

The programs Ascribe runs and reads the answers of, and what it remembers of them between runs: what `ascribe report` reports beyond `ascribe check`, and the prose check. This README is about the code.

## Pieces

| Module | Main types and functions | What it does |
|---|---|---|
| `src/tool.rs` | `ToolError` | Running a program the project installs, with a fixed argument list, never through a shell, until it ends or its time runs out (content checks decision 5). |
| `src/prose/` | `lint`, `lint_pages`, `Linter`, `Program`, `eject` | The prose, checked with Vale (`[checks.vale]`): each page's prose extracted with a map back to its source, linted in one run, and each alert a `prose` diagnostic at its place; the files Vale runs with (`setup.rs`), and `ascribe vale eject` (`eject.rs`). |
| `src/links.rs` | `check`, `LinkChecker`, `Lychee`, `Answer` | External links, checked by lychee: what `ascribe report links` reports, with a server's error asked about again before it counts. |
| `src/links/cache.rs`, `src/links/history.rs` | `Cache`, `History` | The answers a link check reuses for a few days, and what's known of each link, so one that answered before is held as unconfirmed while it fails. |
| `src/site.rs` | `SiteChecker`, `Afdocs`, `owner` | A published site, checked against the Web Documentation Delivery Spec with `afdocs`: what `ascribe report delivery` reports. |
| `src/report_history.rs` | `ReportHistory` | What `ascribe report` remembers of its sections from one run to the next, so each says what's new and what's gone. |
| `src/store.rs` | `Store`, `MemoryStore` | Where the caches and histories are read and written: through the `Store` the caller hands the library, the CLI's on the disk and a test's in memory. |

```rust,ignore
let found = ascribe_report::prose::lint(
    &project,
    None,
    &ascribe_report::prose::Program,
    ascribe_report::prose::PROJECT_TIMEOUT,
);
```

## Rules

- **It depends on `ascribe-core`, `ascribe-syntax`, `ascribe-model`, `ascribe-resolve`, and `ascribe-check`,** whose `Diagnostic`, `Project`, and a finding's `Identity` it uses. `ascribe-check` never uses it: the checks run no program.
- **What it remembers, it writes through its caller.** The caches and histories go through the caller's `Store`, and only the report's code opens them. The files Vale runs with are the one write here, each marked `Library write:` (`crates/ascribe-resolve/tests/all/file_writes.rs`).
- **Each program is behind a trait** (`Linter`, `LinkChecker`, `SiteChecker`), so a test gives a stand-in and only `tests/all/vale.rs` runs a real one.

## Tests

- `tests/all/vale.rs`: the prose checked by a real Vale, the one `ASCRIBE_TEST_VALE` names, which CI installs on Linux.
- `tests/all/link_cache.rs`: the answers a link check reuses and what it remembers, and that only the report's code opens them.
- `tests/all/link_retry.rs`: a server's error is asked about again, and only the second answer counts.
- `tests/all/site.rs`: the site's checks, each known by who can make it pass.
- The unit tests in `src/prose/` cover the rest of the prose check with a stand-in for Vale.
