//! Attribute keys and value types against a schema (SPEC §3.3, §4.3, §5.3).
//!
//! The parser reports what's wrong with an attribute block's *shape*; these
//! checks report what needs a schema: keys the directive or image doesn't
//! declare, values of the wrong type, `@variant` dimensions and values, and
//! required attributes that are missing.

use tessera_core::{
    Attribute, AttributeBlock, AttributeSchema, AttributeType, AttributeValue, Attributes,
    DirectiveSchema, Fix, Issue, Location, SetMember, Span, TextEdit, diagnostics,
};

use super::Ctx;
use super::text::{is_key, list, quoted_list, suggest};

/// What an attribute block belongs to.
#[derive(Clone, Copy)]
pub(super) enum Owner<'a> {
    /// A directive.
    Directive(&'a DirectiveSchema),
    /// An image.
    Image,
}

impl Ctx<'_> {
    /// Checks a directive's attribute block, or lack of one.
    pub(super) fn check_directive_attributes(
        &mut self,
        schema: &DirectiveSchema,
        block: Option<&AttributeBlock>,
        closed: bool,
        name_span: Span,
    ) {
        // An unclosed block can't be read, so nothing more is said about
        // the line (SPEC §3.3).
        if !closed {
            return;
        }
        match &schema.attributes {
            Attributes::Dimensions => {
                if let Some(block) = block {
                    self.check_dimensions(block);
                }
            }
            Attributes::Declared(declared) => {
                if let Some(block) = block {
                    self.check_block(block, declared, Owner::Directive(schema));
                }
                for missing in required_missing(declared, block) {
                    let issue = Issue::new(
                        diagnostics::WIDGET_SCHEMA,
                        Location::new(self.id, name_span),
                    )
                    .with_variant("missing-attribute")
                    .with_arg("name", schema.name.clone())
                    .with_arg("key", missing.key.clone());
                    self.report(issue);
                }
            }
        }
    }

    /// Checks an attribute block against declared attributes: unknown keys
    /// and value types. Keys the parser already reported (invalid keys and
    /// repeats) and values it couldn't read are left alone.
    pub(super) fn check_block(
        &mut self,
        block: &AttributeBlock,
        declared: &[AttributeSchema],
        owner: Owner<'_>,
    ) {
        let mut seen: Vec<&str> = Vec::new();
        for attribute in &block.attributes {
            if seen.contains(&attribute.key.as_str()) || !is_key(&attribute.key) {
                continue;
            }
            seen.push(&attribute.key);
            let Some(schema) = declared.iter().find(|a| a.key == attribute.key) else {
                self.unknown_key(attribute, declared, owner);
                continue;
            };
            if let Some(value) = &attribute.value {
                self.check_value(&attribute.key, value, &schema.ty);
            }
        }
    }

    fn unknown_key(
        &mut self,
        attribute: &Attribute,
        declared: &[AttributeSchema],
        owner: Owner<'_>,
    ) {
        let at = Location::new(self.id, attribute.key_span);
        let keys: Vec<&str> = declared.iter().map(|a| a.key.as_str()).collect();
        let issue = Issue::new(diagnostics::ATTRIBUTE_UNKNOWN_KEY, at)
            .with_arg("key", attribute.key.clone());
        let issue = match owner {
            Owner::Image if keys.is_empty() => issue.with_variant("image-none"),
            Owner::Image => issue
                .with_variant("image")
                .with_arg("keys", quoted_list(&keys)),
            Owner::Directive(schema) => {
                let issue = issue.with_arg("name", schema.name.clone());
                if keys.is_empty() {
                    issue.with_variant("none")
                } else if let Some(s) = suggest(&attribute.key, keys.iter().copied()) {
                    let fix = Fix {
                        title: format!("Rename the attribute to `{s}`"),
                        file: self.id,
                        edits: vec![TextEdit::replace(attribute.key_span, s.to_owned())],
                    };
                    issue
                        .with_variant("suggestion")
                        .with_arg("suggestion", s.to_owned())
                        .with_fix(fix)
                } else {
                    issue.with_arg("keys", quoted_list(&keys))
                }
            }
        };
        self.report(issue);
    }

    /// One value against its declared type (SPEC §3.3: types come from the
    /// schema, never from how a value looks).
    fn check_value(&mut self, key: &str, value: &AttributeValue, ty: &AttributeType) {
        let at = Location::new(self.id, value.span());
        let base =
            Issue::new(diagnostics::ATTRIBUTE_TYPE_MISMATCH, at).with_arg("key", key.to_owned());
        let shown = match value {
            AttributeValue::Set { members, .. } => members
                .iter()
                .map(|t| t.text.as_str())
                .collect::<Vec<_>>()
                .join("|"),
            other => other.as_text().unwrap_or_default().to_owned(),
        };
        let is_set = matches!(value, AttributeValue::Set { .. });
        if is_set && !matches!(ty, AttributeType::Set(_)) {
            self.report(base.with_variant("set"));
            return;
        }
        match ty {
            AttributeType::String => {}
            AttributeType::Number => {
                if !is_number(&shown) {
                    self.report(
                        base.with_arg("expected", "a number such as 600")
                            .with_arg("value", shown),
                    );
                }
            }
            AttributeType::Boolean => {
                if shown != "true" && shown != "false" {
                    self.report(base.with_variant("boolean").with_arg("value", shown));
                }
            }
            AttributeType::Enum(values) => {
                if !values.contains(&shown) {
                    self.report(enum_mismatch(base, shown, values));
                }
            }
            AttributeType::NoteType => {
                let types: Vec<String> = self.model.notes.iter().map(|n| n.name.clone()).collect();
                if !types.contains(&shown) {
                    self.report(enum_mismatch(base, shown, &types));
                }
            }
            AttributeType::Set(SetMember::String) => {}
            AttributeType::Set(SetMember::Enum(values)) => {
                let members: Vec<(&str, Span)> = match value {
                    AttributeValue::Set { members, .. } => {
                        members.iter().map(|t| (t.text.as_str(), t.span)).collect()
                    }
                    other => other
                        .as_text()
                        .map(|t| vec![(t, other.span())])
                        .unwrap_or_default(),
                };
                for (text, span) in members {
                    if !values.iter().any(|v| v == text) {
                        let mut issue = enum_mismatch(base.clone(), text.to_owned(), values);
                        issue.location = Location::new(self.id, span);
                        self.report(issue);
                    }
                }
            }
        }
    }

    /// `@variant`'s attributes: each key a declared dimension, each value a
    /// declared value of it (SPEC §4.3).
    fn check_dimensions(&mut self, block: &AttributeBlock) {
        let mut seen: Vec<&str> = Vec::new();
        for attribute in &block.attributes {
            if seen.contains(&attribute.key.as_str()) || !is_key(&attribute.key) {
                continue;
            }
            seen.push(&attribute.key);
            let members: Vec<(String, Span)> = match &attribute.value {
                Some(AttributeValue::Set { members, .. }) => {
                    members.iter().map(|t| (t.text.clone(), t.span)).collect()
                }
                Some(other) => other
                    .as_text()
                    .map(|t| vec![(t.to_owned(), other.span())])
                    .unwrap_or_default(),
                None => Vec::new(),
            };
            self.check_dimension(&attribute.key, attribute.key_span, &members);
        }
    }

    /// One dimension name and its values, from an arm's attributes or from
    /// `variant` frontmatter.
    pub(super) fn check_dimension(
        &mut self,
        name: &str,
        key_span: Span,
        values: &[(String, Span)],
    ) {
        let Some(dimension) = self.model.dimension(name) else {
            let names: Vec<&str> = self
                .model
                .dimensions
                .iter()
                .map(|d| d.name.as_str())
                .collect();
            let issue = Issue::new(
                diagnostics::VARIANT_UNKNOWN,
                Location::new(self.id, key_span),
            )
            .with_arg("dimension", name.to_owned())
            .with_arg("dimensions", quoted_list(&names));
            self.report(issue);
            return;
        };
        let declared: Vec<String> = dimension.values.iter().map(|v| v.value.clone()).collect();
        for (value, span) in values {
            if !declared.contains(value) {
                let issue = Issue::new(diagnostics::VARIANT_UNKNOWN, Location::new(self.id, *span))
                    .with_variant("value")
                    .with_arg("value", value.clone())
                    .with_arg("dimension", name.to_owned())
                    .with_arg("values", quoted_list(&declared));
                self.report(issue);
            }
        }
    }
}

