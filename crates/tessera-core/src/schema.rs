//! Directive schemas (SPEC §3, §4, §6, §7.2).
//!
//! A [`DirectiveSchema`] is everything a processor needs to recognize, parse,
//! and validate one directive: its forms, primary, binding, title rule,
//! whether it groups, and its attributes. Built-in schemas come from
//! [`builtin_schemas`]; project widgets' come from the content model
//! (the loader converts each `[widgets.<name>]` table into one). The parser
//! takes the full set as input, so a declared widget is
//! "recognized, parsed, and validated exactly like a built-in directive"
//! (SPEC §6).

/// The keyword that closes a container (SPEC §3.1). It's a known keyword
/// (SPEC §3.2) but not a directive, so it has no schema.
pub const END_KEYWORD: &str = "end";

/// One directive's schema.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DirectiveSchema {
    /// The keyword, without `@`: a built-in name (no hyphen) or a widget name
    /// (at least one hyphen), per SPEC Appendix A `name`.
    pub name: String,
    /// Where the schema comes from.
    pub origin: Origin,
    /// The permitted forms (SPEC §3.5). At least one is `true`.
    pub forms: Forms,
    /// What follows the colon (SPEC §3.4). A container opener never has a
    /// primary: its colon ends the line.
    pub primary: Primary,
    /// What the line form applies to (SPEC §3.8). `Some` exactly when
    /// `forms.line` is true.
    pub binding: Option<Binding>,
    /// Whether a title line may or must sit above the directive (SPEC §3.7).
    pub title: TitleRule,
    /// Whether a run of this directive's openers forms a group of arms
    /// (SPEC §3.6). Only container-only directives are groupable.
    pub groupable: bool,
    /// The attributes the directive accepts (SPEC §3.3).
    pub attributes: Attributes,
    /// Help text for the editor's hover and completion, if any.
    pub description: Option<String>,
}

/// Where a schema comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Origin {
    /// A directive defined by SPEC §4.
    Builtin(Builtin),
    /// A project widget declared in the content model (SPEC §6).
    Widget,
}

/// The built-in directives (SPEC §4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Builtin {
    /// `@id` (§4.1).
    Id,
    /// `@include` (§4.2).
    Include,
    /// `@variant` (§4.3).
    Variant,
    /// `@available` (§4.4).
    Available,
    /// `@note` (§4.5).
    Note,
    /// `@steps` (§4.6).
    Steps,
    /// `@details` (§4.7).
    Details,
    /// `@snippet` (§4.8).
    Snippet,
}

impl Builtin {
    /// Every built-in directive, in SPEC §4 order.
    pub const ALL: [Builtin; 8] = [
        Builtin::Id,
        Builtin::Include,
        Builtin::Variant,
        Builtin::Available,
        Builtin::Note,
        Builtin::Steps,
        Builtin::Details,
        Builtin::Snippet,
    ];

    /// The keyword, without `@`.
    pub const fn name(self) -> &'static str {
        match self {
            Builtin::Id => "id",
            Builtin::Include => "include",
            Builtin::Variant => "variant",
            Builtin::Available => "available",
            Builtin::Note => "note",
            Builtin::Steps => "steps",
            Builtin::Details => "details",
            Builtin::Snippet => "snippet",
        }
    }

    /// The built-in with this keyword.
    pub fn from_name(name: &str) -> Option<Builtin> {
        Builtin::ALL.into_iter().find(|b| b.name() == name)
    }

    /// This directive's schema, as SPEC §4 defines it.
    pub fn schema(self) -> DirectiveSchema {
        let base = DirectiveSchema {
            name: self.name().to_owned(),
            origin: Origin::Builtin(self),
            forms: Forms::LINE,
            primary: Primary::None,
            binding: None,
            title: TitleRule::None,
            groupable: false,
            attributes: Attributes::Declared(Vec::new()),
            description: None,
        };
        match self {
            Builtin::Id => DirectiveSchema {
                primary: Primary::Identifier { required: true },
                binding: Some(Binding::Heading),
                description: Some("Give a heading a stable id.".into()),
                ..base
            },
            Builtin::Include => DirectiveSchema {
                primary: Primary::Identifier { required: true },
                binding: Some(Binding::SelfBound),
                attributes: Attributes::Declared(vec![AttributeSchema {
                    key: "heading".into(),
                    ty: AttributeType::Boolean,
                    required: false,
                    default: Some(DefaultValue::Boolean(true)),
                    description: Some(
                        "When false, the included section's own heading is omitted.".into(),
                    ),
                }]),
                description: Some("Transclude a file or a region.".into()),
                ..base
            },
            Builtin::Variant => DirectiveSchema {
                forms: Forms::CONTAINER,
                title: TitleRule::Accepted,
                groupable: true,
                attributes: Attributes::Dimensions,
                description: Some("Mark alternative content.".into()),
                ..base
            },
            Builtin::Available => DirectiveSchema {
                // SPEC §3.4: a line primary, the rest of the line.
                primary: Primary::Availability { required: true },
                binding: Some(Binding::HeadingOrBlock),
                description: Some("Declare where content applies.".into()),
                ..base
            },
            Builtin::Note => DirectiveSchema {
                forms: Forms::BOTH,
                primary: Primary::Text { required: false },
                binding: Some(Binding::Block),
                title: TitleRule::Accepted,
                attributes: Attributes::Declared(vec![AttributeSchema {
                    key: "type".into(),
                    ty: AttributeType::NoteType,
                    required: false,
                    default: Some(DefaultValue::Text("note".into())),
                    description: Some("The kind of callout.".into()),
                }]),
                description: Some("Callout.".into()),
                ..base
            },
            Builtin::Steps => DirectiveSchema {
                binding: Some(Binding::Block),
                description: Some("Mark an ordered list as a procedure.".into()),
                ..base
            },
            Builtin::Details => DirectiveSchema {
                forms: Forms::BOTH,
                binding: Some(Binding::Block),
                title: TitleRule::Required,
                description: Some("Collapsible content.".into()),
                ..base
            },
            Builtin::Snippet => DirectiveSchema {
                primary: Primary::Identifier { required: true },
                binding: Some(Binding::SelfBound),
                attributes: Attributes::Declared(vec![
                    AttributeSchema {
                        key: "lang".into(),
                        ty: AttributeType::String,
                        required: false,
                        default: None,
                        description: Some(
                            "The code block's language; by default, the file's extension.".into(),
                        ),
                    },
                    AttributeSchema {
                        key: "title".into(),
                        ty: AttributeType::String,
                        required: false,
                        default: None,
                        description: Some("A title for the code block.".into()),
                    },
                    AttributeSchema {
                        key: "phrases".into(),
                        ty: AttributeType::Boolean,
                        required: false,
                        default: Some(DefaultValue::Boolean(false)),
                        description: Some("Whether phrases in the code are substituted.".into()),
                    },
                ]),
                description: Some("Take a code example from a file.".into()),
                ..base
            },
        }
    }
}

