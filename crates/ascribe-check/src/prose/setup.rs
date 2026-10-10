//! What Vale is run with: the files Ascribe writes under the project's
//! `.ascribe/vale/` folder.
//!
//! - **A preset** (`[checks.vale] preset`) is written to
//!   `.ascribe/vale/<preset>/`: its `.vale.ini`, with the project's
//!   vocabulary and the rules `off` names turned off, and its styles.
//! - **The vocabulary**, the project's own words, is a Vale vocabulary named
//!   `Ascribe` (`styles/config/vocabularies/Ascribe/accept.txt`): in the
//!   preset's styles, or, for a project's own configuration, in
//!   `.ascribe/vale/vocabulary/`, beside a `.vale.ini` that only turns it on
//!   and that Vale reads after the project's.
//!
//! Every file there is Ascribe's, written again when its text should
//! change; nothing the project wrote is ever changed.

use std::io;
use std::path::{Path, PathBuf};

use ascribe_model::vale::Preset;
use ascribe_model::{ContentModel, ValeSettings, ValeSource};

/// The folder under the project root that holds what Vale runs with.
pub const FOLDER: &str = ".ascribe/vale";

/// The name of the vocabulary of the project's own words.
pub const VOCABULARY: &str = "Ascribe";

/// Writes what Vale runs with, and returns the configuration files to give
/// it, in order.
///
/// # Errors
///
/// What's wrong, as a sentence: the project's configuration isn't there, or
/// a file couldn't be written.
pub fn prepare(
    root: &Path,
    model: &ContentModel,
    settings: &ValeSettings,
) -> Result<Vec<PathBuf>, String> {
    let written = |e: io::Error| format!("can't write its configuration under {FOLDER}: {e}");
    let words = vocabulary(model);
    let base = root.join(FOLDER);
    match &settings.source {
        ValeSource::Preset(name) => {
            let Some(preset) = Preset::find(name) else {
                return Err(format!("there's no preset `{name}`"));
            };
            let folder = base.join(preset.name);
            for (path, text) in preset_files(preset, &settings.off) {
                write_if_changed(&folder.join(path), &text).map_err(written)?;
            }
            write_if_changed(&folder.join(vocabulary_file()), &words).map_err(written)?;
            Ok(vec![folder.join(".vale.ini")])
        }
        ValeSource::Config(config) => {
            let own = root.join(config);
            // Outside FileSystem: Vale's configuration, which Vale reads; this
            // only asks whether it's there, since Vale says nothing when a
            // file it's given to read is missing.
            if !own.is_file() {
                return Err(format!("its configuration `{config}` isn't there"));
            }
            let folder = base.join("vocabulary");
            let ini = format!("StylesPath = styles\nVocab = {VOCABULARY}\n");
            write_if_changed(&folder.join(".vale.ini"), &ini).map_err(written)?;
            write_if_changed(&folder.join(vocabulary_file()), &words).map_err(written)?;
            Ok(vec![own, folder.join(".vale.ini")])
        }
    }
}

/// The vocabulary's file, from a styles folder's parent.
fn vocabulary_file() -> String {
    format!("styles/config/vocabularies/{VOCABULARY}/accept.txt")
}

/// A preset's files, by their paths from its folder: its `.vale.ini` with
/// the vocabulary turned on and the rules in `off` turned off, then its
/// rules.
pub fn preset_files(preset: &Preset, off: &[String]) -> Vec<(String, String)> {
    preset
        .files
        .iter()
        .map(|(path, text)| {
            let text = if *path == ".vale.ini" {
                with_settings(text, &[format!("Vocab = {VOCABULARY}")], off)
            } else {
                (*text).to_owned()
            };
            ((*path).to_owned(), text)
        })
        .collect()
}

/// A `.vale.ini` with `global` lines added before its first section, and a
/// `<rule> = NO` line for each rule in `off` at the end of its last.
pub fn with_settings(ini: &str, global: &[String], off: &[String]) -> String {
    let first_section = ini
        .match_indices('\n')
        .map(|(i, _)| i + 1)
        .find(|&i| ini[i..].starts_with('['))
        .unwrap_or(ini.len());
    let mut out = String::new();
    let head = ini[..first_section].trim_end();
    out.push_str(head);
    out.push('\n');
    for line in global {
        out.push_str(line);
        out.push('\n');
    }
    if first_section < ini.len() {
        out.push('\n');
        out.push_str(ini[first_section..].trim_end());
        out.push('\n');
    }
    for rule in off {
        out.push_str(&format!("{rule} = NO\n"));
    }
    out
}

