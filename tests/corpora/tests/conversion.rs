//! The converters: hand-written fixtures (offline), and each corpus converted
//! and checked (fetched; skipped without the network).

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    clippy::print_stderr
)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use ascribe_check::{Project, Severity, check_all_builds};
use ascribe_corpora::convert::{self, Converted, Extra};
use ascribe_corpora::corpus::{self, Corpus};
use serde::{Deserialize, Serialize};

fn convert_one(corpus: Corpus, path: &str, text: &str, docset: &str) -> Converted {
    convert::convert(
        corpus,
        &[(path.to_owned(), text.to_owned())],
        &Extra {
            docset: docset.to_owned(),
        },
    )
}

fn check(converted: &Converted) -> Vec<ascribe_check::Diagnostic> {
    let dir = tempfile::tempdir().expect("temp dir");
    converted.write_to(dir.path()).expect("writes");
    let project = Project::load(&dir.path().join("ascribe.toml")).expect("the model loads");
    check_all_builds(&project)
}

#[test]
fn elastic_constructs_become_ascribe() {
    let source = "\
---
applies_to:
  stack: ga 9.0
  serverless: ga
navigation_title: Short
---

# Setup [setup_guide]

Use {{es}} and {{kib}}, see [the guide]({{docs-url}}).

::::{note}
Back up first.
::::

:::{dropdown} More detail
Hidden {{es}} text.
:::

::::{tab-set}
:::{tab-item} Linux
:sync: linux
Run it.
:::
:::{tab-item} macOS
Run it here.
:::
::::

```sh subs=true
curl {{es}}
```
";
    let docset =
        "subs:\n  es: \"Elasticsearch\"\n  kib: Kibana\n  docs-url: \"https://example.com/docs\"\n";
    let converted = convert_one(Corpus::Elastic, "guide/setup.md", source, docset);
    let text = &converted.files["guide/setup.md"];
    for expected in [
        "available: \"stack ga 9.0, serverless ga\"",
        "# Setup\n@id: setup-guide",
        "Use {es} and {kib}, see [the guide]({docs-url}).",
        "@note:\nBack up first.\n@end",
        ".More detail\n@details:\nHidden {es} text.\n@end",
        ".Linux\n@variant:\nRun it.",
        ".macOS\n@variant:\nRun it here.\n\n@end",
        "```sh phrases=true\ncurl {es}\n```",
    ] {
        assert!(text.contains(expected), "missing {expected:?} in:\n{text}");
    }
    assert!(converted.model.contains("es = \"Elasticsearch\""));
    assert!(converted.model.contains("[dimensions.applies-to]"));
    let diagnostics = check(&converted);
    let errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .map(|d| d.slug.as_str())
        .collect();
    assert!(errors.is_empty(), "{errors:?}");
}

#[test]
fn a_directive_inside_an_html_comment_is_left_alone() {
    let source = "<!--\n:::{note}\nhidden\n:::\n-->\n\nText.\n";
    let converted = convert_one(Corpus::Elastic, "a.md", source, "");
    assert!(converted.files["a.md"].contains("<!--\n:::{note}\nhidden\n:::\n-->"));
}

#[test]
fn astro_constructs_become_ascribe() {
    let source = "\
---
title: Deploy
---
import { Steps } from '@astrojs/starlight/components';

:::tip[Before you start]
Have a Git repository.
:::

<Steps>
1. Push it.
2. Connect it.
</Steps>

<Tabs>
  <TabItem label=\"npm\">
    Run npm.
  </TabItem>
  <TabItem label=\"pnpm\">
    Run pnpm.
  </TabItem>
</Tabs>
";
    let converted = convert_one(Corpus::Astro, "guides/deploy.mdx", source, "");
    let text = &converted.files["guides/deploy.md"];
    for expected in [
        "title: \"Deploy\"",
        ".Before you start\n@note {type=tip}:\nHave a Git repository.\n@end",
        "@steps\n1. Push it.",
        ".npm\n@variant:\nRun npm.",
        ".pnpm\n@variant:\nRun pnpm.",
    ] {
        assert!(text.contains(expected), "missing {expected:?} in:\n{text}");
    }
    assert!(!text.contains("import "));
    let errors: Vec<_> = check(&converted)
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .map(|d| d.slug.as_str().to_owned())
        .collect();
    assert!(errors.is_empty(), "{errors:?}");
}

