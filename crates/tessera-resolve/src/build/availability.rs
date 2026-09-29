//! Availability (SPEC §4.4, §9.2 step 2): feature keys, inherited scopes, and
//! whether content is available for a target at a version (§9.3).
//!
//! [`annotate`] turns an expanded page's blocks into resolved blocks and gives
//! each its **effective availability**: the spec written for its block, else
//! for the section it's in, else the enclosing block's, else the page's. It
//! records the scope problems it meets: a spec that exceeds its enclosing
//! scope. It removes nothing; [`super::modes`] does, in a later pass.

use std::collections::HashMap;
use std::sync::Arc;

use tessera_core::availability::{AvailabilitySpec, Detail, Entry, Version, parse_availability};
use tessera_core::{Issue, Location, Span, diagnostics};
use tessera_model::ContentModel;
use tessera_syntax::{Block, BlockKind, Bound};

use super::tree::ResolvedKind;
use super::tree::{Annotation, Availability, ResolvedArm, ResolvedBlock, ResolvedItem, Scope};
use crate::expand::{ExpandedBlock, ExpandedKind, IncludeSite, PageProblem};
use crate::index::FileIndex;
use crate::project::Project;

/// A spec after feature keys: what it is, and how to show it.
#[derive(Clone, Debug)]
pub(crate) struct ResolvedSpec {
    pub spec: AvailabilitySpec,
    pub text: String,
    pub feature: Option<String>,
}

/// Replaces a bare feature key by the spec it stands for (SPEC §4.4, Q25);
/// any other spec stands for itself.
pub(crate) fn resolve_spec(
    model: &ContentModel,
    text: &str,
    spec: &AvailabilitySpec,
) -> ResolvedSpec {
    if let Some(name) = spec.bare_name()
        && let Some(feature) = model.feature(&name.text)
    {
        return ResolvedSpec {
            spec: feature.available.clone(),
            text: feature.available_text.clone(),
            feature: Some(feature.key.clone()),
        };
    }
    ResolvedSpec {
        spec: spec.clone(),
        text: text.trim().to_owned(),
        feature: None,
    }
}

/// The page-level availability of a file: its frontmatter `available`, when
/// it holds a spec that parses (a value of the wrong shape or a spec that
/// doesn't parse is a file-level error, and restricts nothing).
pub(crate) fn page_availability(
    model: &ContentModel,
    index: &FileIndex,
) -> Option<Arc<Availability>> {
    let text = index.frontmatter.as_ref()?.get("available")?.as_str()?;
    let spec = parse_availability(text, 0).ok()?;
    let resolved = resolve_spec(model, text, &spec);
    let at = index
        .document
        .frontmatter
        .as_ref()
        .map_or(Span::empty(0), |f| f.span);
    Some(Arc::new(Availability {
        spec: resolved.spec,
        text: resolved.text,
        feature: resolved.feature,
        scope: Scope::Page,
        written_at: Location::new(index.file, at),
        enclosing: None,
    }))
}

impl Availability {
    /// Whether content with this availability is available for `target` at
    /// `version` (SPEC §9.3): this spec and every spec it sits in must say
    /// so.
    ///
    /// `version` is `None` for a versionless target.
    pub fn is_available(
        &self,
        model: &ContentModel,
        target: &str,
        version: Option<&Version>,
    ) -> bool {
        spec_allows(model, &self.spec, target, version)
            && self
                .enclosing
                .as_ref()
                .is_none_or(|e| e.is_available(model, target, version))
    }
}

