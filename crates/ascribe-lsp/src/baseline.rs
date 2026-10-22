//! `ascribe.baseline` in the editor (#790): the advice it lists isn't
//! published, as `ascribe check` doesn't report it, and an entry that
//! matches nothing is published on the file, as advice.
//!
//! The server checks a file at a time, so it holds back a file's
//! diagnostics at a time ([`hold`]): an identity's order is among the
//! findings of one file and one check, so that's the same as holding back
//! the whole project's at once. What's held back is kept for the file
//! ([`crate::core::Core`]'s `held`), with the entry that holds it in its
//! data, so the hover on its line says so, and the quick fix that moves it
//! to an `@intended` can find it ([`actions`]): a held finding isn't a
//! diagnostic, so it has no other place in the editor.
//!
//! An entry is stale when no finding of the last round of each file, nor
//! of the checks across the project, matches it ([`published`]). The
//! editor checks its own build, so it says so only when that build is the
//! project's every build, as `ascribe check` says so only when it checks
//! every build; and never for Vale's, which it runs on open documents only.

use std::collections::{HashMap, HashSet};

use ascribe_check::baseline::{self, BASELINE_FILE, Baseline, BaselineError, Held};
use ascribe_check::identity::{Identity, identities, identities_in};
use ascribe_check::{Diagnostic, Project, Registry};
use ascribe_core::{LineIndex, RelPath, Span, diagnostics};
use ascribe_model::ContentModel;
use ascribe_resolve::FileSystem;
use lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, DiagnosticSeverity, Position, Range, TextEdit,
    WorkspaceEdit,
};

use crate::compute::to_lsp;
use crate::fsx::LayerFs;
use crate::intended;
use crate::nav::Ctx;
use crate::position::Encoding;

/// The quick fix's title.
pub(crate) const TITLE: &str = "Move out of the baseline, mark as intended, and write why";

/// `ascribe.baseline` as the editor has it: its open buffer over the disk.
#[derive(Debug, Default, PartialEq)]
pub(crate) struct Now {
    /// Its text, when there's a file.
    pub text: Option<String>,
    /// It read, or the line that doesn't parse, when there's a file.
    pub read: Option<Result<Baseline, BaselineError>>,
}

impl Now {
    /// The baseline as `fs` has it now.
    pub(crate) fn of(fs: &LayerFs) -> Now {
        let text = RelPath::parse(BASELINE_FILE)
            .ok()
            .and_then(|path| fs.read_file(&path).ok())
            .map(|bytes| String::from_utf8_lossy(&bytes).into_owned());
        let read = text.as_deref().map(Baseline::parse);
        Now { text, read }
    }

    /// The baseline, when there's one that reads. One that doesn't holds
    /// nothing back: its problem is published on the file instead.
    pub(crate) fn usable(&self) -> Option<&Baseline> {
        self.read.as_ref().and_then(|r| r.as_ref().ok())
    }
}

/// `diagnostics`, all of one file of `project`, with the advice the
/// baseline lists set aside.
pub(crate) fn hold(now: &Now, project: &Project, diagnostics: Vec<Diagnostic>) -> Held {
    match now.usable() {
        Some(baseline) => {
            let found = identities(project, &diagnostics);
            baseline::hold(baseline, diagnostics, found)
        }
        None => Held {
            diagnostics,
            ..Held::default()
        },
    }
}

/// `diagnostics`, all located in `file`, from the project's root, whose
/// text is `text`, with the advice the baseline lists set aside: Vale's,
/// which the server checks without a project.
pub(crate) fn hold_in(now: &Now, file: &str, text: &str, diagnostics: Vec<Diagnostic>) -> Held {
    match now.usable() {
        Some(baseline) => {
            let found = identities_in(file, text, &diagnostics);
            baseline::hold(baseline, diagnostics, found)
        }
        None => Held {
            diagnostics,
            ..Held::default()
        },
    }
}

/// `held`'s findings as the protocol has them, positions in `encoding`
/// over `index`, each with the entry that holds it back in its data, under
/// `baseline`. A related place isn't kept: only the hover shows them.
pub(crate) fn held_to_lsp(
    held: &Held,
    index: &LineIndex,
    encoding: Encoding,
) -> Vec<lsp_types::Diagnostic> {
    held.held
        .iter()
        .zip(&held.held_as)
        .map(|(d, identity)| {
            let mut lsp = to_lsp(d, index, encoding, &|_| None);
            if let (Some(map), Ok(value)) = (
                lsp.data.as_mut().and_then(serde_json::Value::as_object_mut),
                serde_json::to_value(identity),
            ) {
                map.insert("baseline".to_owned(), value);
            }
            lsp
        })
        .collect()
}