fn enum_mismatch(base: Issue, value: String, values: &[String]) -> Issue {
    base.with_variant("enum")
        .with_arg("value", value)
        .with_arg("values", list(values))
}

/// The declared attributes that are required and not given. A key given with
/// no value counts as given (the parser reported it).
pub(super) fn required_missing<'a>(
    declared: &'a [AttributeSchema],
    block: Option<&AttributeBlock>,
) -> Vec<&'a AttributeSchema> {
    declared
        .iter()
        .filter(|a| a.required && block.is_none_or(|b| b.get(&a.key).is_none()))
        .collect()
}

/// SPEC §3.3 / content-model.md §6: `["-"] 1*DIGIT ["." 1*DIGIT]`.
fn is_number(text: &str) -> bool {
    let text = text.strip_prefix('-').unwrap_or(text);
    let (whole, fraction) = match text.split_once('.') {
        Some((w, f)) => (w, Some(f)),
        None => (text, None),
    };
    let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    digits(whole) && fraction.is_none_or(digits)
}

#[cfg(test)]
mod tests {
    use super::is_number;

    #[test]
    fn numbers() {
        for ok in ["600", "-3", "1.5", "0"] {
            assert!(is_number(ok), "{ok}");
        }
        for bad in ["600px", "", "-", "1.", ".5", "1.2.3", "+1", "1e3"] {
            assert!(!is_number(bad), "{bad}");
        }
    }
}