/// The project's own words, one to a line, as Vale vocabulary entries:
/// phrase values, glossary terms and their aliases, and the labels of
/// dimensions, their values, and features. Each is a regular expression to
/// Vale, so its special characters are escaped.
pub fn vocabulary(model: &ContentModel) -> String {
    let mut words: Vec<&str> = Vec::new();
    words.extend(model.phrases.iter().map(|p| p.value.as_str()));
    for term in &model.glossary.terms {
        words.push(&term.term);
        words.extend(term.aliases.iter().map(String::as_str));
    }
    for dimension in &model.dimensions {
        words.push(&dimension.label);
        words.extend(dimension.values.iter().map(|v| v.label.as_str()));
    }
    words.extend(model.features.iter().map(|f| f.name.as_str()));
    let mut seen = std::collections::BTreeSet::new();
    let mut out = String::new();
    for word in words {
        let word = word.split_whitespace().collect::<Vec<_>>().join(" ");
        if word.is_empty() || !seen.insert(word.clone()) {
            continue;
        }
        out.push_str(&escape_regex(&word));
        out.push('\n');
    }
    out
}

fn escape_regex(word: &str) -> String {
    let mut out = String::with_capacity(word.len());
    for c in word.chars() {
        if "\\.+*?()|[]{}^$#&-~".contains(c) {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

/// Writes `text` to `path` unless it's there already, through a file beside
/// it, so a reader never sees half of it.
pub(super) fn write_if_changed(path: &Path, text: &str) -> io::Result<()> {
    // Outside FileSystem: a file Ascribe writes under `.ascribe/`, read to
    // see whether it needs writing.
    if std::fs::read_to_string(path).is_ok_and(|current| current == text) {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let temporary = path.with_extension(format!("tmp-{}", std::process::id()));
    std::fs::write(&temporary, text)?;
    std::fs::rename(&temporary, path).inspect_err(|_| {
        let _ = std::fs::remove_file(&temporary);
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn settings_go_before_the_first_section_and_off_at_the_end() {
        let ini = "StylesPath = styles\n\n[*.md]\nBasedOnStyles = Ascribe\n";
        let out = with_settings(
            ini,
            &["Vocab = Ascribe".to_owned()],
            &["Ascribe.Repeated".to_owned()],
        );
        assert_eq!(
            out,
            "StylesPath = styles\nVocab = Ascribe\n\n[*.md]\nBasedOnStyles = Ascribe\nAscribe.Repeated = NO\n"
        );
    }

    #[test]
    fn the_vocabulary_is_the_project_s_words_escaped() {
        let model = ascribe_model::load_str(
            "spec = \"0.1\"\n[phrases]\nproduct = \"Acme Cloud\"\nlang = \"C++\"\nsame = \"Acme Cloud\"\n\
             [dimensions.platform]\nlabel = \"Platform\"\nvalues = [\"linux\"]\n",
            ascribe_core::FileId::new(0),
        )
        .unwrap();
        let words = vocabulary(&model);
        assert_eq!(words, "Acme Cloud\nC\\+\\+\nPlatform\nlinux\n");
    }

    #[test]
    fn a_preset_and_its_vocabulary_are_written_once() {
        let dir = tempfile::tempdir().unwrap();
        let model = ascribe_model::load_str(
            "spec = \"0.1\"\n[phrases]\nproduct = \"Acme\"\n",
            ascribe_core::FileId::new(0),
        )
        .unwrap();
        let settings = ValeSettings {
            source: ValeSource::Preset("quiet".to_owned()),
            command: "vale".to_owned(),
            in_check: false,
            max_level: ascribe_model::CheckLevel::Error,
            off: Vec::new(),
            span: ascribe_core::Span::empty(0),
        };
        let configs = prepare(dir.path(), &model, &settings).unwrap();
        let ini = dir.path().join(".ascribe/vale/quiet/.vale.ini");
        assert_eq!(configs, [ini.clone()]);
        assert!(std::fs::read_to_string(&ini).unwrap().contains("Vocab = Ascribe"));
        let accept = dir
            .path()
            .join(".ascribe/vale/quiet/styles/config/vocabularies/Ascribe/accept.txt");
        assert_eq!(std::fs::read_to_string(&accept).unwrap(), "Acme\n");

        let own = ValeSettings {
            source: ValeSource::Config(".vale.ini".to_owned()),
            ..settings
        };
        assert_eq!(
            prepare(dir.path(), &model, &own).unwrap_err(),
            "its configuration `.vale.ini` isn't there"
        );
        std::fs::write(dir.path().join(".vale.ini"), "").unwrap();
        let configs = prepare(dir.path(), &model, &own).unwrap();
        assert_eq!(
            configs,
            [
                dir.path().join(".vale.ini"),
                dir.path().join(".ascribe/vale/vocabulary/.vale.ini")
            ]
        );
    }
}
