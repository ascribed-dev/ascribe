//! The directive reference's table of built-ins, in
//! `docs/content/_generated/directives-builtins.md`, is generated from
//! `builtin_schemas()`, and the reference's sections follow the same order,
//! SPEC §4's. This test fails when either is out of date; run it with
//! `ASCRIBE_BLESS=1` to rewrite the table. To change what a row says, change
//! the built-in's schema in `crates/ascribe-core/src/schema.rs`.

use std::fmt::Write as _;

use ascribe_core::{Builtin, DirectiveSchema, Origin, Primary, builtin_schemas};

use super::{check_fragments, escape, repo};

/// The directive reference, from `docs/content/`.
const REFERENCE: &str = "writing/directives.md";

/// What the table starts with: a memo, touching it, saying what generates it.
const HEADER: &str = "@memo: Generated from builtin_schemas() in crates/ascribe-core/src/schema.rs \
     by crates/ascribe-cli/src/docs/builtins.rs; edit the schema, then run \
     `ASCRIBE_BLESS=1 cargo test -p ascribe-cli docs` to rewrite it.\n";

/// The availability of a built-in newer than the reference: its row is marked
/// `{available="<availability>"}`. A built-in no release has yet is `ascribe
/// planned`; when a release ships it, it's `None`, since before 1.0 the docs
/// don't say which release a feature arrived in (decision #973).
fn available(builtin: Builtin) -> Option<&'static str> {
    match builtin {
        Builtin::Id
        | Builtin::Include
        | Builtin::Variant
        | Builtin::Available
        | Builtin::Note
        | Builtin::Steps
        | Builtin::Details
        | Builtin::Snippet
        | Builtin::Intended
        | Builtin::Memo
        | Builtin::Term
        | Builtin::Summary
        | Builtin::Operation => None,
    }
}

/// What a built-in's primary holds, in the table's words.
fn primary(builtin: Builtin, schema: &DirectiveSchema) -> String {
    let what = match schema.primary {
        Primary::None => return "none".to_owned(),
        Primary::Text { .. } => "text",
        Primary::Availability { .. } => "an availability spec or feature key",
        Primary::Identifier { .. } | Primary::Reason { .. } => match builtin {
            Builtin::Id => "an id",
            Builtin::Include => "a path",
            Builtin::Snippet => "an address",
            Builtin::Operation => "a link to an operation",
            Builtin::Intended => "a reason",
            Builtin::Memo => "text",
            _ => panic!(
                "say what @{}'s primary holds in crates/ascribe-cli/src/docs/builtins.rs",
                builtin.name()
            ),
        },
    };
    if schema.primary.is_required() {
        what.to_owned()
    } else {
        format!("{what}, optional")
    }
}

/// Each built-in with its schema, in SPEC §4 order.
fn builtins() -> Vec<(Builtin, DirectiveSchema)> {
    builtin_schemas()
        .into_iter()
        .map(|schema| match schema.origin {
            Origin::Builtin(builtin) => (builtin, schema),
            _ => panic!("@{} isn't a built-in", schema.name),
        })
        .collect()
}

fn table() -> String {
    let mut out = format!("{HEADER}| Directive | Forms | Primary | Purpose |\n|---|---|---|---|\n");
    for (builtin, schema) in builtins() {
        let name = builtin.name();
        let mark =
            available(builtin).map_or_else(String::new, |a| format!(" {{available=\"{a}\"}}"));
        let forms = [
            (schema.forms.line, "line"),
            (schema.forms.container, "container"),
            (schema.groupable, "grouped"),
        ]
        .into_iter()
        .filter_map(|(has, form)| has.then_some(form))
        .collect::<Vec<_>>()
        .join(", ");
        let purpose = schema
            .description
            .as_deref()
            .unwrap_or_else(|| panic!("@{name} has no description"));
        let purpose = escape(purpose.strip_suffix('.').unwrap_or(purpose));
        let primary = primary(builtin, &schema);
        writeln!(
            out,
            "| [`@{name}`](../{REFERENCE}#{name}){mark} | {forms} | {primary} | {purpose} |"
        )
        .unwrap();
    }
    out
}

#[test]
fn the_directive_reference_s_built_ins_are_current() {
    check_fragments(
        "directives-",
        &[("directives-builtins.md".to_owned(), table())],
    );

    let path = repo().join("docs/content").join(REFERENCE);
    let page = std::fs::read_to_string(&path).unwrap();
    assert!(
        page.contains("\n@include: ../_generated/directives-builtins.md\n"),
        "{REFERENCE} doesn't include _generated/directives-builtins.md"
    );

    // Each built-in has a section, `## `@name``, in the same order.
    let sections: Vec<&str> = page
        .lines()
        .filter_map(|l| l.strip_prefix("## `@")?.strip_suffix('`'))
        .collect();
    let names: Vec<&str> = builtins().iter().map(|(b, _)| b.name()).collect();
    assert_eq!(
        sections, names,
        "{REFERENCE}'s sections for the built-ins aren't SPEC §4's built-ins in its order"
    );
}
