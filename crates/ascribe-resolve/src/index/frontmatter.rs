//! Phrase candidates in the frontmatter fields that take phrases (SPEC
//! §5.1): those the build substitutes (`build/phrases.rs`), found in the
//! frontmatter's text so that the editor can find, show, and rename them.
//!
//! The text is read line by line, by indentation
//! ([`ascribe_syntax::frontmatter_lines`]): a field's value is what follows
//! its key on its line, and the lines indented under it (or, for a list, its
//! `-` items, or a block scalar's lines), comments left out. That's how
//! frontmatter is written; a value in a form this doesn't follow has no
//! candidates listed.

use ascribe_core::Span;
use ascribe_model::{ContentModel, Field, FieldType};

use super::{FileIndex, PhrasePlace, PhraseUse};

/// The declared phrase candidates in the fields of the frontmatter at
/// `content` (a span of `source`) that take phrases, of the page's type's
/// `fields` (`None` when it has no type).
pub(crate) fn phrases(
    source: &str,
    content: Span,
    fields: Option<&[Field]>,
    model: &ContentModel,
) -> Vec<PhraseUse> {
    let Some(fields) = fields else {
        return Vec::new();
    };
    if !fields.iter().any(takes_phrases) {
        return Vec::new();
    }
    let Some(text) = source.get(content.range()) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    // The keys the current line is under, each with its indentation.
    let mut stack: Vec<(usize, Option<&Field>)> = Vec::new();
    for line in ascribe_syntax::frontmatter_lines(text) {
        let value_at = content.start() + line.value_start;
        if let Some(key) = line.key {
            while stack.last().is_some_and(|(i, _)| *i >= line.indent) {
                stack.pop();
            }
            let candidates: &[Field] = match stack.last() {
                None => fields,
                Some((_, Some(parent))) => subfields(&parent.ty),
                Some((_, None)) => &[],
            };
            let field = candidates.iter().find(|f| f.name == key);
            if let Some(field) = field.filter(|f| f.phrases) {
                scan(line.value, value_at, field, model, &mut out);
            }
            stack.push((line.indent, field));
            continue;
        }
        // A list's item belongs to the key at or above its `-`; any other
        // line, a block scalar's included, to the key it's indented under.
        let owner = line.item.unwrap_or(line.indent);
        while stack
            .last()
            .is_some_and(|(i, _)| *i > owner || (line.item.is_none() && *i >= owner))
        {
            stack.pop();
        }
        if let Some((_, Some(field))) = stack.last()
            && field.phrases
        {
            scan(line.value, value_at, field, model, &mut out);
        }
    }
    out
}

/// A field, or one of its fields at any depth, takes phrases.
fn takes_phrases(field: &Field) -> bool {
    field.phrases || subfields(&field.ty).iter().any(takes_phrases)
}

/// The fields of an object, or of a list's objects.
fn subfields(ty: &FieldType) -> &[Field] {
    match ty {
        FieldType::Object(fields) => fields,
        FieldType::List(inner) => subfields(inner),
        _ => &[],
    }
}

/// The declared candidates in `value`, which starts at `offset`: outside
/// code spans, in a field read with inline markup.
fn scan(value: &str, offset: usize, field: &Field, model: &ContentModel, out: &mut Vec<PhraseUse>) {
    for phrase in ascribe_syntax::code_phrases(value) {
        let start = phrase.span.start();
        if field.inline.is_some() && value[..start].matches('`').count() % 2 == 1 {
            continue;
        }
        if !model.has_phrase(&phrase.key) {
            continue;
        }
        out.push(PhraseUse {
            phrase: ascribe_syntax::Phrase {
                key: phrase.key,
                key_span: Span::new(
                    offset + phrase.key_span.start(),
                    offset + phrase.key_span.end(),
                ),
                span: Span::new(offset + start, offset + phrase.span.end()),
            },
            declared: true,
            place: PhrasePlace::Frontmatter,
        });
    }
}

/// The line of a top-level key of the frontmatter, from the key to the end
/// of its value on that line.
pub(crate) fn key_line(file: &FileIndex, key: &str) -> Option<Span> {
    let content = file.document.frontmatter.as_ref()?.content;
    let text = file.source.get(content.range())?;
    let mut at = content.start();
    for line in text.split_inclusive('\n') {
        if line
            .strip_prefix(key)
            .is_some_and(|rest| rest.trim_start().starts_with(':'))
        {
            return Some(Span::new(at, at + line.trim_end().len()));
        }
        at += line.len();
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    const MODEL: &str = "spec = \"0.1\"\n\n[phrases]\nproduct = \"Quill\"\n\n[types.page.frontmatter]\ntitle = { type = \"string\", phrases = true }\ndescription = \"string?\"\nkeywords = { type = \"list(string)?\", phrases = true }\ncode = { type = \"string?\", phrases = true, inline = \"code\" }\n";

    fn found(frontmatter: &str) -> Vec<String> {
        let model = ascribe_model::load_str(MODEL, ascribe_core::FileId::new(0)).expect("model");
        let source = format!("---\n{frontmatter}---\n");
        let content = Span::new(4, 4 + frontmatter.len());
        let fields = model
            .type_named(Some("page"))
            .found()
            .map(|t| t.frontmatter.fields.as_slice());
        phrases(&source, content, fields, &model)
            .into_iter()
            .map(|p| {
                assert_eq!(&source[p.phrase.key_span.range()], p.phrase.key);
                format!("{}@{}", p.phrase.key, p.phrase.span.start())
            })
            .collect()
    }

    #[test]
    fn candidates_are_found_in_the_fields_that_take_phrases() {
        assert_eq!(
            found("title: Use {product}\ndescription: Not {product}\n"),
            ["product@15"]
        );
        assert_eq!(
            found("title: \"{product} and {nope}\"\n"),
            ["product@12"],
            "declared only"
        );
        assert_eq!(
            found("keywords:\n- {product}\n  - x\ndescription: |\n  {product}\n"),
            ["product@16"]
        );
        assert_eq!(found("keywords:\n  - a\n  - {product}\n"), ["product@24"]);
        assert_eq!(found("title: >\n  Use {product}\n"), ["product@19"]);
        assert_eq!(found("code: \"`{product}` {product}\"\n"), ["product@23"]);
        assert_eq!(
            found("title: |\n  Note: {product}\n"),
            ["product@21"],
            "a block scalar's line is text, not a key"
        );
        assert_eq!(
            found("title: Use {product} # not {product}\n"),
            ["product@15"],
            "not in a comment"
        );
    }
}
