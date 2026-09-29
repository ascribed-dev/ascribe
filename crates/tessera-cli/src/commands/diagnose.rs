//! What `ascribe check` and `ascribe build` share: choosing builds, and the
//! diagnostics reported for them. One implementation, so the two commands
//! print the same report.

use tessera_check::{Diagnostic, Project, check_all_builds, check_builds};
use tessera_model::Build;

/// The builds named by `--build` (repeatable), in the order given and each
/// once; every build of the content model when none is named.
///
/// # Errors
///
/// A message, ready to print, when a name isn't a build of the content model.
pub fn select_builds<'p>(project: &'p Project, names: &[String]) -> Result<Vec<&'p Build>, String> {
    let all = &project.model().builds;
    if names.is_empty() {
        return Ok(all.iter().collect());
    }
    let mut out: Vec<&Build> = Vec::new();
    for name in names {
        let build = all.iter().find(|b| &b.name == name).ok_or_else(|| {
            let known: Vec<&str> = all.iter().map(|b| b.name.as_str()).collect();
            format!(
                "the content model has no build `{name}`; its builds are {}",
                known.join(", ")
            )
        })?;
        if !out.iter().any(|b| b.name == build.name) {
            out.push(build);
        }
    }
    Ok(out)
}

/// The diagnostics of the selected builds, and the builds. With none named,
/// that's every build and also the content no build publishes; each problem is
/// listed once, and the builds it appears in are added to its message when
/// they aren't all of the selected ones.
///
/// # Errors
///
/// A message, ready to print, when a name isn't a build of the content model.
pub fn diagnose<'p>(
    project: &'p Project,
    names: &[String],
) -> Result<(Vec<Diagnostic>, Vec<&'p Build>), String> {
    let builds = select_builds(project, names)?;
    let mut diagnostics = if names.is_empty() {
        check_all_builds(project)
    } else {
        check_builds(project, &builds)
    };
    for d in &mut diagnostics {
        if let Some(note) = d.builds_note(builds.len()) {
            d.message = format!("{} ({note})", d.message);
        }
    }
    Ok((diagnostics, builds))
}
