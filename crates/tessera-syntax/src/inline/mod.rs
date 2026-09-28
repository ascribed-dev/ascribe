//! Phase 07: Tessera's inline extensions (SPEC §5.1 and §5.3).
//!
//! [`extend`] runs over the finished tree, after every span is final, and
//! adds what comrak's inline parser doesn't know:
//!
//! - **Phrase candidates** (`{key}`, SPEC §5.1) in text, split out of the
//!   [`InlineKind::Text`] that holds them as [`InlineKind::Phrase`] nodes.
//!   Recognition works on the *source*, so an escaped `\{` (which comrak has
//!   already decoded to `{`) is told apart from a real brace, and an entity
//!   that decodes to a brace (`&#123;`) is never a candidate. See [`phrase`].
//! - **Candidates in link destinations**, and in fenced code that opts in
//!   with `phrases=true`, which aren't text nodes.
//! - **Attribute blocks after images** (SPEC §5.3), parsed with
//!   `tessera_core::parse_attribute_block`. The fork's inline parser skips the
//!   block, so its contents never become emphasis or links; this finds the
//!   same block with the fork's own scan. See [`image`].
//!
//! Code spans, indented code, raw HTML, and fences without `phrases=true`
//! hold no candidates because they hold no [`InlineKind::Text`].

mod image;
mod phrase;

use tessera_core::{FileId, Issue};

use crate::tree::*;

/// Adds phrase candidates and image attribute blocks to `blocks`, reporting
/// what's wrong with an attribute block in `issues`. Returns the `\{key}`
/// escapes it found ([`ParsedDocument::escaped_phrases`]).
pub(crate) fn extend(
    source: &str,
    file: FileId,
    blocks: &mut [Block],
    issues: &mut Vec<Issue>,
) -> Vec<Phrase> {
    let mut pass = Pass {
        source,
        file,
        issues,
        escaped: Vec::new(),
    };
    pass.blocks(blocks);
    pass.escaped.sort_by_key(|p| p.span.start());
    pass.escaped
}

struct Pass<'a> {
    source: &'a str,
    file: FileId,
    issues: &'a mut Vec<Issue>,
    escaped: Vec<Phrase>,
}

impl Pass<'_> {
    fn blocks(&mut self, blocks: &mut [Block]) {
        for block in blocks {
            self.block(block);
        }
    }

    fn block(&mut self, block: &mut Block) {
        let span = block.span;
        match &mut block.kind {
            BlockKind::Heading(h) => self.inlines(&mut h.inlines),
            BlockKind::Paragraph(p) => self.inlines(&mut p.inlines),
            BlockKind::CodeBlock(c) => self.code_block(span, c),
            BlockKind::BlockQuote(q) => self.blocks(&mut q.children),
            BlockKind::List(l) => {
                for item in &mut l.items {
                    self.blocks(&mut item.children);
                }
            }
            BlockKind::Table(t) => {
                for cell in t.rows.iter_mut().flat_map(|r| &mut r.cells) {
                    self.inlines(&mut cell.inlines);
                }
            }
            BlockKind::Directive(line) => self.directive(line),
            BlockKind::Container(c) => {
                self.directive(&mut c.opener);
                self.blocks(&mut c.children);
            }
            BlockKind::Group(g) => {
                for arm in &mut g.arms {
                    self.directive(&mut arm.opener);
                    // `Arm::title` is the same as `opener.title`.
                    arm.title.clone_from(&arm.opener.title);
                    self.blocks(&mut arm.children);
                }
            }
            BlockKind::Title(t) => self.inlines(&mut t.inlines),
            BlockKind::HtmlBlock(_) | BlockKind::ThematicBreak | BlockKind::End(_) => {}
        }
    }

    /// A directive line's text primary and its title line (SPEC §3.7).
    fn directive(&mut self, line: &mut DirectiveLine) {
        if let Some(title) = &mut line.title {
            self.inlines(&mut title.inlines);
        }
        if let Some(PrimaryValue::Text(primary)) = &mut line.primary {
            self.inlines(&mut primary.inlines);
        }
    }

    /// Walks a run of inlines, in place.
    fn inlines(&mut self, inlines: &mut Vec<Inline>) {
        let old = std::mem::take(inlines);
        for mut inline in old {
            match &mut inline.kind {
                InlineKind::Text(_) => {
                    self.split_text(inline, inlines);
                    continue;
                }
                InlineKind::Emphasis(children) | InlineKind::Strong(children) => {
                    self.inlines(children);
                }
                InlineKind::Link(link) => {
                    // An autolink's text is its destination, not prose.
                    // SPEC-QUESTION(Q43): no candidates there.
                    if link.form != LinkForm::Autolink {
                        link.destination_phrases = self.destination_phrases(inline.span, link.form);
                        self.inlines(&mut link.children);
                    }
                }
                InlineKind::Image(image) => {
                    image.destination_phrases = self.destination_phrases(inline.span, image.form);
                    self.inlines(&mut image.children);
                    self.image_attributes(&mut inline);
                }
                InlineKind::Code(_)
                | InlineKind::Html(_)
                | InlineKind::SoftBreak
                | InlineKind::HardBreak
                | InlineKind::Phrase(_) => {}
            }
            inlines.push(inline);
        }
    }
}
