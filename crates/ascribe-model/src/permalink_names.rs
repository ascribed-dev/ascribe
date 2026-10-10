//! `[permalinks]`: the names `ascribe.toml` gives somewhere to lead that no
//! heading or page declares (SPEC §7.6). `[permalinks.retired]` ends a
//! permalink on purpose, with a reason; `[permalinks.aliases]` gives one
//! place a second name. What needs the content (whether an alias leads to a
//! permalink, whether `to` leads anywhere, a name the content declares too)
//! is checked with the project, in `ascribe-check`.

use ascribe_core::{Span, diagnostics};

use crate::loader::Loader;
use crate::model::{ModelPermalinks, PermalinkAlias, RetiredPermalink};
use crate::permalinks::{NAME_RULE, RETIRED_PLACEHOLDER, is_retired_placeholder};
use crate::toml_util::{V, entries, sp};

impl Loader<'_> {
    /// `[permalinks]`: its two tables, each name valid, a retired name with
    /// a reason of its own, and aliases that end somewhere.
    pub(crate) fn permalinks(&mut self, v: Option<&V<'_>>) -> ModelPermalinks {
        let mut out = ModelPermalinks::default();
        let Some(v) = v else { return out };
        let Some(t) = self.as_table("permalinks", v) else {
            return out;
        };
        self.check_keys("permalinks", t, &["retired", "aliases"]);
        if let Some(retired) = t.get("retired")
            && let Some(retired) = self.as_table("permalinks.retired", retired)
        {
            for (name, span, value) in entries(retired) {
                if !self.permalink_name(name, span) {
                    continue;
                }
                if let Some(entry) = self.retired(name, span, value) {
                    out.retired.push(entry);
                }
            }
        }
        if let Some(aliases) = t.get("aliases")
            && let Some(aliases) = self.as_table("permalinks.aliases", aliases)
        {
            for (name, span, value) in entries(aliases) {
                let path = format!("permalinks.aliases.{name}");
                let target = self.string(&path, value, false);
                if !self.permalink_name(name, span) {
                    continue;
                }
                let Some(target) = target else { continue };
                if !self.permalink_name(&target, sp(value)) {
                    continue;
                }
                out.aliases.push(PermalinkAlias {
                    name: name.to_owned(),
                    target,
                    span,
                    target_span: sp(value),
                });
            }
        }
        for alias in &out.aliases {
            if let Some(retired) = out.retired(&alias.name) {
                self.push(
                    self.issue(diagnostics::MODEL_PERMALINKS, alias.span)
                        .with_arg("name", alias.name.clone())
                        .with_related(
                            self.loc(retired.span),
                            format!("`{}` is retired here", alias.name),
                        ),
                );
            } else if out.alias_end(&alias.name).is_none() {
                self.push(
                    self.issue(diagnostics::MODEL_PERMALINKS, alias.target_span)
                        .with_variant("loop")
                        .with_arg("name", alias.name.clone()),
                );
            }
        }
        out
    }

    /// A retired permalink's entry: a table with a `reason` and maybe a
    /// `to`.
    fn retired(&mut self, name: &str, span: Span, v: &V<'_>) -> Option<RetiredPermalink> {
        let path = format!("permalinks.retired.{name}");
        let t = self.as_table(&path, v)?;
        self.check_keys(&path, t, &["reason", "to"]);
        let reason = self.require(&path, t, sp(v), "reason").and_then(|r| {
            let reason = self.string(&format!("{path}.reason"), r, true)?;
            if is_retired_placeholder(&reason) {
                self.push(
                    self.issue(diagnostics::MODEL_PERMALINKS, sp(r))
                        .with_variant("placeholder")
                        .with_arg("name", name)
                        .with_arg("reason", RETIRED_PLACEHOLDER),
                );
            }
            Some(reason)
        });
        let to = match t.get("to") {
            None => Some(None),
            Some(to) => self
                .string(&format!("{path}.to"), to, true)
                .map(|text| Some((text, sp(to)))),
        };
        let (reason, to) = (reason?, to?);
        Some(RetiredPermalink {
            name: name.to_owned(),
            reason,
            to_span: to.as_ref().map(|(_, span)| *span),
            to: to.map(|(text, _)| text),
            span,
        })
    }

    /// Whether `name` can be a permalink's name; `model-invalid-name` when
    /// it can't.
    fn permalink_name(&mut self, name: &str, span: Span) -> bool {
        let ok = ascribe_core::is_id(name);
        if !ok {
            self.push(
                self.issue(diagnostics::MODEL_INVALID_NAME, span)
                    .with_arg("name", name)
                    .with_arg("role", "permalink")
                    .with_arg("rule", NAME_RULE),
            );
        }
        ok
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ascribe_core::{DiagnosticSlug, FileId};

    fn load(permalinks: &str) -> Result<ModelPermalinks, Vec<(DiagnosticSlug, Option<String>)>> {
        let text = format!("spec = \"0.1\"\n{permalinks}");
        crate::load_str(&text, FileId::new(0))
            .map(|m| m.permalinks)
            .map_err(|issues| {
                issues
                    .into_iter()
                    .map(|i| (i.slug, i.variant.map(|v| v.to_owned())))
                    .collect()
            })
    }

    #[test]
    fn both_tables() {
        let loaded = load(
            "[permalinks.retired]\n\
             rotate-keys = { to = \"guides/security.md#keys\", reason = \"Merged into the security guide\" }\n\
             legacy-sync = { reason = \"The sync agent was removed in 4.0\" }\n\
             [permalinks.aliases]\n\
             agent-settings = \"settings\"\n\
             cli-settings = \"agent-settings\"\n",
        )
        .expect("loads");
        assert_eq!(loaded.retired.len(), 2);
        assert_eq!(
            loaded.retired[0].to.as_deref(),
            Some("guides/security.md#keys")
        );
        assert_eq!(loaded.retired[1].to, None);
        assert_eq!(
            loaded.retired("legacy-sync").map(|r| r.reason.as_str()),
            Some("The sync agent was removed in 4.0")
        );
        assert_eq!(loaded.alias_end("cli-settings"), Some("settings"));
        assert_eq!(loaded.alias_end("settings"), None);
    }

    #[test]
    fn problems() {
        let slugs = |text: &str| load(text).expect_err(text);
        assert_eq!(
            slugs("[permalinks.retired]\na = {}\n"),
            [(diagnostics::MODEL_MISSING_KEY, None)]
        );
        assert_eq!(
            slugs("[permalinks.retired]\na = { reason = \" \" }\n"),
            [(diagnostics::MODEL_EMPTY_TEXT, None)]
        );
        assert_eq!(
            slugs("[permalinks.retired]\na = { reason = \"why this permalink was retired\" }\n"),
            [(
                diagnostics::MODEL_PERMALINKS,
                Some("placeholder".to_owned())
            )]
        );
        assert_eq!(
            slugs("[permalinks.retired]\na = { reason = \"r\", from = \"x\" }\n"),
            [(diagnostics::MODEL_UNKNOWN_KEY, None)]
        );
        assert_eq!(
            slugs("[permalinks.retired]\na = \"r\"\n"),
            [(diagnostics::MODEL_WRONG_TYPE, None)]
        );
        assert_eq!(
            slugs("[permalinks.aliases]\n\"a/b\" = \"c\"\n"),
            [(diagnostics::MODEL_INVALID_NAME, None)]
        );
        assert_eq!(
            slugs("[permalinks.aliases]\na = \"b\"\nb = \"a\"\n"),
            [
                (diagnostics::MODEL_PERMALINKS, Some("loop".to_owned())),
                (diagnostics::MODEL_PERMALINKS, Some("loop".to_owned()))
            ]
        );
        assert_eq!(
            slugs("[permalinks.aliases]\na = \"a\"\n"),
            [(diagnostics::MODEL_PERMALINKS, Some("loop".to_owned()))]
        );
        assert_eq!(
            slugs(
                "[permalinks.retired]\na = { reason = \"r\" }\n[permalinks.aliases]\na = \"b\"\n"
            ),
            [(diagnostics::MODEL_PERMALINKS, None)]
        );
        assert_eq!(
            slugs("[permalinks]\nalias = {}\n"),
            [(
                diagnostics::MODEL_UNKNOWN_KEY,
                Some("suggestion".to_owned())
            )]
        );
    }
}
