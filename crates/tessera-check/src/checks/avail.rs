//! `@available` specs, in a directive or in `available` frontmatter
//! (SPEC §4.4).

use tessera_core::availability::parse_availability;
use tessera_core::{Issue, Location, Span, diagnostics};
use tessera_model::AvailabilityProblem;

use super::Ctx;
use super::text::quoted_list;

impl Ctx<'_> {
    /// Parses a spec and checks it against the content model: syntax,
    /// targets, states, versionless targets, and history order.
    ///
    /// `offset` is where `text` starts in the file. When `precise` is false
    /// the offset can't be trusted (a multi-line or escaped YAML scalar), so
    /// everything is reported at `whole`.
    pub(super) fn check_availability_text(
        &mut self,
        text: &str,
        offset: usize,
        whole: Span,
        precise: bool,
    ) {
        let at = |span: Span| Location::new(self.id, if precise { span } else { whole });
        let spec = match parse_availability(text, offset) {
            Ok(spec) => spec,
            Err(e) => {
                let issue = Issue::new(diagnostics::AVAILABLE_SYNTAX, at(e.span))
                    .with_arg("spec", text.trim().to_owned())
                    .with_arg("detail", e.detail);
                self.report(issue);
                return;
            }
        };
        let states: Vec<&str> = self
            .model
            .lifecycle
            .iter()
            .map(|s| s.name.as_str())
            .collect();
        let mut issues = Vec::new();
        for problem in self.model.check_availability(&spec) {
            issues.push(match problem {
                AvailabilityProblem::UnknownTarget(name) => {
                    Issue::new(diagnostics::AVAILABLE_UNKNOWN, at(name.span))
                        .with_arg("name", name.text)
                }
                AvailabilityProblem::UnknownState(name) => {
                    Issue::new(diagnostics::AVAILABLE_UNKNOWN, at(name.span))
                        .with_variant("state")
                        .with_arg("name", name.text)
                        .with_arg("states", quoted_list(&states))
                }
                AvailabilityProblem::VersionlessVersion { target, version } => {
                    Issue::new(diagnostics::AVAILABLE_VERSIONLESS, at(version.span))
                        .with_arg("target", target.text)
                }
                AvailabilityProblem::DimensionVersion {
                    target,
                    version,
                    example,
                } => Issue::new(diagnostics::AVAILABLE_VERSIONLESS, at(version.span))
                    .with_variant("dimension")
                    .with_arg("target", target.text)
                    .with_arg("example", example),
                AvailabilityProblem::HistoryOrder {
                    target,
                    later,
                    earlier,
                    span,
                } => Issue::new(diagnostics::AVAILABLE_HISTORY_ORDER, at(span))
                    .with_arg("target", target.text)
                    .with_arg("later", later)
                    .with_arg("earlier", earlier),
            });
        }
        for issue in issues {
            self.report(issue);
        }
    }
}