/// What's published on `ascribe.baseline`: the line that doesn't parse, as
/// an error; or, when `every_build` says the editor's build is all the
/// project has, an advice `baseline-entry-unmatched` at each entry no
/// finding `matched`, apart from Vale's.
pub(crate) fn published(
    now: &Now,
    model: &ContentModel,
    matched: &HashSet<&Identity>,
    every_build: bool,
    encoding: Encoding,
) -> Vec<lsp_types::Diagnostic> {
    let (Some(text), Some(read)) = (&now.text, &now.read) else {
        return Vec::new();
    };
    let index = LineIndex::new(text);
    match read {
        Err(e) => {
            let line = u32::try_from(e.line.saturating_sub(1)).unwrap_or(u32::MAX);
            let end = text.lines().nth(e.line.saturating_sub(1)).map_or(0, |l| {
                encoding.range(&index, Span::new(0, l.len())).end.character
            });
            vec![lsp_types::Diagnostic {
                range: Range::new(Position::new(line, 0), Position::new(line, end)),
                severity: Some(DiagnosticSeverity::ERROR),
                code: Some(lsp_types::NumberOrString::String(
                    "baseline_invalid".to_owned(),
                )),
                source: Some("ascribe".to_owned()),
                message: format!(
                    "{}; until it's fixed, the editor holds nothing back",
                    e.message
                ),
                ..lsp_types::Diagnostic::default()
            }]
        }
        Ok(_) if !every_build => Vec::new(),
        Ok(baseline) => {
            let prose = diagnostics::PROSE.as_str();
            baseline::stale(&model.checks, baseline, matched, &[prose])
                .iter()
                .map(|d| to_lsp(d, &index, encoding, &|_| None))
                .collect()
        }
    }
}

/// Whether the editor's build, with its other lines when it's versioned,
/// is every build the project has: what the editor checks then covers what
/// `ascribe check` does, so an entry it matches nothing to is stale.
pub(crate) fn every_build(model: &ContentModel) -> bool {
    let build = model.editor_default_build();
    model.builds.iter().all(|b| {
        b.name == build.name || (b.line.is_some() && b.declared_name() == build.declared_name())
    })
}

/// What the hover on `line` of the requested file adds for the findings
/// the baseline holds back there.
pub(crate) fn hover_text(ctx: &Ctx, line: u32) -> Option<String> {
    let found: Vec<String> = ctx
        .held
        .iter()
        .filter(|d| d.range.start.line <= line && line <= d.range.end.line)
        .map(|d| {
            let code = match &d.code {
                Some(lsp_types::NumberOrString::String(code)) => format!(" `{code}`"),
                _ => String::new(),
            };
            format!("- {}{code}", d.message)
        })
        .collect();
    if found.is_empty() {
        return None;
    }
    let what = if found.len() == 1 {
        "this advice"
    } else {
        "these advice"
    };
    Some(format!(
        "**`{BASELINE_FILE}` holds back {what}:**\n\n{}\n\nTo keep one for good, use its quick fix, *{TITLE}*.",
        found.join("\n")
    ))
}

/// The quick fix on each held finding between `start` and `end` of the
/// requested file, a review check's: its entry taken out of the baseline,
/// and an acknowledgement written where the check's place says, as the
/// `@intended` quick fix writes it, with a reason for the author to write.
// `Uri` hashes by its text; the map is only built and handed back.
#[allow(clippy::mutable_key_type)]
pub(crate) fn actions(ctx: &Ctx, range: Range) -> Vec<CodeActionOrCommand> {
    let Some(file) = ctx.file() else {
        return Vec::new();
    };
    let (Some(text), Some(baseline)) = (&ctx.baseline.text, ctx.baseline.usable()) else {
        return Vec::new();
    };
    let Some(root) = ctx.config.parent() else {
        return Vec::new();
    };
    let Some(baseline_uri) = crate::uri::path_to_uri(&root.join(BASELINE_FILE)) else {
        return Vec::new();
    };
    let index = LineIndex::new(&file.source);
    let baseline_index = LineIndex::new(text);
    let mut actions = Vec::new();
    for d in &ctx.held {
        if d.range.end < range.start || range.end < d.range.start {
            continue;
        }
        let data = d.data.as_ref();
        let Some(identity) = data
            .and_then(|data| data.get("baseline"))
            .and_then(|v| serde_json::from_value::<Identity>(v.clone()).ok())
        else {
            continue;
        };
        let Some(place) = Registry::global()
            .entries()
            .find(|e| e.slug.as_str() == identity.check)
            .and_then(|e| e.place)
        else {
            continue;
        };
        let Some(removal) = baseline::removal(text, baseline, &identity) else {
            continue;
        };
        let subject = data
            .and_then(|data| data.get("subject"))
            .and_then(serde_json::Value::as_str);
        let at = ctx
            .encoding
            .offset_lenient(&index, &file.source, d.range.start);
        let mark = intended::mark(
            place,
            &identity.check,
            &file.source,
            &file.document,
            at,
            subject,
            &ctx.model_text,
        );
        let (uri, edit) = match mark {
            Some(intended::Mark::Source(edit)) => (ctx.uri_of(&ctx.path), (edit, &index)),
            Some(intended::Mark::Model(edit)) => (
                crate::uri::path_to_uri(&ctx.config),
                (edit, &LineIndex::new(&ctx.model_text)),
            ),
            None => continue,
        };
        let Some(uri) = uri else {
            continue;
        };
        let (edit, edit_index) = edit;
        let mut changes = HashMap::new();
        changes.insert(
            uri,
            vec![TextEdit {
                range: ctx.encoding.range(edit_index, edit.span),
                new_text: edit.new_text,
            }],
        );
        changes.insert(
            baseline_uri.clone(),
            vec![TextEdit {
                range: ctx.encoding.range(&baseline_index, removal),
                new_text: String::new(),
            }],
        );
        actions.push(CodeActionOrCommand::CodeAction(CodeAction {
            title: TITLE.to_owned(),
            kind: Some(CodeActionKind::QUICKFIX),
            diagnostics: None,
            edit: Some(WorkspaceEdit {
                changes: Some(changes),
                ..WorkspaceEdit::default()
            }),
            command: None,
            is_preferred: None,
            disabled: None,
            data: None,
        }));
    }
    actions
}
