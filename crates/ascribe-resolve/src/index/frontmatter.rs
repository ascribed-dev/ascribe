//! Phrase candidates in the frontmatter fields that take phrases (SPEC
//! §5.1): those the build substitutes (`build/phrases.rs`), found in the
//! frontmatter's text so that the editor can find, show, and rename them.
//!
//! The text is read line by line, by indentation: a field's value is what
//! follows its key on its line, and the lines indented under it (or, for a
//! list, its `-` items). That's how frontmatter is written; a value in a
//! form this doesn't follow has no candidates listed.

use ascribe_core::Span;
use ascribe_model::{ContentModel, Field, FieldType, TypeMatch};

use super::{PhrasePlace, PhraseUse};

/// The declared phrase candidates in the fields of the frontmatter at
/// `content` (a span of `source`) that take phrases.
pub(crate) fn phrases(
    source: &str,
    content: Span,
    path: &str,
    model: &ContentModel,
) -> Vec<PhraseUse> {
    let fields = match model.type_for(path) {
        TypeMatch::One(t) => &t.frontmatter.fields,
        _ => return Vec::new(),
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
    let mut at = content.start();
    for line in text.split_inclusive('\n') {
        let start = at;
        at += line.len();
        let line = line.trim_end_matches(['\n', '\r']);
        let rest = line.trim_start();
        if rest.is_empty() || rest.starts_with('#') {
            continue;
        }
        let indent = line.len() - rest.len();
        let (indent, rest, item) = match rest.strip_prefix('-') {
            Some(after) if after.is_empty() || after.starts_with(' ') => {
                let after = after.trim_start();
                (line.len() - after.len(), after, Some(indent))
            }
            _ => (indent, rest, None),
        };
        let value_at = |value: &str| start + (line.len() - value.len());
        if let Some((key, value)) = key_value(rest) {
            while stack.last().is_some_and(|(i, _)| *i >= indent) {
                stack.pop();
            }
            let candidates: &[Field] = match stack.last() {
                None => fields,
                Some((_, Some(parent))) => subfields(&parent.ty),
                Some((_, None)) => &[],
            };
            let field = candidates.iter().find(|f| f.name == key);
            if let Some(field) = field.filter(|f| f.phrases) {
                scan(value, value_at(value), field, model, &mut out);
            }
            stack.push((indent, field));
            continue;
        }
        // A list's item belongs to the key at or above its `-`; any other
        // line, to the key it's indented under.
        let owner = item.unwrap_or(indent);
        while stack
            .last()
            .is_some_and(|(i, _)| *i > owner || (item.is_none() && *i >= owner))
        {
            stack.pop();
        }
        if let Some((_, Some(field))) = stack.last()
            && field.phrases
        {
            scan(rest, value_at(rest), field, model, &mut out);
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

/// `key: value`, with the key unquoted.
fn key_value(line: &str) -> Option<(&str, &str)> {
    let colon = line.find(':')?;
    let after = &line[colon + 1..];
    if !(after.is_empty() || after.starts_with(' ')) {
        return None;
    }
    let key = line[..colon].trim().trim_matches(['"', '\'']);
    Some((key, after.trim_start()))
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

#[cfg(test)]
mod tests {
    use super::*;

    const MODEL: &str = "spec = \"0.1\"\n\n[phrases]\nproduct = \"Quill\"\n\n[types.page]\ndefault = true\n\n[types.page.frontmatter]\ntitle = { type = \"string\", phrases = true }\ndescription = \"string?\"\nkeywords = { type = \"list(string)?\", phrases = true }\ncode = { type = \"string?\", phrases = true, inline = \"code\" }\n";

    fn found(frontmatter: &str) -> Vec<String> {
        let model = ascribe_model::load_str(MODEL, ascribe_core::FileId::new(0)).expect("model");
        let source = format!("---\n{frontmatter}---\n");
        let content = Span::new(4, 4 + frontmatter.len());
        phrases(&source, content, "a.md", &model)
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
    }
}
