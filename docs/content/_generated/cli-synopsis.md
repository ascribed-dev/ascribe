@memo: Generated from the help text in crates/ascribe-cli/src/ by crates/ascribe-cli/src/docs.rs; edit the help text, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli docs` to rewrite it.
```text
ascribe agents sync       [--check] [--target agents-md|skills|claude|claude-rules|copilot|codex]... [--with-hook] [--cloud] [--agent]
ascribe agents rules      [PATH]
ascribe agents index      [--build <NAME>] [--into <FILE>] [--pages <DIR>] [--format text|json]
ascribe agents skill      [FILE]
ascribe agents prompt     [NAME] [--arg <KEY=VALUE>]... [--list]
ascribe agents hook       HARNESS [--event edit|stop]
ascribe build             [--build <NAME>]... [--emit site,plain,json] [--format text|json] [--anchors]
ascribe check             [PATHS]... [--stdin] [--path <PATH>] [--build <NAME>]... [--editor-build] [--code <CODE>]... [--summary] [--format text|concise|json|prompt] [--deny-warnings] [--fix] [--vale]
ascribe diff              [PAGE] [--base <REV>] [--base-exact] [--build <NAME>]... [--format text|json|html|prompt] [--exit-code] [--pages-only]
ascribe drift             [--base <REV>] [--build <NAME>]... [--format text|json|markdown] [--exit-code]
ascribe explain           [CODE] [--list] [--format text|json]
ascribe fmt               [--check] [PATHS]... [--format text|json]
ascribe init              [DIR] [--templates] [--format text|json]
ascribe link              TARGET --from <PAGE> [--format text|json]
ascribe lsp
ascribe mcp
ascribe model             [PATH] [--section types|dimensions|phrases|features|glossary|widgets|releases|builds|sources|project|consumer|editor|checks|crawlers|listing|permalinks] [--format text|json]
ascribe observe           [--name <NAME>] [--check] [--yes] [--format text|json]
ascribe outline           PAGE [--build <NAME>] [--format text|json]
ascribe permalinks sync   [--check] [--format text|json]
ascribe permalinks list   [--format text|json]
ascribe permalinks verify [NAME]... [--from <FILE>] [--against <PATH>] [--format text|json]
ascribe permalinks retire [NAME]... --reason <TEXT> [--to <PAGE>] [--page <PAGE>] [--under <FOLDER>] [--format text|json]
ascribe refs              TARGET [--limit <N>] [--format text|json]
ascribe rename            TARGET NEW
ascribe render            PAGE [--build <NAME>] [--frontmatter] [--format text|json]
ascribe report            [SECTION]... [--format text|markdown|json|prompt] [--build <NAME>]... [--not-listed] [--site <URL>] [--limit <N>] [--recheck] [--no-history] [--exit-code advice|warning|error]
ascribe search            QUERY [PATHS]... [--type <NAME>] [--case] [--word] [--summary] [--build <NAME>]... [--limit <N>] [--format text|json]
ascribe sources fetch     [NAME]...
ascribe sources update    [NAME]... [--to <REV>] [--format text|json|markdown]
ascribe sources status    [--format text|json]
ascribe templates list
ascribe templates eject   [NAME]... [--to <DIR>] [--keep-type] [--format text|json]
ascribe templates status  [--format text|json] [--exit-code] [--diff <NAME>]
ascribe vale eject
ascribe versions list     [--format text|json]
ascribe versions changes  FROM TO [--format text|json|markdown]
ascribe versions release  LINE [--next <LINE>] [--dry-run] [--force] [--format text|json]
ascribe versions archive  LINE --to <DIR> [--build <NAME>] [--current <URL>] [--format text|json]
ascribe versions retire   LINE [--url <URL>] [--dry-run] [--force] [--show-unused] [--remove-unused] [--format text|json]
ascribe --version
```
