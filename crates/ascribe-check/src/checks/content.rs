//! The checks about a page's content rather than its validity, which a
//! project can set the level of: a page's description and its review date,
//! read from the fields the content type gives those roles (SPEC §7.2), and
//! code blocks without a language.

use ascribe_core::{Applicability, Fix, Issue, Location, Span, TextEdit, diagnostics};
use ascribe_model::{FieldRole, FrontmatterSchema};
use ascribe_syntax::{Block, BlockKind, CodeBlock};
use serde_yaml_ng::Value;

use super::Ctx;
use crate::yaml::YamlIndex;

/// The most characters of a page's first paragraph a prompt quotes.
const MAX_OPENING: usize = 600;

/// The most lines of a code block a prompt quotes.
const MAX_CODE_LINES: usize = 5;

impl Ctx<'_> {
    /// `page-description-missing` and `review-overdue`, on a page whose
    /// frontmatter is `value`.
    pub(super) fn check_roles(
        &mut self,
        schema: &FrontmatterSchema,
        value: &Value,
        index: &YamlIndex,
        first_line: Span,
    ) {
        let checks = &self.model.checks;
        if !checks.is_off(diagnostics::PAGE_DESCRIPTION_MISSING)
            && let Some(field) = schema.field_with_role(FieldRole::Description)
            // A required field's absence is already `frontmatter-missing-field`,
            // and a default is a description.
            && !field.required
            && field.default.is_none()
        {
            let missing = match value.get(field.name.as_str()) {
                None | Some(Value::Null) => true,
                Some(Value::String(s)) => s.trim().is_empty(),
                // Another type is `frontmatter-type-mismatch`.
                Some(_) => false,
            };
            if missing {
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
        }
        if let Some(today) = self.project.today()
            && !checks.is_off(diagnostics::REVIEW_OVERDUE)
            && let Some(field) = schema.field_with_role(FieldRole::ReviewDate)
            && let Some(Value::String(text)) = value.get(field.name.as_str())
            && let Some(due) = ascribe_core::Date::parse(text)
            && due < today
        {
            let at = index.get(&field.name).map_or(first_line, |n| n.value);
            let issue = Issue::new(diagnostics::REVIEW_OVERDUE, Location::new(self.id, at))
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
        let first = code.info.split_whitespace().next().unwrap_or("");
        // `phrases=true` and an attribute block aren't a language.
        if !first.is_empty() && !first.contains('=') && !first.starts_with('{') {
            return;
        }
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
        let issue = Issue::new(
            diagnostics::CODE_LANGUAGE_MISSING,
            Location::new(self.id, Span::new(fence, line_end)),
        )
        .with_arg("lines", lines.join("\n"))
        .with_fix(Fix {
            title: "Mark the block as `text`".to_owned(),
            file: self.id,
            edits: vec![TextEdit::insert(after, insert)],
            // `text` is right for output, and wrong for code.
            applicability: Applicability::Unsafe,
        });
        self.report(issue);
    }
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
