//! Vale's presets: the configurations Ascribe ships, which `[checks.vale]
//! preset` names. A preset is text, a `.vale.ini` and its rule files, and
//! nothing is downloaded: Ascribe writes it under the project's `.ascribe/`
//! folder and runs Vale with it.

/// A preset: its name, and its files by their paths from the folder its
/// `.vale.ini` is in.
#[derive(Clone, Copy, Debug)]
pub struct Preset {
    /// The name `[checks.vale] preset` gives.
    pub name: &'static str,
    /// Each file and its text: `.vale.ini`, then the rules, each at
    /// `styles/<style>/<rule>.yml`.
    pub files: &'static [(&'static str, &'static str)],
}

/// The presets, by name.
pub const PRESETS: &[Preset] = &[Preset {
    name: "quiet",
    files: &[
        (".vale.ini", include_str!("../presets/quiet/.vale.ini")),
        (
            "styles/Ascribe/Repeated.yml",
            include_str!("../presets/quiet/styles/Ascribe/Repeated.yml"),
        ),
        (
            "styles/Ascribe/Terms.yml",
            include_str!("../presets/quiet/styles/Ascribe/Terms.yml"),
        ),
        (
            "styles/Ascribe/Typos.yml",
            include_str!("../presets/quiet/styles/Ascribe/Typos.yml"),
        ),
    ],
}];

impl Preset {
    /// The preset with this name.
    pub fn find(name: &str) -> Option<&'static Preset> {
        PRESETS.iter().find(|p| p.name == name)
    }

    /// The names of the presets.
    pub fn names() -> Vec<&'static str> {
        PRESETS.iter().map(|p| p.name).collect()
    }

    /// Its rules, as Vale names them in an alert (`Ascribe.Repeated`), in
    /// file order.
    pub fn rules(&self) -> Vec<String> {
        self.files
            .iter()
            .filter_map(|(path, _)| {
                let rest = path.strip_prefix("styles/")?;
                let (style, file) = rest.split_once('/')?;
                let rule = file.strip_suffix(".yml")?;
                Some(format!("{style}.{rule}"))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    /// Every file in a preset's folder is in its list, so none is left out
    /// of what's written, and none listed is missing.
    #[test]
    fn each_preset_lists_its_folder() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("presets");
        for preset in PRESETS {
            let mut on_disk = Vec::new();
            let root = dir.join(preset.name);
            let mut stack = vec![root.clone()];
            while let Some(d) = stack.pop() {
                for entry in std::fs::read_dir(&d).unwrap() {
                    let path = entry.unwrap().path();
                    if path.is_dir() {
                        stack.push(path);
                    } else {
                        let rel = path.strip_prefix(&root).unwrap();
                        on_disk.push(rel.to_string_lossy().replace('\\', "/"));
                    }
                }
            }
            on_disk.sort();
            let mut listed: Vec<String> =
                preset.files.iter().map(|(p, _)| (*p).to_owned()).collect();
            listed.sort();
            assert_eq!(listed, on_disk, "the files of preset `{}`", preset.name);
            assert_eq!(preset.files[0].0, ".vale.ini");
        }
    }

    #[test]
    fn rules_are_named_as_vale_names_them() {
        let quiet = Preset::find("quiet").unwrap();
        assert!(quiet.rules().contains(&"Ascribe.Repeated".to_owned()));
        assert!(Preset::find("loud").is_none());
    }
}
