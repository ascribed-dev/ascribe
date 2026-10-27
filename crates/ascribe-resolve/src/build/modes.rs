//! Build modes (SPEC §9.3, §9.2 step 3): a build's variant mode and its
//! availability mode, applied to a page and to the blocks of a page.
//!
//! - **Selection** removes the arms and pages that conflict with the build. A
//!   group keeps every surviving arm: one arm becomes plain content, several
//!   stay a group, none removes the group (and is recorded). Groups on
//!   unselected dimensions and labeled groups are untouched.
//! - **Filter** removes content that isn't available for the build's target
//!   and version, and a page whose frontmatter `available` makes it
//!   unavailable. What remains keeps its availability annotations.
//! - **A line** of a versioned build removes, the same way, content that
//!   names the build's target and isn't in the line.
//!
//! Each pass also says what it removed and why ([`Removed`]), so the editor
//! shows what a build leaves out from the same decisions the build makes.

use ascribe_core::availability::{
    AvailabilitySpec, Version, parse_availability, spec_text, without_versions_through,
};
use ascribe_core::{Issue, Location, diagnostics};
use ascribe_model::{AvailabilityMode, Build, BuildLine, ContentModel, VariantMode};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use ascribe_syntax::{Block, BlockKind, DirectiveLine};

use super::availability::{page_availability, spec_allows, spec_in_line};
use super::tree::{
    Availability, DropReason, Removal, Removed, ResolvedArm, ResolvedBlock, ResolvedKind,
};
use crate::expand::PageProblem;
use crate::index::FileIndex;

/// Why the build doesn't publish this page, if it doesn't (SPEC §9.3): it's
/// a draft, its `variant` frontmatter conflicts with the selection, or its
/// `available` frontmatter makes it unavailable in a filter build.
pub(crate) fn drop_reason(
    model: &ContentModel,
    index: &FileIndex,
    build: &Build,
) -> Option<DropReason> {
    if !build.drafts && index.is_draft() {
        return Some(DropReason::Draft);
    }
    if let VariantMode::Select(selection) = &build.variants
        && page_conflicts(index, selection)
    {
        return Some(DropReason::Variant);
    }
    if let Some(keep) = Keep::of(build)
        && let Some(page) = page_availability(model, index)
        && !keep.availability(model, &page)
    {
        return Some(DropReason::Unavailable);
    }
    None
}

/// What a build's availability keeps: content available for a filter's
/// target and version, or content in a line. `None` keeps everything.
enum Keep<'a> {
    Filter {
        target: &'a str,
        version: Option<&'a Version>,
    },
    Line {
        target: &'a str,
        line: &'a BuildLine,
    },
}

