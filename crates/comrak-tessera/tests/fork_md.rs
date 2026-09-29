//! TESSERA: checks that `FORK.md` lists every file Ascribe changed, with the
//! number of `// TESSERA:` markers each one has.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Files that are wholly Ascribe's. They have no upstream counterpart, so
/// they aren't in the changed-locations table.
const TESSERA_FILES: &[&str] = &["src/tessera.rs", "src/parser/tessera.rs"];

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// Marker counts per upstream file, from the source.
fn markers_in_source(root: &Path) -> BTreeMap<String, usize> {
    let mut files = Vec::new();
    rust_files(&root.join("src"), &mut files);
    let mut counts = BTreeMap::new();
    for path in files {
        let rel = path
            .strip_prefix(root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        if TESSERA_FILES.contains(&rel.as_str()) {
            continue;
        }
        let text = std::fs::read_to_string(&path).unwrap();
        let count = text.matches("// TESSERA:").count();
        if count > 0 {
            counts.insert(rel, count);
        }
    }
    counts
}

/// Marker counts per file, from the table rows in `FORK.md` that start with
/// a file name: "| `src/lib.rs` | 1 | ...".
fn markers_in_fork_md(root: &Path) -> BTreeMap<String, usize> {
    let text = std::fs::read_to_string(root.join("FORK.md")).unwrap();
    let mut counts = BTreeMap::new();
    for line in text.lines() {
        let cells: Vec<&str> = line.split('|').map(str::trim).collect();
        if cells.len() < 3 {
            continue;
        }
        let (file, count) = (cells[1], cells[2]);
        let Some(file) = file.strip_prefix("`src/").and_then(|f| f.strip_suffix('`')) else {
            continue;
        };
        counts.insert(format!("src/{file}"), count.parse().unwrap());
    }
    counts
}

#[test]
fn fork_md_lists_every_marked_file() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source = markers_in_source(root);
    assert!(!source.is_empty());
    assert_eq!(
        markers_in_fork_md(root),
        source,
        "FORK.md's changed-locations table (left) doesn't match the `// TESSERA:` markers in src/ (right)"
    );
}