/// The schemas of every built-in directive, in SPEC §4 order.
pub fn builtin_schemas() -> Vec<DirectiveSchema> {
    Builtin::ALL.into_iter().map(Builtin::schema).collect()
}

/// Which forms a directive permits (SPEC §3.5).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Forms {
    /// A single directive line with no end line.
    pub line: bool,
    /// A directive line ending in `:`, holding blocks until `@end`.
    pub container: bool,
}

impl Forms {
    /// Line form only.
    pub const LINE: Forms = Forms {
        line: true,
        container: false,
    };
    /// Container form only.
    pub const CONTAINER: Forms = Forms {
        line: false,
        container: true,
    };
    /// Both forms.
    pub const BOTH: Forms = Forms {
        line: true,
        container: true,
    };
}

/// What may follow a directive's colon (SPEC §3.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Primary {
    /// No primary. A colon followed by text is an error, except that a
    /// trailing colon with nothing after it opens a container where the
    /// directive has a container form.
    None,
    /// A single token ending at the first whitespace: a path, id, or key.
    Identifier {
        /// Whether the line form must have one.
        required: bool,
    },
    /// CommonMark inline content that continues onto following lines the way
    /// a paragraph does.
    Text {
        /// Whether the line form must have one.
        required: bool,
    },
    /// A line primary (SPEC §3.4): the rest of the directive line, trimmed.
    /// It isn't inline content, and it doesn't continue onto the next line.
    /// It holds an availability spec or feature key (SPEC §4.4). Built-in
    /// `@available` only; a content model can't declare it.
    Availability {
        /// Whether the line form must have one.
        required: bool,
    },
}

impl Primary {
    /// Whether the line form must have a primary.
    pub fn is_required(self) -> bool {
        match self {
            Primary::None => false,
            Primary::Identifier { required }
            | Primary::Text { required }
            | Primary::Availability { required } => required,
        }
    }
}

/// What a line-form directive applies to (SPEC §3.8). The names match
/// `binding` in a widget declaration in `ascribe.toml`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Binding {
    /// `"self"`: its own primary, or nothing (it stands alone), like `@include`.
    SelfBound,
    /// `"heading"`: the heading at the start of its section, and that
    /// section. It must sit at the top of the section, like `@id`.
    Heading,
    /// `"block"`: the next block in the same container. When the directive
    /// has a text primary and one is given, the primary is the content and
    /// nothing else is bound, like `@note: text`.
    Block,
    /// `"heading-or-block"`: at the top of a section, the section; anywhere
    /// else, the next block, like `@available`.
    HeadingOrBlock,
}

/// Whether a directive takes a title line (SPEC §3.7).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TitleRule {
    /// A `.` line directly above it stays ordinary text, and processors
    /// warn that it may be a misplaced title (SPEC §3.7, §8.2).
    None,
    /// A title is allowed.
    Accepted,
    /// A title is required.
    Required,
}