/// Whether one spec makes content available for `target` at `version`.
///
/// It's available when the spec lists the target, directly or through the
/// target's dimension name, and the state in effect at the version counts as
/// available. A direct entry wins over one through the dimension name
/// (`cloud removed` beside `deployment`).
// SPEC-QUESTION(Q82): which entry applies when both name the target.
pub(crate) fn spec_allows(
    model: &ContentModel,
    spec: &AvailabilitySpec,
    target: &str,
    version: Option<&Version>,
) -> bool {
    let dimension = model.dimension_of_value(target).map(|d| d.name.as_str());
    let entry = spec
        .entries
        .iter()
        .find(|e| e.target.text == target)
        .or_else(|| {
            spec.entries
                .iter()
                .find(|e| Some(e.target.text.as_str()) == dimension || e.target.text == target)
        });
    let Some(entry) = entry else {
        return false;
    };
    match state_in_effect(entry, version) {
        Some(state) => model.state_is_available(state).unwrap_or(true),
        // Before the first state's start version: nothing is in effect.
        None => false,
    }
}

/// The state in effect for an entry at a version (SPEC §9.3): the last state
/// in its history that starts at or before the version. `None` when the
/// version precedes the first state.
///
/// A bare target with no version is generally available at every version, and
/// a state with no version is in effect at every version (a versionless
/// target's single state). When the build has no version (a versionless
/// target) an entry that has one is treated as in effect.
// SPEC-QUESTION(Q86): a state with no version on a versioned target.
fn state_in_effect<'a>(entry: &'a Entry, version: Option<&Version>) -> Option<&'a str> {
    let started = |start: &Version| version.is_none_or(|v| v.compare(start).is_ge());
    match &entry.detail {
        Detail::None => Some("ga"),
        Detail::Version(v) => started(v).then_some("ga"),
        Detail::State {
            state,
            version: start,
        } => start
            .as_ref()
            .is_none_or(started)
            .then_some(state.text.as_str()),
        Detail::History(steps) => steps
            .iter()
            .rev()
            .find(|s| started(&s.version))
            .map(|s| s.state.text.as_str()),
    }
}

// -- Scope problems ---------------------------------------------------------

/// How a spec exceeds its enclosing scope (SPEC §4.4).
#[derive(Debug, PartialEq, Eq)]
enum Exceeds {
    /// It lists a target the enclosing scope doesn't.
    Target(String),
    /// It starts a target earlier than the enclosing scope does.
    Version {
        target: String,
        version: String,
        enclosing: String,
    },
}

/// The version an entry starts at: `None` for one that starts at no version
/// (generally available everywhere, or a versionless target's state).
fn start(entry: &Entry) -> Option<&Version> {
    match &entry.detail {
        Detail::None => None,
        Detail::Version(v) => Some(v),
        Detail::State { version, .. } => version.as_ref(),
        Detail::History(steps) => steps.first().map(|s| &s.version),
    }
}

/// The first way `child` exceeds `parent`, if it does: it can't list a target
/// the enclosing scope doesn't, or name a version earlier than it does. A
/// dimension name stands for all of its values, so it fits only where each of
/// them does.
fn exceeds(
    model: &ContentModel,
    child: &AvailabilitySpec,
    parent: &AvailabilitySpec,
) -> Option<Exceeds> {
    for entry in &child.entries {
        let name = entry.target.text.as_str();
        let (values, is_dimension): (Vec<&str>, bool) = match model.dimension(name) {
            Some(d) => (d.values.iter().map(|v| v.value.as_str()).collect(), true),
            None => (vec![name], false),
        };
        for value in values {
            let dimension = model.dimension_of_value(value).map(|d| d.name.as_str());
            let covering = parent
                .entries
                .iter()
                .find(|p| p.target.text == value)
                .or_else(|| {
                    parent
                        .entries
                        .iter()
                        .find(|p| Some(p.target.text.as_str()) == dimension)
                });
            let Some(covering) = covering else {
                return Some(Exceeds::Target(name.to_owned()));
            };
            let child_start = if is_dimension { None } else { start(entry) };
            if let Some(parent_start) = start(covering)
                && child_start.is_none_or(|c| c.compare(parent_start).is_lt())
            {
                return Some(Exceeds::Version {
                    target: name.to_owned(),
                    version: child_start.map_or("every version".to_owned(), |v| v.text.clone()),
                    enclosing: parent_start.text.clone(),
                });
            }
        }
    }
    None
}

