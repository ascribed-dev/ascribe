//! Moving from a preset to a Vale configuration the project owns:
//! `ascribe vale eject`.
//!
//! The preset is written out at the project root as an ordinary `.vale.ini`
//! and a `.vale/` styles folder, with the rules `off` names turned off in
//! it, and `[checks.vale]` is switched to `config = ".vale.ini"`. The
//! project's vocabulary stays Ascribe's: it's written under `.ascribe/vale/`
//! and given to Vale with the project's configuration, as for any project
//! that brings its own.

use std::path::Path;

use ascribe_model::vale::Preset;
use ascribe_model::{ContentModel, ValeSource};
use toml_edit::{DocumentMut, Item, value};

use super::setup::{with_settings, write_if_changed};

/// The configuration file eject writes, from the project root.
pub const EJECTED_CONFIG: &str = ".vale.ini";

/// The styles folder eject writes, from the project root.
pub const EJECTED_STYLES: &str = ".vale";

/// What eject wrote.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ejected {
    /// The preset written out.
    pub preset: String,
    /// The files written, from the project root, `/`-separated, in the
    /// order they were written; `ascribe.toml` last.
    pub written: Vec<String>,
}

/// Why eject didn't write anything, or stopped.
#[derive(Debug, thiserror::Error)]
pub enum EjectError {
    /// `[checks.vale]` doesn't name a preset.
    #[error("eject writes out a preset, and `[checks.vale]` has {}", match .config {
        Some(config) => format!("the project's own configuration, `{config}`"),
        None => "no preset".to_owned(),
    })]
    NoPreset {
        /// The configuration it names instead, if it names one.
        config: Option<String>,
    },
    /// A file eject would write is there already.
    #[error("`{path}` is there already; eject writes it, and never over a file of the project's")]
    Exists {
        /// The file, from the project root.
        path: String,
    },
    /// A file couldn't be written.
    #[error("can't write `{path}`: {reason}")]
    Write {
        /// The file, from the project root.
        path: String,
        /// Why.
        reason: String,
    },
    /// `ascribe.toml` couldn't be edited.
    #[error("can't change `[checks.vale]` in ascribe.toml: {0}")]
    Model(String),
}

impl ascribe_core::Coded for EjectError {
    fn code(&self) -> &'static str {
        match self {
            EjectError::NoPreset { .. } => "vale_no_preset",
            EjectError::Exists { .. } => "vale_config_exists",
            EjectError::Write { .. } => "vale_unwritable",
            EjectError::Model(_) => "vale_model_unchanged",
        }
    }
}

/// The files a preset is written out as, by their paths from the project
/// root: the `.vale.ini`, with the rules in `off` turned off and its styles
/// in `.vale/`, then the rules.
pub fn ejected_files(preset: &Preset, off: &[String]) -> Vec<(String, String)> {
    preset
        .files
        .iter()
        .map(|(path, text)| {
            if *path == ".vale.ini" {
                let ini: String = text
                    .lines()
                    .map(|line| {
                        if line.trim_start().starts_with("StylesPath") {
                            format!("StylesPath = {EJECTED_STYLES}\n")
                        } else {
                            format!("{line}\n")
                        }
                    })
                    .collect();
                (EJECTED_CONFIG.to_owned(), with_settings(&ini, &[], off))
            } else {
                let rule = path.strip_prefix("styles/").unwrap_or(path);
                (format!("{EJECTED_STYLES}/{rule}"), (*text).to_owned())
            }
        })
        .collect()
}

/// Writes the project's preset out as its own Vale configuration, and
/// switches `[checks.vale]` to it. `model_text` is `ascribe.toml`'s text,
/// which is written again with `preset` and `off` replaced by `config`;
/// its other keys and comments stay as they are.
///
/// # Errors
///
/// `[checks.vale]` names no preset, a file eject writes is there already
/// (then nothing is written), or a file can't be written.
pub fn eject(root: &Path, model: &ContentModel, model_text: &str) -> Result<Ejected, EjectError> {
    let settings = model.checks.vale.as_ref();
    let (name, off) = match settings.map(|s| (&s.source, &s.off)) {
        Some((ValeSource::Preset(name), off)) => (name, off),
        Some((ValeSource::Config(config), _)) => {
            return Err(EjectError::NoPreset {
                config: Some(config.clone()),
            });
        }
        None => return Err(EjectError::NoPreset { config: None }),
    };
    let preset = Preset::find(name).ok_or(EjectError::NoPreset { config: None })?;
    for path in [EJECTED_CONFIG, EJECTED_STYLES] {
        // Outside FileSystem: Vale's configuration, which only Vale reads;
        // this asks whether the project has one, so as not to write over it.
        if root.join(path).exists() {
            return Err(EjectError::Exists {
                path: path.to_owned(),
            });
        }
    }
    let new_model = switched(model_text)?;
    let mut written = Vec::new();
    for (path, text) in ejected_files(preset, off) {
        write_if_changed(&root.join(&path), &text).map_err(|e| EjectError::Write {
            path: path.clone(),
            reason: e.to_string(),
        })?;
        written.push(path);
    }
    let model_file = crate::MODEL_FILE;
    write_if_changed(&root.join(model_file), &new_model).map_err(|e| EjectError::Write {
        path: model_file.to_owned(),
        reason: e.to_string(),
    })?;
    written.push(model_file.to_owned());
    Ok(Ejected {
        preset: preset.name.to_owned(),
        written,
    })
}

