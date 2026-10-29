//! Finding the content model a command works on, and the projects below a
//! folder that has none.

#![allow(clippy::expect_used)]

use std::fs;

use ascribe_model::MODEL_FILE;
use ascribe_workspace::{LocateError, find_below, locate};

const MODEL: &str = "spec = \"0.1\"\n[project]\ncontent-root = \"docs\"\n";

#[test]
fn locate_takes_a_file_or_the_folder_that_holds_one() {
    let dir = tempfile::tempdir().expect("a temporary directory");
    let config = dir.path().join(MODEL_FILE);
    fs::write(&config, MODEL).expect("write the model");
    assert_eq!(locate(Some(dir.path())).expect("found"), config);
    assert_eq!(locate(Some(&config)).expect("found"), config);

    let missing = dir.path().join("nowhere.toml");
    let err = locate(Some(&missing)).expect_err("not a file");
    assert!(matches!(&err, LocateError::NotAFile { path } if *path == missing));
    assert_eq!(
        err.to_string(),
        format!("{} doesn't exist or isn't a file", missing.display())
    );
}

#[test]
fn find_below_lists_the_projects_in_the_folders_below() {
    let dir = tempfile::tempdir().expect("a temporary directory");
    for folder in [
        "b",
        "a",
        "a/nested",
        "node_modules/c",
        ".hidden",
        "none/deeper",
    ] {
        fs::create_dir_all(dir.path().join(folder)).expect("create a folder");
    }
    for folder in ["b", "a", "a/nested", "node_modules/c", ".hidden"] {
        fs::write(dir.path().join(folder).join(MODEL_FILE), MODEL).expect("write the model");
    }
    let found = find_below(dir.path(), usize::MAX);
    let a = dir.path().join("a");
    assert_eq!(found, [a.clone(), a.join("nested"), dir.path().join("b")]);
    assert_eq!(find_below(dir.path(), 2), [a.clone(), a.join("nested")]);

    // The error names them; with none, it says what it said before.
    let err = LocateError::not_found(dir.path().to_owned());
    assert!(
        matches!(&err, LocateError::NotFound { below, .. } if *below == found),
        "{err:?}"
    );
    let empty = dir.path().join("none");
    assert_eq!(
        LocateError::not_found(empty.clone()).to_string(),
        format!(
            "no {MODEL_FILE} found in {} or any parent directory; run ascribe from a project, \
             or pass --config",
            empty.display()
        )
    );
}
