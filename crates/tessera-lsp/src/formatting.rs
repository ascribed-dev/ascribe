//! Document formatting through the Ascribe-aware formatter.

use lsp_types::{DocumentFormattingParams, TextEdit};
use tessera_core::LineIndex;

use crate::nav::Ctx;

pub(crate) fn format(ctx: &Ctx, _params: DocumentFormattingParams) -> Vec<TextEdit> {
    let Some(file) = ctx.file() else {
        return Vec::new();
    };
    let options = tessera_fmt::options_from_model(&ctx.model);
    let index = LineIndex::new(&file.source);
    tessera_fmt::format(&file.source, &options, &ctx.model)
        .into_iter()
        .map(|edit| TextEdit {
            range: ctx.encoding.range(&index, edit.span),
            new_text: edit.new_text,
        })
        .collect()
}