/// `ascribe.toml`'s text with `[checks.vale]`'s `preset` and `off` replaced
/// by `config = ".vale.ini"`, where `preset` was.
fn switched(model_text: &str) -> Result<String, EjectError> {
    let mut doc: DocumentMut = model_text
        .parse()
        .map_err(|e: toml_edit::TomlError| EjectError::Model(e.message().to_owned()))?;
    let vale = doc
        .get_mut("checks")
        .and_then(|checks| checks.get_mut("vale"))
        .and_then(Item::as_table_like_mut)
        .ok_or_else(|| EjectError::Model("there's no `[checks.vale]` table".to_owned()))?;
    if !vale.contains_key("preset") {
        return Err(EjectError::Model("`[checks.vale]` has no `preset`".to_owned()));
    }
    rename(vale, "preset", "config");
    if let Some(item) = vale.get_mut("config") {
        // The value's own spacing and comment, kept around the new one.
        let decor = item.as_value().map(|v| v.decor().clone());
        let mut config = value(EJECTED_CONFIG);
        if let (Some(decor), Some(v)) = (decor, config.as_value_mut()) {
            *v.decor_mut() = decor;
        }
        *item = config;
    }
    vale.remove("off");
    Ok(doc.to_string())
}

/// Renames a key in place: the same position, value, and comments.
fn rename(table: &mut dyn toml_edit::TableLike, from: &str, to: &str) {
    let keys: Vec<String> = table.iter().map(|(k, _)| k.to_owned()).collect();
    let mut entries = Vec::new();
    for key in keys {
        let decor = table
            .key(&key)
            .map(|k| (k.leaf_decor().clone(), k.dotted_decor().clone()));
        let Some(item) = table.remove(&key) else {
            continue;
        };
        let name = if key == from { to.to_owned() } else { key };
        entries.push((name, decor, item));
    }
    for (name, decor, item) in entries {
        table.insert(&name, item);
        if let (Some((leaf, dotted)), Some(mut key)) = (decor, table.key_mut(&name)) {
            *key.leaf_decor_mut() = leaf;
            *key.dotted_decor_mut() = dotted;
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    fn load(text: &str) -> ContentModel {
        ascribe_model::load_str(text, ascribe_core::FileId::new(0)).unwrap()
    }

    #[test]
    fn the_preset_is_written_out_and_the_model_switched_to_it() {
        let dir = tempfile::tempdir().unwrap();
        let text = "spec = \"0.1\"\n\n# Prose.\n[checks.vale]\npreset = \"quiet\" # the quiet one\noff = [\"Ascribe.Repeated\"]\nin-check = true\n";
        let ejected = eject(dir.path(), &load(text), text).unwrap();
        assert_eq!(ejected.written.first().unwrap(), ".vale.ini");
        assert_eq!(ejected.written.last().unwrap(), "ascribe.toml");
        let ini = std::fs::read_to_string(dir.path().join(".vale.ini")).unwrap();
        assert!(ini.contains("StylesPath = .vale\n"), "{ini}");
        assert!(ini.ends_with("Ascribe.Repeated = NO\n"), "{ini}");
        assert!(!ini.contains("Vocab"), "{ini}");
        for path in &ejected.written {
            assert!(dir.path().join(path).is_file(), "{path}");
        }
        assert!(dir.path().join(".vale/Ascribe/Repeated.yml").is_file());
        let model = std::fs::read_to_string(dir.path().join("ascribe.toml")).unwrap();
        assert_eq!(
            model,
            "spec = \"0.1\"\n\n# Prose.\n[checks.vale]\nconfig = \".vale.ini\" # the quiet one\nin-check = true\n"
        );
        let switched = load(&model);
        assert_eq!(
            switched.checks.vale.unwrap().source,
            ValeSource::Config(".vale.ini".to_owned())
        );
    }

    #[test]
    fn an_inline_table_is_switched_too() {
        let text = "spec = \"0.1\"\n[checks]\nvale = { preset = \"quiet\", in-check = true }\n";
        assert_eq!(
            switched(text).unwrap(),
            "spec = \"0.1\"\n[checks]\nvale = { config = \".vale.ini\", in-check = true }\n"
        );
    }

    #[test]
    fn nothing_is_written_over_the_project_s_own_files() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(".vale.ini"), "mine").unwrap();
        let text = "spec = \"0.1\"\n[checks.vale]\npreset = \"quiet\"\n";
        std::fs::write(dir.path().join("ascribe.toml"), text).unwrap();
        let error = eject(dir.path(), &load(text), text).unwrap_err();
        assert!(matches!(error, EjectError::Exists { ref path } if path == ".vale.ini"));
        assert_eq!(
            std::fs::read_to_string(dir.path().join("ascribe.toml")).unwrap(),
            text
        );
        assert!(!dir.path().join(".vale").exists());
    }

    #[test]
    fn a_project_without_a_preset_has_nothing_to_eject() {
        let dir = tempfile::tempdir().unwrap();
        let text = "spec = \"0.1\"\n[checks.vale]\nconfig = \"x.ini\"\n";
        let error = eject(dir.path(), &load(text), text).unwrap_err();
        assert!(matches!(error, EjectError::NoPreset { config: Some(_) }));
        let text = "spec = \"0.1\"\n";
        let error = eject(dir.path(), &load(text), text).unwrap_err();
        assert!(matches!(error, EjectError::NoPreset { config: None }));
    }
}
