//! Include expansion over random include graphs (SPEC §4.2), against a model
//! written from the spec: a file is its paragraph and, in place of each
//! include, its target's expansion, except an include of a file already being
//! expanded, which is a cycle. Graphs have cycles, self-includes, and files
//! included several times in a row.
//!
//! The property: expansion finishes, with the model's paragraphs and cycles;
//! and if a graph would take the page past the limit, it has one
//! `include-limit`, at an include in the page's own file, and no more than
//! the limit's blocks. Graphs that small seldom pass the limit within the
//! shared budget, so `includes.rs` holds the limit, and the rules' edges.

#![allow(clippy::expect_used, clippy::panic)]

use crate::support;

use ascribe_resolve::{ExpandedKind, MAX_EXPANDED_BLOCKS};
use ascribe_syntax::BlockKind;
use proptest::prelude::*;

/// Each file's includes, by index, each written some times in a row; file 0
/// is the page.
type Graph = Vec<Vec<(usize, usize)>>;

fn graph() -> impl Strategy<Value = Graph> {
    (1usize..=6).prop_flat_map(|n| {
        prop::collection::vec(prop::collection::vec((0..n, 1usize..=6), 0..=3), n)
    })
}

/// What the model says the page expands to, until it passes the limit.
#[derive(Default)]
struct Model {
    paragraphs: usize,
    cycles: usize,
}

impl Model {
    fn expand(&mut self, graph: &Graph, file: usize, stack: &mut Vec<usize>) {
        self.paragraphs += 1;
        for target in graph[file]
            .iter()
            .flat_map(|&(t, times)| std::iter::repeat_n(t, times))
        {
            if self.paragraphs > MAX_EXPANDED_BLOCKS {
                return;
            }
            if stack.contains(&target) {
                self.cycles += 1;
                continue;
            }
            stack.push(target);
            self.expand(graph, target, stack);
            stack.pop();
        }
    }
}

proptest! {
    #[test]
    fn expansion_matches_the_model_and_stops_at_the_limit(graph in graph()) {
        let files: Vec<(String, String)> = graph
            .iter()
            .enumerate()
            .map(|(i, includes)| {
                let mut text = format!("Text {i}.\n");
                for &(target, times) in includes {
                    for _ in 0..times {
                        text.push_str(&format!("\n@include: f{target}.md\n"));
                    }
                }
                (format!("docs/f{i}.md"), text)
            })
            .collect();
        let files: Vec<(&str, &str)> = files
            .iter()
            .map(|(p, t)| (p.as_str(), t.as_str()))
            .collect();
        let project = support::project(&files);
        let page = project.expand(&support::path("f0.md")).expect("the page");

        let mut model = Model::default();
        model.expand(&graph, 0, &mut vec![0]);

        let mut paragraphs = 0;
        page.visit(&mut |block| {
            if let ExpandedKind::Leaf(leaf) = &block.kind
                && matches!(leaf.kind, BlockKind::Paragraph(_))
            {
                paragraphs += 1;
            }
        });
        let slugs: Vec<&str> = page.problems.iter().map(|p| p.issue.slug.as_str()).collect();
        if model.paragraphs <= MAX_EXPANDED_BLOCKS {
            prop_assert_eq!(paragraphs, model.paragraphs);
            prop_assert_eq!(slugs, vec!["include-cycle"; model.cycles]);
        } else {
            let limits: Vec<_> = page
                .problems
                .iter()
                .filter(|p| p.issue.slug.as_str() == "include-limit")
                .collect();
            prop_assert_eq!(limits.len(), 1, "{:?}", slugs);
            prop_assert!(limits[0].via.is_empty());
            prop_assert_eq!(project.path_of(limits[0].issue.location.file), Some(&support::path("f0.md")));
            prop_assert!(paragraphs <= MAX_EXPANDED_BLOCKS);
        }
    }
}