// -- The pass ---------------------------------------------------------------

/// An `@available` directive, read.
struct Declared {
    spec: ResolvedSpec,
    at: Location,
}

/// A stack frame of the sections open in a list of blocks.
struct Frame {
    /// The heading's level; 0 for a section that has no heading.
    level: u8,
    /// The effective availability in the section.
    scope: Option<Arc<Availability>>,
    /// Where the section ends, for one that has no heading.
    end: Option<usize>,
}

/// Resolves availability over a page's expanded blocks (SPEC §9.2 step 2).
///
/// Every block of the result has its effective availability. `page` is the
/// page-level spec, which every node inherits. Each scope problem goes into
/// `problems` with the spec that exceeds its scope, so the caller can keep
/// only the problems of specs that content the build publishes is under.
pub(crate) fn annotate(
    project: &Project,
    blocks: &[ExpandedBlock],
    page: Option<Arc<Availability>>,
    problems: &mut Vec<(Arc<Availability>, PageProblem)>,
) -> Vec<ResolvedBlock> {
    Annotator { project, problems }.list(blocks, &page)
}

struct Annotator<'a> {
    project: &'a Project,
    problems: &'a mut Vec<(Arc<Availability>, PageProblem)>,
}

/// The `@available` directive line a block is, if it is one.
fn available_line(block: &ExpandedBlock) -> Option<&tessera_syntax::DirectiveLine> {
    match &block.kind {
        ExpandedKind::Leaf(Block {
            kind: BlockKind::Directive(line),
            ..
        }) if line.name == "available" => Some(line),
        _ => None,
    }
}

fn is_heading(block: &ExpandedBlock) -> bool {
    matches!(&block.kind, ExpandedKind::Leaf(b) if matches!(b.kind, BlockKind::Heading(_)))
}

fn heading_level(block: &ExpandedBlock) -> Option<u8> {
    match &block.kind {
        ExpandedKind::Leaf(Block {
            kind: BlockKind::Heading(h),
            ..
        }) => Some(h.level),
        _ => None,
    }
}

/// Whether the block is a line-form directive that binds the block after it.
fn binds_following(block: &ExpandedBlock) -> bool {
    matches!(
        &block.kind,
        ExpandedKind::Leaf(Block { kind: BlockKind::Directive(l), .. })
            if l.binding == Some(Bound::FollowingBlock)
    )
}