#[test]
fn docker_constructs_become_ascribe() {
    let source = "\
---
title: Get started
---

## Install {#install}

> [!NOTE]
> Needs Docker Desktop.

{{< tabs >}}
{{< tab name=\"Go\" >}}
Go text.
{{< /tab >}}
{{< tab name=\"Node\" >}}
Node text.
{{< /tab >}}
{{< /tabs >}}
";
    let converted = convert_one(Corpus::Docker, "get-started.md", source, "");
    let text = &converted.files["get-started.md"];
    for expected in [
        "## Install\n@id: install",
        "@note:\nNeeds Docker Desktop.\n@end",
        ".Go\n@variant:\nGo text.",
        ".Node\n@variant:\nNode text.",
    ] {
        assert!(text.contains(expected), "missing {expected:?} in:\n{text}");
    }
    let errors: Vec<_> = check(&converted)
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .map(|d| d.slug.as_str().to_owned())
        .collect();
    assert!(errors.is_empty(), "{errors:?}");
}

/// What a converted corpus is allowed to produce: none of these, whatever the
/// prose says, because they'd mean the converter or the model is wrong.
const MUST_NOT_APPEAR: &[&str] = &[
    "source-unreadable",
    "frontmatter-unknown-key",
    "frontmatter-missing-field",
    "frontmatter-type-mismatch",
    "frontmatter-reserved-in-fragment",
    "content-type-unresolved",
    "directive-unknown",
    "attribute-syntax",
    "attribute-unknown-key",
    "available-unknown",
    "available-syntax",
    "available-history-order",
    "available-versionless",
    "variant-unknown",
    "widget-schema",
    "id-invalid",
];

#[derive(Serialize, Deserialize, Debug)]
struct Counts {
    pages: usize,
    errors: usize,
    warnings: usize,
}

fn converted_corpus(corpus: Corpus) -> Option<Converted> {
    let fetched = corpus::corpus_or_skip(corpus)?;
    let pages = corpus::pages(&fetched).expect("reads the corpus");
    let extra = Extra {
        docset: std::fs::read_to_string(fetched.root.join("docset.yml")).unwrap_or_default(),
    };
    Some(convert::convert(corpus, &pages, &extra))
}

fn run_corpus(corpus: Corpus, at_least: usize) {
    let Some(converted) = converted_corpus(corpus) else {
        return;
    };
    assert!(converted.pages() >= at_least, "{} pages", converted.pages());
    let diagnostics = check(&converted);
    let mut by_slug: BTreeMap<&str, usize> = BTreeMap::new();
    for d in &diagnostics {
        *by_slug.entry(d.slug.as_str()).or_insert(0) += 1;
    }
    for slug in MUST_NOT_APPEAR {
        assert!(
            !by_slug.contains_key(slug),
            "{corpus}: {slug} x{} (a converter or model bug): {:?}",
            by_slug[slug],
            diagnostics
                .iter()
                .filter(|d| d.slug.as_str() == *slug)
                .take(3)
                .map(|d| d.message.clone())
                .collect::<Vec<_>>()
        );
    }
    let current = Counts {
        pages: converted.pages(),
        errors: diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .count(),
        warnings: diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Warning)
            .count(),
    };
    eprintln!("  {corpus}: {current:?}; by slug {by_slug:?}");
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("baselines")
        .join(format!("converted-{corpus}.json"));
    if std::env::var_os("ASCRIBE_CORPORA_BLESS").is_some() {
        let mut text = serde_json::to_string_pretty(&current).expect("serializes");
        text.push('\n');
        std::fs::write(&path, text).expect("writes");
        return;
    }
    let recorded: Counts =
        serde_json::from_str(&std::fs::read_to_string(&path).expect("a baseline")).expect("parses");
    assert_eq!(current.pages, recorded.pages, "{corpus}: pages");
    // Errors and warnings may drift with the checks; a jump of a tenth is a
    // change worth a look (a converter regression, or a check gone wrong).
    for (what, now, then) in [
        ("errors", current.errors, recorded.errors),
        ("warnings", current.warnings, recorded.warnings),
    ] {
        assert!(
            now <= then + then / 10 + 5 && now + now / 10 + 5 >= then,
            "{corpus}: {what} went from {then} to {now}; review, then ASCRIBE_CORPORA_BLESS=1"
        );
    }
}

#[test]
#[ignore = "slow in debug builds; run with --release -- --ignored"]
fn elastic_converts_and_checks() {
    run_corpus(Corpus::Elastic, 3000);
}

#[test]
#[ignore = "slow in debug builds; run with --release -- --ignored"]
fn astro_converts_and_checks() {
    run_corpus(Corpus::Astro, 400);
}

#[test]
#[ignore = "slow in debug builds; run with --release -- --ignored"]
fn docker_converts_and_checks() {
    run_corpus(Corpus::Docker, 1000);
}
