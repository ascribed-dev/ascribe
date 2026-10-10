<!-- Generated from tests/conformance/diagnostics.toml by tests/conformance/tests/docs.rs. Edit the registry, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-conformance --test docs`. -->

### Prose, through Vale

#### ASC148 `prose`

Advice · file level · next step: write · configurable in `[checks]` · [SPEC §2.1]({repo}/blob/main/SPEC.md#21-files)

**Message:** \{rule}: \{message}

**Fix:** Vale reported this about the prose, under the rule the message starts with. Change the text, or take the replacement the editor offers when the rule gives one; for text from a phrase, change the phrase's value in `ascribe.toml`. A rule that's wrong for the project is turned off in Vale's configuration, or, for the `quiet` preset, in `[checks.vale] off`; one place is quieted with Vale's own comments (`<!-- vale Rule = NO -->`). See [Prose, through Vale](../reference/../guides/vale.md).

#### ASC149 `prose-not-checked`

Advice · file level · next step: outside · configurable in `[checks]` · [SPEC §2.1]({repo}/blob/main/SPEC.md#21-files)

**Message:** the prose wasn't checked: `{command}` couldn't be run (\{reason}). Install Vale, or set `[checks.vale] command` to where it is

**Fix:** Install [Vale](https://vale.sh) 3 or later so that the command `[checks.vale] command` names (`vale` by default) runs, or fix what Vale's own message says about its configuration. Nothing else is checked differently while it can't run. A project that checks prose only in CI can turn this off on other machines with `prose-not-checked = "off"` in `[checks]`.