impl Annotator<'_> {
    fn model(&self) -> &ContentModel {
        self.project.model()
    }

    /// Reads the `@available` at `block`, when its spec parses.
    fn declared(&self, block: &ExpandedBlock) -> Option<Declared> {
        let line = available_line(block)?;
        let index = self.project.file_by_id(block.file)?;
        let marker = index.availability.iter().find(|m| m.span == line.span)?;
        let (text, _) = marker.primary.as_ref()?;
        let spec = marker.spec.as_ref()?.as_ref().ok()?;
        Some(Declared {
            spec: resolve_spec(self.model(), text, spec),
            at: Location::new(block.file, line.span),
        })
    }

    /// Chains `declared` specs onto `enclosing`, recording each that exceeds
    /// the scope it's in.
    // SPEC-QUESTION(Q83): several `@available` for one heading or block.
    fn chain(
        &mut self,
        declared: &[(Declared, Arc<[IncludeSite]>)],
        scope: Scope,
        enclosing: Option<Arc<Availability>>,
    ) -> Option<Arc<Availability>> {
        let mut current = enclosing;
        for (d, via) in declared {
            let problem = current
                .as_ref()
                .and_then(|parent| {
                    exceeds(self.project.model(), &d.spec.spec, &parent.spec)
                        .map(|why| exceeds_issue(d.at, why, parent.scope))
                })
                .map(|issue| PageProblem {
                    issue,
                    via: via.clone(),
                });
            let made = Arc::new(Availability {
                spec: d.spec.spec.clone(),
                text: d.spec.text.clone(),
                feature: d.spec.feature.clone(),
                scope,
                written_at: d.at,
                enclosing: current.take(),
            });
            if let Some(problem) = problem {
                self.problems.push((made.clone(), problem));
            }
            current = Some(made);
        }
        current
    }

    /// One list of sibling blocks: sections are found within it (SPEC §3.8).
    fn list(
        &mut self,
        blocks: &[ExpandedBlock],
        inherited: &Option<Arc<Availability>>,
    ) -> Vec<ResolvedBlock> {
        // Which `@available` directives apply to which heading or block.
        let mut for_heading: HashMap<usize, Vec<usize>> = HashMap::new();
        let mut for_block: HashMap<usize, Vec<usize>> = HashMap::new();
        let mut block_of: HashMap<usize, usize> = HashMap::new();
        let mut orphans: Vec<usize> = Vec::new();
        let mut read: HashMap<usize, Declared> = HashMap::new();
        for (i, block) in blocks.iter().enumerate() {
            let Some(line) = available_line(block) else {
                continue;
            };
            if let Some(d) = self.declared(block) {
                read.insert(i, d);
            }
            match line.binding {
                Some(Bound::Heading) => {
                    let governing = (0..i).rev().find(|&k| {
                        is_heading(&blocks[k])
                            && blocks[k].file == block.file
                            && blocks[k].via == block.via
                    });
                    match governing {
                        Some(k) => for_heading.entry(k).or_default().push(i),
                        // The heading was left out (`{heading=false}`): the
                        // directive describes what's left of its section.
                        // SPEC-QUESTION(Q84): bindings are per source file.
                        None => orphans.push(i),
                    }
                }
                Some(Bound::FollowingBlock) => {
                    let target = (i + 1..blocks.len()).find(|&j| !binds_following(&blocks[j]));
                    if let Some(j) = target
                        && !is_heading(&blocks[j])
                        && blocks[j].file == block.file
                    {
                        for_block.entry(j).or_default().push(i);
                    }
                }
                _ => {}
            }
        }
        for (&j, directives) in &for_block {
            for &i in directives {
                block_of.insert(i, j);
            }
        }
        // Following-block directives stacked above a block, that aren't
        // `@available`, belong with the block too.
        for (i, block) in blocks.iter().enumerate() {
            if binds_following(block)
                && !block_of.contains_key(&i)
                && let Some(j) = (i + 1..blocks.len()).find(|&j| !binds_following(&blocks[j]))
                && !is_heading(&blocks[j])
                && blocks[j].file == block.file
            {
                block_of.insert(i, j);
            }
        }

        let mut out = Vec::with_capacity(blocks.len());
        let mut stack: Vec<Frame> = Vec::new();
        let mut block_scopes: HashMap<usize, Option<Arc<Availability>>> = HashMap::new();
        for (k, block) in blocks.iter().enumerate() {
            // A section with no heading ends where its content does.
            stack.retain(|f| f.end.is_none_or(|e| e > k));
            let section = |stack: &[Frame]| {
                stack
                    .last()
                    .map_or_else(|| inherited.clone(), |f| f.scope.clone())
            };

            // A heading-bound directive with no heading opens a section of
            // its own, to the end of the content it came in with.
            if orphans.contains(&k)
                && let Some(d) = read.remove(&k)
            {
                let end = (k + 1..blocks.len())
                    .find(|&j| !blocks[j].via.starts_with(&block.via))
                    .unwrap_or(blocks.len());
                let scope = self.chain(&[(d, block.via.clone())], Scope::Section, section(&stack));
                stack.push(Frame {
                    level: 0,
                    scope,
                    end: Some(end),
                });
            }

            let scope = if let Some(level) = heading_level(block) {
                while stack
                    .last()
                    .is_some_and(|f| f.level >= level && f.level > 0)
                {
                    stack.pop();
                }
                let declared: Vec<_> = for_heading
                    .get(&k)
                    .into_iter()
                    .flatten()
                    .filter_map(|i| read.remove(i).map(|d| (d, blocks[*i].via.clone())))
                    .collect();
                let scope = self.chain(&declared, Scope::Section, section(&stack));
                stack.push(Frame {
                    level,
                    scope: scope.clone(),
                    end: None,
                });
                scope
            } else {
                // The block a following-block directive binds, or the
                // directive: both take the block's scope.
                let j = block_of.get(&k).copied().unwrap_or(k);
                match block_scopes.get(&j) {
                    Some(scope) => scope.clone(),
                    None => {
                        let declared: Vec<_> = for_block
                            .get(&j)
                            .into_iter()
                            .flatten()
                            .filter_map(|i| read.remove(i).map(|d| (d, blocks[*i].via.clone())))
                            .collect();
                        let scope = self.chain(&declared, Scope::Block, section(&stack));
                        block_scopes.insert(j, scope.clone());
                        scope
                    }
                }
            };
            out.push(self.block(block, scope));
        }
        out
    }

    fn block(&mut self, block: &ExpandedBlock, scope: Option<Arc<Availability>>) -> ResolvedBlock {
        let kind = match &block.kind {
            ExpandedKind::Leaf(b) => ResolvedKind::Leaf(b.clone()),
            ExpandedKind::BlockQuote { children } => ResolvedKind::BlockQuote {
                children: self.list(children, &scope),
            },
            ExpandedKind::List {
                ordered,
                start,
                tight,
                items,
            } => ResolvedKind::List {
                ordered: *ordered,
                start: *start,
                tight: *tight,
                items: items
                    .iter()
                    .map(|item| ResolvedItem {
                        span: item.span,
                        marker: item.marker,
                        children: self.list(&item.children, &scope),
                    })
                    .collect(),
            },
            ExpandedKind::Container {
                opener,
                end,
                children,
            } => ResolvedKind::Container {
                opener: opener.clone(),
                end: end.clone(),
                children: self.list(children, &scope),
            },
            ExpandedKind::Group { name, arms, end } => ResolvedKind::Group {
                name: name.clone(),
                end: end.clone(),
                arms: arms
                    .iter()
                    .map(|arm| ResolvedArm {
                        span: arm.span,
                        opener: arm.opener.clone(),
                        children: self.list(&arm.children, &scope),
                    })
                    .collect(),
            },
        };
        ResolvedBlock {
            file: block.file,
            span: block.span,
            via: block.via.clone(),
            availability: scope,
            heading: None,
            annotation: self.annotation(block),
            substitutions: Vec::new(),
            links: Vec::new(),
            glossary: Vec::new(),
            kind,
        }
    }

    /// What a surviving `@available` declares: its spec as shown, with a
    /// feature key replaced by its spec (Q25).
    fn annotation(&self, block: &ExpandedBlock) -> Option<Annotation> {
        let line = available_line(block)?;
        let index = self.project.file_by_id(block.file)?;
        let marker = index.availability.iter().find(|m| m.span == line.span)?;
        let (text, _) = marker.primary.as_ref()?;
        let spec = marker.spec.as_ref()?.as_ref().ok()?;
        let resolved = resolve_spec(self.model(), text, spec);
        Some(Annotation {
            text: resolved.text,
            feature: resolved.feature,
            binding: line.binding,
        })
    }
}

fn exceeds_issue(at: Location, why: Exceeds, scope: Scope) -> Issue {
    let issue =
        Issue::new(diagnostics::AVAILABLE_EXCEEDS_SCOPE, at).with_arg("scope", scope.as_str());
    match why {
        Exceeds::Target(target) => issue.with_arg("target", target),
        Exceeds::Version {
            target,
            version,
            enclosing,
        } => issue
            .with_variant("version")
            .with_arg("target", target)
            .with_arg("version", version)
            .with_arg("enclosing", enclosing),
    }
}
