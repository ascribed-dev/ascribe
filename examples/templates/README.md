# Templates

A project that `ascribe templates eject how-to` gave a how-to type: the template and its guide in `templates/`, with `templates/templates.lock` saying what they were copied from, and `[types.how-to]` in `ascribe.toml`. `docs/how-to/rotate-keys.md` began as a copy of the template, with every placeholder written over, so `ascribe check` has nothing to report. `crates/ascribe-cli/tests/all/check.rs` checks that, and that the copies are still what the command writes.
