//! The lines of a versioned build, as one line's resolver sees them (SPEC
//! §5.2, §9.3): the page that takes another's place in a line, the line
//! nearest to this one that publishes a page, an address in another line,
//! and the checks on `replaces` (SPEC §2.1).

use std::collections::{BTreeMap, HashSet, VecDeque};
use std::rc::Rc;
use std::sync::Arc;

use ascribe_core::{Issue, Location, RelPath, diagnostics};
use ascribe_model::Build;

use super::BuildResolver;
use crate::expand::PageProblem;
use crate::index::FileIndex;
use crate::replaces::replaces_span;

/// Where a link to a page this line doesn't publish leads (SPEC §5.2).
pub(crate) enum Landing<'p> {
    /// The page that takes its place in this line.
    Replaced(RelPath),
    /// The page itself, in the nearest line that publishes it.
    Line(&'p Build),
}

impl<'p> BuildResolver<'p> {
    /// Every page's `replaces` that names a page, by the page that says it.
    pub(crate) fn replacements(&self) -> &BTreeMap<RelPath, RelPath> {
        self.replacements
            .get_or_init(|| self.project.replacements())
    }

    /// The lines of this resolver's versioned build, newest first, this one
    /// among them. Empty for a build that isn't versioned.
    pub(crate) fn lines(&self) -> Vec<&'p Build> {
        let Some(own) = &self.build.line else {
            return Vec::new();
        };
        let mut lines: Vec<&'p Build> = self
            .project
            .model()
            .builds
            .iter()
            .filter(|b| b.line.as_ref().is_some_and(|l| l.build == own.build))
            .collect();
        lines.sort_by(|a, b| match (&a.line, &b.line) {
            (Some(a), Some(b)) => b.line.compare(&a.line),
            _ => std::cmp::Ordering::Equal,
        });
        lines
    }

    /// Where a link to `target`, a page this line doesn't publish, leads:
    /// the page that replaces it or that it replaces in this line, else the
    /// nearest line that publishes it, the closest newer line first, then
    /// the closest older. `None` in a build that isn't versioned, and when no
    /// line publishes it.
    pub(crate) fn landing(&self, target: &RelPath) -> Option<Landing<'p>> {
        self.build.line.as_ref()?;
        if let Some(page) = self.replacement(target) {
            return Some(Landing::Replaced(page));
        }
        let lines = self.lines();
        let here = lines.iter().position(|b| b.name == self.build.name)?;
        let newer = lines[..here].iter().rev();
        let older = lines[here + 1..].iter();
        newer
            .chain(older)
            .find(|b| self.project.dropped(target, b).is_none())
            .map(|b| Landing::Line(b))
    }

    /// Where a link to the page `target` goes in this build: the page it
    /// lands on, and, for a link out of a line, the line it leads to (SPEC
    /// §5.2). `None` when it goes nowhere: the build doesn't publish the
    /// page and, in a versioned build, nothing takes its place and no line
    /// publishes it.
    pub fn link_landing(&self, target: &RelPath) -> Option<(RelPath, Option<&'p Build>)> {
        if self.is_published(target) {
            return Some((target.clone(), None));
        }
        match self.landing(target)? {
            Landing::Replaced(page) => Some((page, None)),
            Landing::Line(line) => Some((target.clone(), Some(line))),
        }
    }

    /// In an older line, the page the current line publishes in `path`'s
    /// place: `path` itself, or else the page `replaces` links it with,
    /// nearest along the chain first (SPEC §9.3). `None` in the current line,
    /// in a build that isn't versioned, and when the current line has
    /// neither.
    pub(crate) fn current_page(&self, path: &RelPath) -> Option<RelPath> {
        if self.build.line.as_ref()?.current {
            return None;
        }
        let current = self
            .lines()
            .into_iter()
            .find(|b| b.line.as_ref().is_some_and(|l| l.current))?;
        let current = self.line_resolver(current);
        if current.is_published(path) {
            return Some(path.clone());
        }
        current.replacement(path)
    }

    /// The page this line publishes in `target`'s place: one of the pages
    /// `replaces` links it with, either way, nearest along the chain first,
    /// and a page that replaces before the page it replaces.
    fn replacement(&self, target: &RelPath) -> Option<RelPath> {
        let map = self.replacements();
        let mut seen: HashSet<&RelPath> = HashSet::from([target]);
        let mut queue: VecDeque<&RelPath> = VecDeque::from([target]);
        while let Some(at) = queue.pop_front() {
            let newer = map.iter().filter(|(_, r)| *r == at).map(|(by, _)| by);
            let older = map.get(at);
            for next in newer.chain(older) {
                if !seen.insert(next) {
                    continue;
                }
                if self.is_published(next) {
                    return Some(next.clone());
                }
                queue.push_back(next);
            }
        }
        None
    }

    /// A resolver for another line of this build, kept, so its pages are
    /// resolved once however many links lead there.
    pub(crate) fn line_resolver(&self, line: &'p Build) -> Rc<BuildResolver<'p>> {
        if let Some(hit) = self.others.borrow().get(&line.name) {
            return hit.clone();
        }
        let made = Rc::new(BuildResolver::new(self.project, line, self.router));
        self.others
            .borrow_mut()
            .insert(line.name.clone(), made.clone());
        made
    }

    /// The link to a page in another line: its link in this line, with this
    /// line's folder after the base path swapped for that line's (`3.4/`,
    /// or none for the current line).
    pub(crate) fn line_link(&self, line: &Build, page: &RelPath, id: Option<&str>) -> String {
        let own = self.router.link(page, id);
        let own_folder = self.build.line_folder().unwrap_or_default();
        let to_folder = line.line_folder().unwrap_or_default();
        let mut base = self.project.model().consumer.base_path.clone();
        if !base.starts_with('/') {
            base.insert(0, '/');
        }
        if !base.ends_with('/') {
            base.push('/');
        }
        // An older line's model has its folder in the base path; a model
        // that wasn't made for the line has the site's.
        let root = match base.strip_suffix(own_folder.as_str()) {
            Some(root) if !own_folder.is_empty() && root.ends_with('/') => root.to_owned(),
            _ => base.clone(),
        };
        let rest = own
            .strip_prefix(base.as_str())
            .or_else(|| (own == base.trim_end_matches('/')).then_some(""))
            .unwrap_or_else(|| own.trim_start_matches('/'));
        let url = format!("{root}{to_folder}{rest}");
        if rest.is_empty() && !own.ends_with('/') && url.len() > 1 {
            // A route with no trailing slash keeps none.
            return url.trim_end_matches('/').to_owned();
        }
        url
    }

    /// The problems with a page's `replaces` in this build (SPEC §2.1): a
    /// chain that comes back to the page, in any build; and, in a line,
    /// the page it replaces published too, or another page replacing the
    /// same one published too.
    pub(crate) fn replaces_problems(&self, index: &FileIndex) -> Vec<PageProblem> {
        let path = &index.path;
        let map = self.replacements();
        let Some(replaced) = map.get(path) else {
            return Vec::new();
        };
        let Some(span) = replaces_span(index) else {
            return Vec::new();
        };
        let at = Location::new(index.file, span);
        let problem = |issue: Issue| PageProblem {
            issue,
            via: Arc::from(Vec::new()),
        };
        // The chain from this page: back to it is a cycle; into a cycle
        // that doesn't hold it, that cycle's pages report it.
        let mut chain: Vec<&RelPath> = vec![path];
        let mut at_page = replaced;
        loop {
            if at_page == path {
                let issue = if chain.len() == 1 {
                    Issue::new(diagnostics::REPLACES_CYCLE, at)
                        .with_variant("itself")
                        .with_arg("path", path.to_string())
                } else {
                    let mut names: Vec<String> = chain.iter().map(|p| format!("`{p}`")).collect();
                    names.push(format!("`{path}`"));
                    Issue::new(diagnostics::REPLACES_CYCLE, at)
                        .with_arg("chain", names.join(" replaces "))
                };
                return vec![problem(issue)];
            }
            if chain.contains(&at_page) {
                break;
            }
            chain.push(at_page);
            match map.get(at_page) {
                Some(next) => at_page = next,
                None => break,
            }
        }
        if self.build.line.is_none() {
            return Vec::new();
        }
        let mut out = Vec::new();
        if self.is_published(replaced) {
            out.push(problem(
                Issue::new(diagnostics::REPLACES_OVERLAP, at)
                    .with_arg("build", self.build.name.clone())
                    .with_arg("path", path.to_string())
                    .with_arg("replaced", replaced.to_string()),
            ));
        }
        if let Some((other, _)) = map
            .iter()
            .find(|(other, r)| *other != path && *r == replaced && self.is_published(other))
        {
            out.push(problem(
                Issue::new(diagnostics::REPLACES_SAME_PAGE, at)
                    .with_arg("build", self.build.name.clone())
                    .with_arg("path", path.to_string())
                    .with_arg("other", other.to_string())
                    .with_arg("replaced", replaced.to_string()),
            ));
        }
        out
    }
}
