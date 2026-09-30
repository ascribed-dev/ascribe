//! Document links, CodeLens, and inlay hints (SPEC §10): every link and include
//! destination is clickable, each `@include` names the file it pulls in and
//! opens it, and a link with no text shows the title it will get.

use std::str::FromStr;

use lsp_types::{
    CodeLens, Command, DocumentLink, InlayHint, InlayHintLabel, InlayHintTooltip, Range, Uri,
};
use serde_json::json;
use tessera_core::Span;
use tessera_resolve::{RefKind, Resolution};

use crate::definition::{resolution_location, source_location};
use crate::nav::{Ctx, Lines, substitute_phrases};

/// The command a CodeLens carries. The client runs it through the server
/// (`workspace/executeCommand`), which asks the client to show the document,
/// so no client code is involved.
pub(crate) const OPEN_FILE: &str = "ascribe.openFile";

/// A `#L` fragment makes VS Code open a file at a line.
fn with_line(uri: Uri, line: u32) -> Option<Uri> {
    Uri::from_str(&format!("{}#L{}", uri.as_str(), line + 1)).ok()
}

/// Every link, image, and include destination of the file.
pub(crate) fn document_links(ctx: &Ctx) -> Vec<DocumentLink> {
    let Some(file) = ctx.file() else {
        return Vec::new();
    };
    let mut lines = Lines::new(ctx);
    let resolutions = ctx.snapshot.resolutions(&ctx.path);
    let mut out = Vec::new();
    for (reference, resolution) in file.references.iter().zip(resolutions) {
        let Some(span) = reference.destination_span else {
            continue;
        };
        let (target, tooltip) = match resolution {
            Resolution::External => (Uri::from_str(&reference.expanded_destination).ok(), None),
            other => {
                let location = resolution_location(ctx, &mut lines, other);
                let line = location.as_ref().map(|l| l.range.start.line);
                let target = location.and_then(|l| match line {
                    Some(line) if line > 0 => with_line(l.uri, line),
                    _ => Some(l.uri),
                });
                (target, Some(open_text(&reference.destination)))
            }
        };
        let Some(target) = target else {
            continue;
        };
        let Some(range) = lines.range(&ctx.path, span) else {
            continue;
        };
        out.push(DocumentLink {
            range,
            target: Some(target),
            tooltip,
            data: None,
        });
    }
    for include in &file.includes {
        let (Some(primary), Some(target)) = (include.primary, include.target.as_ref()) else {
            continue;
        };
        let Some(location) = source_location(ctx, &mut lines, target, include.section.as_deref())
        else {
            continue;
        };
        if ctx.snapshot.file(target).is_none() {
            continue;
        }
        let line = location.range.start.line;
        let Some(range) = lines.range(&ctx.path, primary) else {
            continue;
        };
        out.push(DocumentLink {
            range,
            target: if line > 0 {
                with_line(location.uri, line)
            } else {
                Some(location.uri)
            },
            tooltip: Some(open_text(&include.written)),
            data: None,
        });
    }
    out.sort_by_key(|l| (l.range.start.line, l.range.start.character));
    out
}

fn open_text(destination: &str) -> String {
    format!("Open {destination}")
}

/// A CodeLens above each `@include` that names its target and opens it.
pub(crate) fn code_lenses(ctx: &Ctx) -> Vec<CodeLens> {
    let Some(file) = ctx.file() else {
        return Vec::new();
    };
    let mut lines = Lines::new(ctx);
    let mut out = Vec::new();
    for include in &file.includes {
        let Some(target) = include.target.as_ref() else {
            continue;
        };
        let Some(target_file) = ctx.snapshot.file(target) else {
            continue;
        };
        let Some(location) = source_location(ctx, &mut lines, target, include.section.as_deref())
        else {
            continue;
        };
        let mut title = format!("Includes {target}");
        if let Some(heading) = include
            .section
            .as_deref()
            .and_then(|id| target_file.heading_by_id(id))
        {
            title.push_str(&format!(" › {}", heading.text));
        }
        let Some(range) = lines.range(&ctx.path, include.span) else {
            continue;
        };
        out.push(CodeLens {
            range,
            command: Some(Command {
                title,
                command: OPEN_FILE.to_owned(),
                arguments: Some(vec![
                    json!(location.uri.as_str()),
                    serde_json::to_value(location.range).unwrap_or_default(),
                ]),
            }),
            data: None,
        });
    }
    out
}

/// The resolved title after the `[` of each empty-text link in `range`.
pub(crate) fn inlay_hints(ctx: &Ctx, range: Range) -> Vec<InlayHint> {
    let Some(file) = ctx.file() else {
        return Vec::new();
    };
    let mut lines = Lines::new(ctx);
    let resolutions = ctx.snapshot.resolutions(&ctx.path);
    let mut out = Vec::new();
    for (reference, resolution) in file.references.iter().zip(resolutions) {
        if reference.kind != RefKind::Link || !reference.text_empty {
            continue;
        }
        let Resolution::Source { target, id, .. } = resolution else {
            continue;
        };
        let Some(title) = ctx.snapshot.title_for(target, id.as_deref()) else {
            continue;
        };
        // Just inside the `[`, where the text would be.
        let at = Span::empty(reference.span.start() + 1);
        let Some(position) = lines.position(&ctx.path, at.start()) else {
            continue;
        };
        if position < range.start || position > range.end {
            continue;
        }
        out.push(InlayHint {
            position,
            label: InlayHintLabel::String(substitute_phrases(&ctx.model, title)),
            kind: None,
            text_edits: None,
            tooltip: Some(InlayHintTooltip::String(format!(
                "The link's text is its target's title ({})",
                reference.destination
            ))),
            padding_left: Some(false),
            padding_right: Some(false),
            data: None,
        });
    }
    out
}
