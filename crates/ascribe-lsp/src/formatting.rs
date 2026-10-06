//! Document formatting through the Ascribe-aware formatter.
//!
//! It formats the editor's buffer with `ascribe_fmt::format`, which
//! `ascribe_fmt::format_files` (`ascribe fmt`) calls for each file on disk.

use ascribe_core::LineIndex;
use lsp_types::{DocumentFormattingParams, TextEdit};

use crate::nav::Ctx;

pub(crate) fn format(ctx: &Ctx, _params: DocumentFormattingParams) -> Vec<TextEdit> {
    let Some(file) = ctx.file() else {
        return Vec::new();
    };
    let options = ascribe_fmt::options_from_model(&ctx.model);
    let index = LineIndex::new(&file.source);
    ascribe_fmt::format(&file.source, &options, &ctx.model)
        .into_iter()
        .map(|edit| TextEdit {
            range: ctx.encoding.range(&index, edit.span),
            new_text: edit.new_text,
        })
        .collect()
}
