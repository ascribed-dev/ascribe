//! The outline of the Tessera nodes the structure pass makes (phase 06):
//! directives with their titles, bindings, and children, and groups with
//! their arms. `syntax.rs` writes the CommonMark blocks and calls
//! [`node`] for the rest.

use tessera_conformance::outline::normalize_ws;
use tessera_conformance::{Arm, AttrValue, Attributes, Binding, Directive, Form, Group, Node};
use tessera_core::AttributeValue;
use tessera_syntax::{Block, BlockKind, Bound, DirectiveLine, PrimaryValue, TitleLine, raw_text};

use super::syntax::text;

/// The outline node for a Tessera block, or `None` for an end line that
/// closes nothing (it only has diagnostics).
pub fn node(source: &str, block: &Block, children: &dyn Fn(&[Block]) -> Vec<Node>) -> Option<Node> {
    match &block.kind {
        BlockKind::Directive(line) => Some(Node::Directive(directive(
            source,
            line,
            Form::Line,
            Vec::new(),
        ))),
        BlockKind::Container(container) => Some(Node::Directive(directive(
            source,
            &container.opener,
            Form::Container,
            children(&container.children),
        ))),
        BlockKind::Group(group) => Some(Node::Group(Group {
            name: group.name.clone(),
            arms: group
                .arms
                .iter()
                .map(|arm| Arm {
                    attributes: attributes(&arm.opener),
                    title: title(source, arm.title.as_ref()),
                    children: children(&arm.children),
                })
                .collect(),
        })),
        _ => None,
    }
}

fn title(source: &str, title: Option<&TitleLine>) -> Option<String> {
    title.map(|t| normalize_ws(&raw_text(source, t.content)))
}

fn attributes(line: &DirectiveLine) -> Attributes {
    let mut attributes = Attributes::new();
    if let Some(block) = &line.attributes {
        for a in &block.attributes {
            let Some(value) = &a.value else { continue };
            let value = match value {
                AttributeValue::Set { members, .. } => {
                    AttrValue::Set(members.iter().map(|m| m.text.clone()).collect())
                }
                v => AttrValue::Single(v.as_text().unwrap_or_default().to_owned()),
            };
            attributes.entry(a.key.clone()).or_insert(value);
        }
    }
    attributes
}

fn directive(source: &str, line: &DirectiveLine, form: Form, children: Vec<Node>) -> Directive {
    let primary = match &line.primary {
        Some(PrimaryValue::Text(p)) => Some(text(source, p.span)),
        Some(PrimaryValue::Identifier(p)) => Some(p.text.clone()),
        Some(PrimaryValue::Line(p)) => Some(p.text.clone()),
        Some(PrimaryValue::Unexpected(_)) | None => None,
    };
    let binding = line.binding.map(|b| match b {
        Bound::Own => Binding::SelfBinding,
        Bound::Heading => Binding::Heading,
        Bound::FollowingBlock => Binding::FollowingBlock,
        Bound::Unbound => Binding::Unbound,
    });
    Directive {
        name: line.name.clone(),
        form,
        attributes: attributes(line),
        primary,
        title: title(source, line.title.as_ref()),
        binding,
        children,
    }
}
