#![allow(clippy::unwrap_used)]

use ascribe_core::{FileId, Issue, Location, diagnostics};
use ascribe_query::report::{Links, NotRun, Problems};

use super::*;

fn broken_link() -> Diagnostic {
    Diagnostic::from_issue(
        &Issue::new(
            diagnostics::LINK_EXTERNAL_BROKEN,
            Location::new(FileId::new(1), 0..1),
        )
        .with_arg("url", "https://gone.example/")
        .with_arg("status", "404 Not Found"),
    )
}

#[test]
fn findings_fail_only_at_the_level_asked_for() {
    let report = Report {
        links: Some(Ran::Done(Links {
            diagnostics: vec![broken_link()],
            ..Links::default()
        })),
        ..Report::default()
    };
    assert_eq!(exit_code(&report, None), exit::OK);
    assert_eq!(exit_code(&report, Some(Level::Advice)), exit::PROBLEMS);
    assert_eq!(exit_code(&report, Some(Level::Warning)), exit::OK);
}

#[test]
fn a_section_that_didn_t_run_is_never_a_pass() {
    let report = Report {
        problems: Some(Problems::default()),
        agents: Some(Ran::NotRun(NotRun {
            reason: "no site".to_owned(),
            how: "give one".to_owned(),
        })),
        ..Report::default()
    };
    assert_eq!(exit_code(&report, None), exit::OK);
    assert_eq!(exit_code(&report, Some(Level::Error)), exit::FAILURE);
}