impl<'a> Keep<'a> {
    fn of(build: &'a Build) -> Option<Keep<'a>> {
        if let (Some(target), Some(line)) = (&build.versions, &build.line) {
            return Some(Keep::Line { target, line });
        }
        match &build.availability {
            AvailabilityMode::Filter { target, version } => Some(Keep::Filter {
                target,
                version: version.as_ref(),
            }),
            AvailabilityMode::Badge => None,
        }
    }

    /// Whether content with this effective availability stays.
    fn availability(&self, model: &ContentModel, a: &Availability) -> bool {
        match self {
            Keep::Filter { target, version } => a.is_available(model, target, *version),
            Keep::Line { target, line } => a.in_line(model, target, line),
        }
    }

    /// Whether one spec, on its own, lets content stay.
    fn spec(&self, model: &ContentModel, spec: &AvailabilitySpec) -> bool {
        match self {
            Keep::Filter { target, version } => spec_allows(model, spec, target, *version),
            Keep::Line { target, line } => spec_in_line(model, spec, target, line),
        }
    }
}

/// Whether a page's `variant` frontmatter names a selected dimension and none
/// of its selected values (SPEC §9.3). A value that isn't the shape SPEC §4.3
/// gives (a value or a list of values) names nothing.
fn page_conflicts(index: &FileIndex, selection: &[(String, Vec<String>)]) -> bool {
    let Some(variant) = index
        .frontmatter
        .as_ref()
        .and_then(|f| f.get("variant"))
        .and_then(|v| v.as_mapping())
    else {
        return false;
    };
    selection.iter().any(|(dimension, selected)| {
        let Some(value) = variant.get(dimension.as_str()) else {
            return false;
        };
        let values: Vec<&str> = match value {
            serde_yaml_ng::Value::String(s) => vec![s.as_str()],
            serde_yaml_ng::Value::Sequence(items) => items
                .iter()
                .filter_map(serde_yaml_ng::Value::as_str)
                .collect(),
            _ => return false,
        };
        !values.iter().any(|v| selected.iter().any(|s| s == v))
    })
}

/// Applies the build's modes to a page's blocks: content that isn't available
/// is removed, conflicting arms are removed, and a group is reduced.
/// `problems` gets a warning for each group whose every arm is removed.
/// Returns what stays, and what was removed, outermost only: nothing inside
/// removed content is listed.
pub(crate) fn apply(
    blocks: Vec<ResolvedBlock>,
    build: &Build,
    model: &ContentModel,
    problems: &mut Vec<PageProblem>,
) -> (Vec<ResolvedBlock>, Vec<Removed>) {
    let mut modes = Modes {
        build,
        model,
        problems,
        removed: Vec::new(),
    };
    let blocks = modes.list(blocks);
    (blocks, modes.removed)
}

/// `blocks` as a build that selects `selection` and keeps every arm of
/// every other dimension writes them (SPEC §9.3): conflicting arms are
/// removed, a group left with one arm is that arm's blocks, and a group left
/// with none is gone. Availability isn't touched: the blocks are already
/// their build's.
pub fn select(
    blocks: &[ResolvedBlock],
    selection: &[(String, Vec<String>)],
    model: &ContentModel,
) -> Vec<ResolvedBlock> {
    let build = Build {
        name: String::new(),
        variants: VariantMode::Select(selection.to_vec()),
        availability: AvailabilityMode::Badge,
        versions: None,
        line: None,
        drafts: false,
    };
    let mut problems = Vec::new();
    let mut modes = Modes {
        build: &build,
        model,
        problems: &mut problems,
        removed: Vec::new(),
    };
    modes.list(blocks.to_vec())
}

struct Modes<'a> {
    build: &'a Build,
    model: &'a ContentModel,
    problems: &'a mut Vec<PageProblem>,
    removed: Vec<Removed>,
}

