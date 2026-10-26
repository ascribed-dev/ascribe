//! The command a project runs Ascribe with, as what `sync` writes and the
//! hook says name it: `npx ascribe`, `pnpm exec ascribe`, or `yarn ascribe`
//! when the project installs it from npm, else `ascribe`.

use std::path::Path;

use ascribe_core::path::relative_path;

/// `ascribe`, found on the path.
pub const PLAIN: &str = "ascribe";

/// The packages whose dependency makes a project run Ascribe from its
/// `node_modules`: `@ascribed/astro` depends on `@ascribed/cli`.
const PACKAGES: &[&str] = &["@ascribed/cli", "@ascribed/astro"];

/// Each lockfile, and the command a project with it runs Ascribe with.
const LOCKFILES: &[(&str, &str)] = &[
    ("pnpm-lock.yaml", "pnpm exec ascribe"),
    ("yarn.lock", "yarn ascribe"),
    ("package-lock.json", "npx ascribe"),
];

/// The command for a package that has no lockfile.
const NO_LOCKFILE: &str = "npx ascribe";

/// The command the project in `here` runs Ascribe with: the nearest
/// `package.json` from `here` up to `root` that depends on `@ascribed/cli`
/// or `@ascribed/astro` gives the command of the lockfile beside it, or
/// above it up to `root`, as a workspace has one; without such a
/// `package.json`, `ascribe`.
pub fn detect(here: &Path, root: &Path) -> String {
    for dir in here.ancestors() {
        if depends(&dir.join("package.json")) {
            return lockfile_command(dir, root).to_owned();
        }
        if dir == root || relative_path(root, dir).is_none() {
            break;
        }
    }
    PLAIN.to_owned()
}

/// Whether the `package.json` at `path` depends on one of [`PACKAGES`].
fn depends(path: &Path) -> bool {
    // Outside FileSystem: the `package.json` around the project, which
    // isn't one of its files.
    let Ok(text) = std::fs::read_to_string(path) else {
        return false;
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
        return false;
    };
    ["dependencies", "devDependencies", "optionalDependencies"]
        .iter()
        .any(|key| PACKAGES.iter().any(|name| value[key].get(name).is_some()))
}

/// The command of the nearest lockfile from `dir` up to `root`.
fn lockfile_command(dir: &Path, root: &Path) -> &'static str {
    for lock_dir in dir.ancestors() {
        for (file, command) in LOCKFILES {
            // Outside FileSystem: the repository's lockfiles, which aren't
            // the project's.
            if lock_dir.join(file).is_file() {
                return command;
            }
        }
        if lock_dir == root || relative_path(root, lock_dir).is_none() {
            break;
        }
    }
    NO_LOCKFILE
}

/// Whether `command` can be written into a hook's command line as it is,
/// in double quotes, for every shell a hook runs in: letters, digits,
/// spaces, and `.`, `_`, `/`, `@`, `:`, `+`, `=`, and `-`.
pub fn is_safe(command: &str) -> bool {
    !command.trim().is_empty()
        && command
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || " ._/@:+=-".contains(c))
}

/// `text` with each command it shows in backticks, `` `ascribe …` ``, run
/// with `command` instead.
pub fn with_command(text: &str, command: &str) -> String {
    if command == PLAIN {
        return text.to_owned();
    }
    text.replace("`ascribe ", &format!("`{command} "))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    fn project(files: &[(&str, &str)]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        for (path, text) in files {
            let path = dir.path().join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, text).unwrap();
        }
        dir
    }

    const CLI: &str = r#"{"devDependencies": {"@ascribed/cli": "^0.9.0"}}"#;
    const ASTRO: &str = r#"{"dependencies": {"@ascribed/astro": "^0.9.0"}}"#;

    #[test]
    fn the_lockfile_beside_the_package_names_the_command() {
        for (lockfile, command) in [
            ("package-lock.json", "npx ascribe"),
            ("pnpm-lock.yaml", "pnpm exec ascribe"),
            ("yarn.lock", "yarn ascribe"),
        ] {
            let dir = project(&[("package.json", CLI), (lockfile, "")]);
            assert_eq!(detect(dir.path(), dir.path()), command, "{lockfile}");
        }
        let dir = project(&[("package.json", ASTRO)]);
        assert_eq!(detect(dir.path(), dir.path()), "npx ascribe");
    }

    #[test]
    fn a_workspace_has_its_lockfile_above_the_package() {
        let dir = project(&[
            ("pnpm-lock.yaml", ""),
            ("package.json", "{}"),
            ("docs/package.json", CLI),
        ]);
        let docs = dir.path().join("docs");
        assert_eq!(detect(&docs, dir.path()), "pnpm exec ascribe");
    }

    #[test]
    fn without_the_dependency_its_ascribe() {
        let dir = project(&[]);
        assert_eq!(detect(dir.path(), dir.path()), "ascribe");
        let dir = project(&[
            ("package.json", r#"{"dependencies": {"astro": "^5"}}"#),
            ("package-lock.json", ""),
        ]);
        assert_eq!(detect(dir.path(), dir.path()), "ascribe");
    }

    #[test]
    fn nothing_above_the_root_counts() {
        let dir = project(&[("package.json", CLI), ("repo/docs/ascribe.toml", "")]);
        let root = dir.path().join("repo");
        assert_eq!(detect(&root.join("docs"), &root), "ascribe");
    }

    #[test]
    fn commands_in_backticks_are_rewritten() {
        let text = "Run `ascribe check <file>`, then `ascribe model`. Edit `ascribe.toml`.";
        assert_eq!(
            with_command(text, "npx ascribe"),
            "Run `npx ascribe check <file>`, then `npx ascribe model`. Edit `ascribe.toml`."
        );
        assert_eq!(with_command(text, "ascribe"), text);
    }

    #[test]
    fn a_command_with_a_shells_characters_is_refused() {
        assert!(is_safe("bunx ascribe"));
        assert!(is_safe("node_modules/.bin/ascribe"));
        for unsafe_ in ["", "  ", "a\"b", "$(x)", "a;b", "a`b", "a'b", "a\\b"] {
            assert!(!is_safe(unsafe_), "{unsafe_}");
        }
    }
}
