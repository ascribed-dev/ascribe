//! The checks about a page's content rather than its validity, which a
//! project can set the level of: a page's description and its recheck date,
//! read from the fields the content type gives those roles (SPEC §7.2), and
//! code blocks without a language. Beside them, a code block's info-string
//! words that look like pairs and aren't (SPEC §3.3.1), which a project
//! can't set the level of.

use ascribe_core::info_string::{self, InfoString, NearMissProblem};
use ascribe_core::{Applicability, Fix, Issue, Location, Span, TextEdit, diagnostics};
use ascribe_model::{FieldRole, FrontmatterSchema};
use ascribe_syntax::{Block, BlockKind, CodeBlock};
use serde_yaml_ng::Value;

use super::Ctx;
use crate::roles;
use crate::yaml::YamlIndex;

/// The most characters of a page's first paragraph a prompt quotes.
const MAX_OPENING: usize = 600;

/// The most lines of a code block a prompt quotes.
const MAX_CODE_LINES: usize = 5;

impl Ctx<'_> {
    /// `page-description-missing` and `recheck-overdue`, on a page whose
    /// frontmatter is `value`.
    pub(super) fn check_roles(
        &mut self,
        schema: &FrontmatterSchema,
        value: &Value,
        index: &YamlIndex,
        first_line: Span,
    ) {
        let checks = &self.model.checks;
        if let Some(field) = schema.field_with_role(FieldRole::Description)
            && roles::description_missing(checks, field, value)
        {
            let at = index
                .get(&field.name)
                .map_or(first_line, |n| n.key.unwrap_or(n.value));
            let issue = Issue::new(
                diagnostics::PAGE_DESCRIPTION_MISSING,
                Location::new(self.id, at),
            )
            .with_arg("field", field.name.clone())
            .with_arg("opening", self.opening(value));
            self.report(issue);
        }
        if let Some(field) = schema.field_with_role(FieldRole::RecheckDate)
            && let Some(due) = roles::recheck_date(field, value)
            && roles::recheck_overdue(checks, self.project.today(), due)
        {
            let at = index.get(&field.name).map_or(first_line, |n| n.value);
            let issue = Issue::new(diagnostics::RECHECK_OVERDUE, Location::new(self.id, at))
                .with_arg("date", due.to_string())
                .with_arg("field", field.name.clone());
            self.report(issue);
        }
    }

    /// How a page begins, for a prompt that asks for its description: its
    /// title and its first paragraph.
    fn opening(&self, value: &Value) -> String {
        let mut lines = Vec::new();
        if let Some(Value::String(title)) = value.get("title") {
            lines.push(format!("Title: {title}"));
        }
        if let Some(paragraph) = &self.first_paragraph {
            let mut text: String = paragraph.chars().take(MAX_OPENING).collect();
            if text.len() < paragraph.len() {
                text.push('…');
            }
            lines.push(format!("First paragraph: {text}"));
        }
        lines.join("\n")
    }

    /// `code-language-missing`: a fenced code block whose info string doesn't
    /// start with a language. Reported at the opening fence.
    pub(super) fn check_code_language(&mut self, block: &Block, code: &CodeBlock) {
        if !code.fenced || self.model.checks.is_off(diagnostics::CODE_LANGUAGE_MISSING) {
            return;
        }
        // A pair, such as `phrases=true`, isn't a language (SPEC §3.3.1), nor
        // is an attribute block.
        let info = InfoString::parse(&code.info);
        let first = info.language.unwrap_or("");
        if !first.is_empty() && !first.starts_with('{') {
            return;
        }
        let pair = info
            .language
            .is_none()
            .then(|| info.words.first().filter(|w| w.pair.is_some()))
            .flatten();
        let text = &self.file.text;
        let start = block.span.start();
        let line_end = text[start..]
            .find(['\n', '\r'])
            .map_or(text.len(), |i| start + i);
        let fence = text[start..line_end].find(['`', '~']).map(|i| start + i);
        let Some(fence) = fence else { return };
        let marker = text.as_bytes()[fence];
        let after = fence
            + text[fence..line_end]
                .bytes()
                .take_while(|b| *b == marker)
                .count();
        let lines: Vec<&str> = code.literal.lines().take(MAX_CODE_LINES).collect();
        let rest = &text[after..line_end];
        let insert = if rest.trim().is_empty() || rest.starts_with([' ', '\t']) {
            "text"
        } else {
            "text "
        };
        let mut issue = Issue::new(
            diagnostics::CODE_LANGUAGE_MISSING,
            Location::new(self.id, Span::new(fence, line_end)),
        )
        .with_arg("lines", lines.join("\n"));
        // A renderer such as comrak or Astro takes the first word as the
        // language, so it loses a pair written there.
        if let Some(word) = pair {
            issue = issue.with_variant("pair").with_arg("word", word.text);
        }
        let issue = issue.with_fix(Fix {
            title: "Mark the block as `text`".to_owned(),
            file: self.id,
            edits: vec![TextEdit::insert(after, insert)],
            // `text` is right for output, and wrong for code.
            applicability: Applicability::Unsafe,
        });
        self.report(issue);
    }
}