impl Modes<'_> {
    /// Whether a node with this effective availability stays.
    fn available(&self, block: &ResolvedBlock) -> bool {
        let Some(keep) = Keep::of(self.build) else {
            return true;
        };
        block
            .availability
            .as_ref()
            .is_none_or(|a| keep.availability(self.model, a))
    }

    /// The spec that rules out content with this effective availability:
    /// the innermost in its chain that doesn't allow the build's target and
    /// version.
    fn ruling_out(&self, spec: &Arc<Availability>) -> Arc<Availability> {
        let Some(keep) = Keep::of(self.build) else {
            return spec.clone();
        };
        let mut at = spec;
        loop {
            if !keep.spec(self.model, &at.spec) {
                return at.clone();
            }
            match &at.enclosing {
                Some(enclosing) => at = enclosing,
                None => return spec.clone(),
            }
        }
    }

    fn list(&mut self, blocks: Vec<ResolvedBlock>) -> Vec<ResolvedBlock> {
        let mut out = Vec::with_capacity(blocks.len());
        for mut block in blocks {
            // Content removed by the filter is gone, and so is anything that
            // would have been reported inside it.
            if !self.available(&block) {
                if let Some(spec) = &block.availability {
                    let spec = self.ruling_out(spec);
                    self.removed.push(Removed {
                        file: block.file,
                        span: block.span,
                        via: block.via.clone(),
                        cause: Removal::Availability(spec),
                    });
                }
                continue;
            }
            if matches!(block.kind, ResolvedKind::Group { .. }) {
                out.extend(self.group(block));
                continue;
            }
            self.rows(&mut block);
            for children in block.child_lists_mut() {
                let taken = std::mem::take(children);
                *children = self.list(taken);
            }
            out.push(block);
        }
        out
    }

    /// Removes a table's rows that aren't available (SPEC §4.4). The header
    /// row has no availability of its own, so a table always keeps it.
    fn rows(&mut self, block: &mut ResolvedBlock) {
        let Some(keep) = Keep::of(self.build) else {
            return;
        };
        let ResolvedKind::Leaf(Block {
            kind: BlockKind::Table(table),
            ..
        }) = &mut block.kind
        else {
            return;
        };
        let (kept, removed): (Vec<_>, Vec<_>) = std::mem::take(&mut block.rows)
            .into_iter()
            .partition(|r| keep.availability(self.model, &r.availability));
        table
            .rows
            .retain(|row| !removed.iter().any(|r| r.span == row.span));
        block.rows = kept;
        for row in removed {
            let spec = self.ruling_out(&row.availability);
            self.removed.push(Removed {
                file: block.file,
                span: row.span,
                via: block.via.clone(),
                cause: Removal::Availability(spec),
            });
        }
    }

    /// A group under the build's selection: conflicting arms are removed; one
    /// remaining arm becomes its content, several stay a group, and none
    /// removes the group and is recorded.
    fn group(&mut self, mut block: ResolvedBlock) -> Vec<ResolvedBlock> {
        let ResolvedKind::Group { name, arms, .. } = &mut block.kind else {
            return vec![block];
        };
        let VariantMode::Select(selection) = &self.build.variants else {
            // `switch` keeps every arm.
            return self.arms_recursed(block);
        };
        // Only `@variant` groups are along dimensions; a project widget's
        // groups are labeled.
        let affected =
            name == "variant" && arms.iter().any(|a| names_selected(&a.opener, selection));
        if !affected {
            return self.arms_recursed(block);
        }
        let first = arms.first().map(|a| a.opener.span);
        let (kept, gone): (Vec<ResolvedArm>, Vec<ResolvedArm>) = std::mem::take(arms)
            .into_iter()
            .partition(|arm| conflicting(&arm.opener, selection).is_empty());
        *arms = kept;
        if arms.is_empty() {
            // The whole group goes, its end line too.
            let mut dimensions: Vec<String> = Vec::new();
            for arm in &gone {
                for dimension in conflicting(&arm.opener, selection) {
                    if !dimensions.contains(&dimension) {
                        dimensions.push(dimension);
                    }
                }
            }
            self.removed.push(Removed {
                file: block.file,
                span: block.span,
                via: block.via.clone(),
                cause: Removal::Variant { dimensions },
            });
        } else {
            for arm in &gone {
                self.removed.push(Removed {
                    file: block.file,
                    span: arm.span,
                    via: block.via.clone(),
                    cause: Removal::Variant {
                        dimensions: conflicting(&arm.opener, selection),
                    },
                });
            }
        }
        match arms.len() {
            0 => {
                if let Some(span) = first {
                    let issue = Issue::new(
                        diagnostics::VARIANT_NO_ARM_SURVIVES,
                        Location::new(block.file, span),
                    )
                    .with_arg("build", self.build.name.clone());
                    self.problems.push(PageProblem {
                        issue,
                        via: block.via.clone(),
                    });
                }
                Vec::new()
            }
            1 => {
                let arm = arms.remove(0);
                self.list(arm.children)
            }
            _ => self.arms_recursed(block),
        }
    }

    /// Applies the modes inside the arms of a group that stays.
    fn arms_recursed(&mut self, mut block: ResolvedBlock) -> Vec<ResolvedBlock> {
        if let ResolvedKind::Group { arms, .. } = &mut block.kind {
            let taken: Vec<ResolvedArm> = std::mem::take(arms);
            *arms = taken
                .into_iter()
                .map(|mut arm| {
                    arm.children = self.list(std::mem::take(&mut arm.children));
                    arm
                })
                .collect();
        }
        vec![block]
    }
}

/// The values an arm's attribute gives: a value, or the members of a set.
fn values(value: &ascribe_core::AttributeValue) -> Vec<&str> {
    match value {
        ascribe_core::AttributeValue::Set { members, .. } => {
            members.iter().map(|m| m.text.as_str()).collect()
        }
        other => other.as_text().into_iter().collect(),
    }
}

/// Whether the arm names a dimension the selection names.
fn names_selected(opener: &DirectiveLine, selection: &[(String, Vec<String>)]) -> bool {
    opener.attributes.as_ref().is_some_and(|block| {
        block
            .attributes
            .iter()
            .any(|a| selection.iter().any(|(d, _)| *d == a.key))
    })
}

