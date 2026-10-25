//! The thread of OpenAPI's phase 1, by running the binary on a copy of
//! `examples/astro-site`: its guide links to `getLoom` in `[apis.loom]`'s
//! spec, `ascribe check` passes, `ascribe build` writes the link as Redoc's
//! address of the operation, and once the spec loses `getLoom`, `check`
//! fails with `link-operation-missing`.

#![allow(clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// The example, as its repository has it.
fn example() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/astro-site")
}

fn copy(from: &Path, to: &Path) {
    fs::create_dir_all(to).expect("create a directory");
    for entry in fs::read_dir(from).expect("read a directory") {
        let entry = entry.expect("an entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("a type").is_dir() {
            copy(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).expect("copy a file");
        }
    }
}

fn ascribe(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ascribe"))
        .current_dir(dir)
        .args(args)
        .env("NO_COLOR", "1")
        .output()
        .expect("run ascribe")
}

fn text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
fn openapi_a_guide_s_link_to_an_operation_is_checked_and_written_as_redoc_s_address() {
    let dir = tempfile::tempdir().expect("a temporary directory");
    let root = dir.path();
    fs::copy(example().join("ascribe.toml"), root.join("ascribe.toml")).expect("the model");
    copy(&example().join("api"), &root.join("api"));
    copy(&example().join("content"), &root.join("content"));

    let out = ascribe(root, &["check"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out));

    let out = ascribe(root, &["build", "--emit", "site"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out));
    let guide =
        fs::read_to_string(root.join(".ascribe/build/site/site/Guides/Teams/Share a loom.md"))
            .expect("the guide, built");
    assert!(
        guide.contains("[Get a loom](/docs/api#operation/getLoom)"),
        "{guide}"
    );
    assert!(
        guide.contains("[List invoices](/docs/api#operation/listInvoices)"),
        "{guide}"
    );

    let spec = root.join("api/openapi.yaml");
    let text_of_spec = fs::read_to_string(&spec).expect("the spec");
    fs::write(
        &spec,
        text_of_spec.replace("operationId: getLoom", "operationId: fetchLoom"),
    )
    .expect("the spec, without getLoom");
    let out = ascribe(root, &["check"]);
    let said = text(&out);
    assert_eq!(out.status.code(), Some(1), "{said}");
    assert!(said.contains("link-operation-missing"), "{said}");
    assert!(
        said.contains("`api/openapi.yaml` has no operation `getLoom`"),
        "{said}"
    );
}