impl Ctx<'_> {
    /// `info-pair-malformed`: each word after a fenced code block's language
    /// that starts like a pair a processor reads and isn't one (SPEC
    /// §3.3.1), with the pair it was meant to be when that's clear.
    pub(super) fn check_info_pairs(&mut self, code: &CodeBlock) {
        let Some(span) = code.info_span.filter(|_| code.fenced) else {
            return;
        };
        let misses = info_string::near_misses(&code.info);
        if misses.is_empty() {
            return;
        }
        let Some(source) = self.file.text.get(span.range()) else {
            return;
        };
        let offsets = source_offsets(source, &code.info);
        for miss in misses {
            // With an entity reference in the info string, the word can't be
            // placed: the whole info string is reported, with no fix.
            let at = offsets.as_ref().map_or(span, |o| {
                Span::new(span.start() + o[miss.start], span.start() + o[miss.end])
            });
            let mut issue =
                Issue::new(diagnostics::INFO_PAIR_MALFORMED, Location::new(self.id, at))
                    .with_arg("key", miss.key)
                    .with_arg("written", &code.info[miss.start..miss.end]);
            if let Some(fixed) = miss
                .fixed
                .filter(|_| offsets.is_some() && miss.problem != NearMissProblem::Other)
            {
                // CommonMark takes one backslash off before the pair is
                // read, so each is written twice (SPEC §3.3.1).
                let fixed = fixed.replace('\\', "\\\\");
                issue = issue
                    .with_variant(miss.problem.as_str())
                    .with_arg("fixed", fixed.clone())
                    .with_fix(Fix {
                        title: format!("Write `{fixed}`"),
                        file: self.id,
                        edits: vec![TextEdit::replace(at, fixed)],
                        // It's what was almost certainly meant, and it
                        // changes what the block shows.
                        applicability: Applicability::Unsafe,
                    });
            }
            self.report(issue);
        }
    }
}

/// Where each byte of `info`, the info string CommonMark gives, was written
/// in `source`, its text on the fence's line, and where `info` ends: past
/// the backslash escapes CommonMark took off. `None` when `source` holds
/// what that can't place, an entity reference.
fn source_offsets(source: &str, info: &str) -> Option<Vec<usize>> {
    let bytes = source.as_bytes();
    let mut offsets = Vec::with_capacity(info.len() + 1);
    let mut read = Vec::with_capacity(info.len());
    let mut i = 0;
    while i < bytes.len() {
        let at = i;
        if bytes[i] == b'\\' && bytes.get(i + 1).is_some_and(u8::is_ascii_punctuation) {
            i += 1;
        }
        let len = source[i..].chars().next().map_or(1, char::len_utf8);
        for _ in 0..len {
            offsets.push(at);
        }
        read.extend_from_slice(&bytes[i..i + len]);
        i += len;
    }
    offsets.push(bytes.len());
    (read == info.as_bytes()).then_some(offsets)
}

/// The text of a page's first paragraph, at the top level of its blocks.
pub(super) fn first_paragraph(text: &str, blocks: &[Block]) -> Option<String> {
    blocks.iter().find_map(|b| match &b.kind {
        BlockKind::Paragraph(_) => text
            .get(b.span.range())
            .map(|p| p.split_whitespace().collect::<Vec<_>>().join(" ")),
        _ => None,
    })
}
