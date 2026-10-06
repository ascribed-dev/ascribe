//! Parse options.

use std::collections::HashMap;
use std::sync::Arc;

use ascribe_core::{DirectiveSchema, END_KEYWORD, FileId, Primary, builtin_schemas};
use comrak_ascribe::ascribe::AscribeOptions;

/// What the parser needs to know about the project.
///
/// The directive schemas decide which `@keywords` are directives: the
/// built-in directives plus the project's widgets (SPEC §3.2, §6). Each
/// schema also says what kind of primary its directive takes (SPEC §3.4),
/// which decides how the line and the lines after it are read.
#[derive(Clone, Debug)]
pub struct ParseOptions {
    /// The id given to the file's locations in issues. The default is
    /// `FileId::new(0)`; whoever owns the file table sets the real one.
    pub file: FileId,
    /// The directive schemas: [`builtin_schemas`] plus one per project
    /// widget. A later schema with the same name is ignored.
    pub schemas: Vec<DirectiveSchema>,
    /// The note types the project declares (SPEC §4.5), used to suggest
    /// `@note {type=warning}:` for a misspelled `@warning:`. Defaults to the
    /// five built-in types.
    pub note_types: Vec<String>,
}

impl ParseOptions {
    /// Options with these schemas, the built-in note types, and file id 0.
    pub fn new(schemas: Vec<DirectiveSchema>) -> ParseOptions {
        ParseOptions {
            file: FileId::new(0),
            schemas,
            note_types: ["note", "tip", "important", "warning", "caution"]
                .map(String::from)
                .to_vec(),
        }
    }

    /// Sets the file id used in issue locations.
    pub fn with_file(mut self, file: FileId) -> ParseOptions {
        self.file = file;
        self
    }

    /// Sets the project's note types.
    pub fn with_note_types(mut self, note_types: Vec<String>) -> ParseOptions {
        self.note_types = note_types;
        self
    }

    /// The schema of a directive, by keyword.
    pub(crate) fn schema(&self, name: &str) -> Option<&DirectiveSchema> {
        self.schemas.iter().find(|s| s.name == name)
    }

    /// The keyword set for the block parser: every schema, and `end`.
    pub(crate) fn keywords(&self) -> Arc<AscribeOptions> {
        let mut seen: HashMap<&str, ()> = HashMap::new();
        let mut keywords = AscribeOptions::new();
        for schema in &self.schemas {
            if seen.insert(&schema.name, ()).is_none() && schema.name != END_KEYWORD {
                keywords.insert(
                    schema.name.clone(),
                    matches!(schema.primary, Primary::Text { .. }),
                );
            }
        }
        keywords.insert(END_KEYWORD, false);
        Arc::new(keywords)
    }
}

impl Default for ParseOptions {
    /// The built-in directives only.
    fn default() -> ParseOptions {
        ParseOptions::new(builtin_schemas())
    }
}