/// The dimensions on which the arm conflicts with the selection (SPEC §9.3):
/// those the selection names where the arm names the dimension and none of
/// the selected values. The arm conflicts when there is one.
fn conflicting(opener: &DirectiveLine, selection: &[(String, Vec<String>)]) -> Vec<String> {
    let Some(block) = &opener.attributes else {
        return Vec::new();
    };
    selection
        .iter()
        .filter(|(dimension, selected)| {
            block
                .attributes
                .iter()
                .filter(|a| a.key == *dimension)
                .any(|a| {
                    let Some(value) = &a.value else {
                        return false;
                    };
                    !values(value)
                        .iter()
                        .any(|v| selected.iter().any(|s| s == v))
                })
        })
        .map(|(dimension, _)| dimension.clone())
        .collect()
}

/// The availability specs that content in `blocks` is under: each block's
/// own effective spec, each surviving table row's, and every spec they sit
/// in, by identity.
pub(crate) fn live(blocks: &[ResolvedBlock]) -> HashSet<*const Availability> {
    let mut set = HashSet::new();
    for block in blocks {
        block.visit(&mut |b| {
            let rows = b.rows.iter().map(|r| &r.availability);
            for mut spec in b.availability.iter().chain(rows) {
                while set.insert(Arc::as_ptr(spec)) {
                    let Some(enclosing) = &spec.enclosing else {
                        break;
                    };
                    spec = enclosing;
                }
            }
        });
    }
    set
}

/// Whether an expansion problem is about content that's still in the tree:
/// an `@include` that couldn't be expanded stays as its directive, so the
/// problem stands while that directive is there.
pub(crate) fn survives(blocks: &[ResolvedBlock], problem: &PageProblem) -> bool {
    let at = problem.issue.location;
    let mut found = false;
    for block in blocks {
        block.visit(&mut |b| {
            if b.file == at.file
                && b.via == problem.via
                && b.span.contains_span(at.span)
                && let ResolvedKind::Leaf(leaf) = &b.kind
                && matches!(&leaf.kind, BlockKind::Directive(line) if line.name == "include")
            {
                found = true;
            }
        });
    }
    found
}

/// What a line of a versioned build shows of the availability that stays:
/// no version of its target at or before the oldest live line (SPEC §9.3).
/// Every spec on the page, its frontmatter's included, is shown that way.
/// Nothing changes for a build that isn't versioned.
pub(crate) fn show_for_line(
    build: &Build,
    blocks: &mut [ResolvedBlock],
    page: &mut Option<Arc<Availability>>,
) {
    let (Some(target), Some(line)) = (&build.versions, &build.line) else {
        return;
    };
    let mut shown = Shown {
        target,
        oldest: &line.oldest,
        done: HashMap::new(),
    };
    if let Some(a) = page {
        *a = shown.availability(a);
    }
    shown.blocks(blocks);
}

/// Specs rewritten for a line, each once, so specs that share an enclosing
/// one still do.
struct Shown<'a> {
    target: &'a str,
    oldest: &'a Version,
    done: HashMap<*const Availability, Arc<Availability>>,
}

impl Shown<'_> {
    fn availability(&mut self, a: &Arc<Availability>) -> Arc<Availability> {
        if let Some(hit) = self.done.get(&Arc::as_ptr(a)) {
            return hit.clone();
        }
        let enclosing = a.enclosing.as_ref().map(|e| self.availability(e));
        let spec = without_versions_through(&a.spec, self.target, self.oldest);
        let same_enclosing = match (&enclosing, &a.enclosing) {
            (Some(x), Some(y)) => Arc::ptr_eq(x, y),
            (None, None) => true,
            _ => false,
        };
        let made = match spec {
            None if same_enclosing => a.clone(),
            spec => {
                let text = match &spec {
                    Some(spec) => spec_text(spec),
                    None => a.text.clone(),
                };
                Arc::new(Availability {
                    spec: spec.unwrap_or_else(|| a.spec.clone()),
                    text,
                    enclosing,
                    ..(**a).clone()
                })
            }
        };
        self.done.insert(Arc::as_ptr(a), made.clone());
        made
    }

    fn blocks(&mut self, blocks: &mut [ResolvedBlock]) {
        for block in blocks {
            if let Some(a) = &block.availability {
                block.availability = Some(self.availability(a));
            }
            for row in &mut block.rows {
                row.availability = self.availability(&row.availability);
            }
            if let Some(annotation) = &mut block.annotation
                && let Ok(spec) = parse_availability(&annotation.text, 0)
                && let Some(spec) = without_versions_through(&spec, self.target, self.oldest)
            {
                annotation.text = spec_text(&spec);
            }
            for children in block.child_lists_mut() {
                self.blocks(children);
            }
        }
    }
}
