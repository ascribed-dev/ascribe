<!-- Generated from the help text in crates/ascribe-cli/src/ by crates/ascribe-cli/src/docs.rs. Edit the help text, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli docs`. -->

```text
ascribe build          [--build <NAME>]... [--emit site,plain,json] [--format text|json] [--anchors]
ascribe check          [PATHS]... [--stdin] [--path <PATH>] [--build <NAME>]... [--editor-build] [--summary] [--format text|concise|json|prompt] [--deny-warnings]
ascribe diff           [--base <REV>] [--base-exact] [--build <NAME>]... [--format text|json|html] [--exit-code]
ascribe drift          [--base <REV>] [--build <NAME>]... [--format text|json|summary] [--exit-code]
ascribe explain        [CODE] [--list] [--format text|json]
ascribe fmt            [--check] [PATHS]...
ascribe link           TARGET --from <PAGE> [--format text|json]
ascribe lsp
ascribe model          [PATH] [--section types|dimensions|phrases|features|glossary|widgets|builds] [--format text|json]
ascribe outline        PAGE [--build <NAME>] [--format text|json]
ascribe refs           TARGET [--limit <N>] [--project <PATH>] [--format text|json]
ascribe render         PAGE [--build <NAME>] [--frontmatter] [--format text|json]
ascribe sources fetch  [NAME]...
ascribe sources update [NAME]... [--to <REV>] [--format text|json|summary]
ascribe sources status [--format text|json]
ascribe --version
```