/// The attributes a directive accepts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Attributes {
    /// A fixed set of keys, in declared order, which is canonical order
    /// (SPEC §8.3). Empty means the directive accepts no attributes.
    Declared(Vec<AttributeSchema>),
    /// Any declared dimension name as a key, with values from that
    /// dimension; value sets allowed (SPEC §4.3). Only `@variant`. Canonical
    /// order is the content model's dimension order.
    Dimensions,
}

/// One accepted attribute key.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttributeSchema {
    /// The key (SPEC Appendix A `key`).
    pub key: String,
    /// The value's type.
    pub ty: AttributeType,
    /// Whether every use must give it. A key with a default is never required.
    pub required: bool,
    /// The value used when the key is absent.
    pub default: Option<DefaultValue>,
    /// Help text for the editor, if any.
    pub description: Option<String>,
}

/// An attribute's type (SPEC §3.3), as `ascribe.toml` declares it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AttributeType {
    /// A token or quoted string.
    String,
    /// A token matching `["-"] 1*DIGIT ["." 1*DIGIT]`.
    Number,
    /// The token `true` or `false`.
    Boolean,
    /// One of these values, compared exactly.
    Enum(Vec<String>),
    /// A value set or a single token; each member of the given type.
    Set(SetMember),
    /// A note type declared in the content model (SPEC §4.5). Built-in
    /// `@note` only.
    NoteType,
}

/// The type of a value set's members.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SetMember {
    /// Any token.
    String,
    /// One of these values.
    Enum(Vec<String>),
}

/// An attribute's default value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DefaultValue {
    /// A string, enumeration value, or number, as its text (`"lazy"`, `"600"`).
    Text(String),
    /// A boolean.
    Boolean(bool),
    /// A value set's members.
    Set(Vec<String>),
}

#[cfg(test)]
mod tests {
    use super::*;

    /// SPEC §4's table, row by row: name, forms, primary, binding.
    #[test]
    fn builtins_match_the_spec_table() {
        let table: [(&str, Forms, Primary, Option<Binding>); 8] = [
            (
                "id",
                Forms::LINE,
                Primary::Identifier { required: true },
                Some(Binding::Heading),
            ),
            (
                "include",
                Forms::LINE,
                Primary::Identifier { required: true },
                Some(Binding::SelfBound),
            ),
            ("variant", Forms::CONTAINER, Primary::None, None),
            (
                "available",
                Forms::LINE,
                Primary::Availability { required: true },
                Some(Binding::HeadingOrBlock),
            ),
            (
                "note",
                Forms::BOTH,
                Primary::Text { required: false },
                Some(Binding::Block),
            ),
            ("steps", Forms::LINE, Primary::None, Some(Binding::Block)),
            ("details", Forms::BOTH, Primary::None, Some(Binding::Block)),
            (
                "snippet",
                Forms::LINE,
                Primary::Identifier { required: true },
                Some(Binding::SelfBound),
            ),
        ];
        let schemas = builtin_schemas();
        assert_eq!(schemas.len(), table.len());
        for (schema, (name, forms, primary, binding)) in schemas.iter().zip(table) {
            assert_eq!(schema.name, name);
            assert_eq!(schema.forms, forms, "{name}");
            assert_eq!(schema.primary, primary, "{name}");
            assert_eq!(schema.binding, binding, "{name}");
            assert_eq!(Builtin::from_name(name).map(Builtin::name), Some(name));
        }
    }

    #[test]
    fn builtins_keep_the_schema_invariants() {
        for s in builtin_schemas() {
            let name = &s.name;
            assert!(s.forms.line || s.forms.container, "{name}");
            assert_eq!(s.binding.is_some(), s.forms.line, "{name}");
            assert!(!name.contains('-') && name != END_KEYWORD, "{name}");
            if s.groupable {
                assert_eq!(s.forms, Forms::CONTAINER, "{name}");
            }
            if s.forms.container {
                assert!(!s.primary.is_required(), "{name}");
            }
            if let Attributes::Declared(attrs) = &s.attributes {
                for a in attrs {
                    assert!(!(a.required && a.default.is_some()), "{name}.{}", a.key);
                }
            }
        }
    }

    #[test]
    fn titles_and_attributes() {
        let title = |b: Builtin| b.schema().title;
        assert_eq!(title(Builtin::Details), TitleRule::Required);
        assert_eq!(title(Builtin::Note), TitleRule::Accepted);
        assert_eq!(title(Builtin::Variant), TitleRule::Accepted);
        assert_eq!(title(Builtin::Steps), TitleRule::None);
        assert_eq!(Builtin::Variant.schema().attributes, Attributes::Dimensions);
        let Attributes::Declared(note) = Builtin::Note.schema().attributes else {
            panic!("@note declares its attributes");
        };
        assert_eq!(note[0].key, "type");
        assert_eq!(note[0].default, Some(DefaultValue::Text("note".into())));
        let Attributes::Declared(include) = Builtin::Include.schema().attributes else {
            panic!("@include declares its attributes");
        };
        assert_eq!(include[0].default, Some(DefaultValue::Boolean(true)));
    }
}
