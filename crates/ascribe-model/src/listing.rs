//! `[listing]`: how a project's pages are listed, set once for many: the
//! site's `default`, `older-lines` for the pages of a versioned build's older
//! lines, `[listing.folders]` by a folder from the content root, and
//! `[listing.types]` by a content type. Each value is one of a page's
//! `listing` values. A rule under a stricter one is an error, since a
//! narrower setting can only restrict: a folder under a stricter folder, and
//! a folder, a type, or `older-lines` under a stricter default.

use std::path::Path;

use ascribe_core::{Span, diagnostics};

use crate::loader::Loader;
use crate::model::{ContentType, Listing, ListingLevel, ListingRule, ListingRules};
use crate::names::list;
use crate::toml_util::{V, entries, join, sp};

impl Loader<'_> {
    /// `[listing]`: `default`, `older-lines`, `folders`, and `types`. `types` are the
    /// model's content types, and `content_root` is where folders are from.
    pub(crate) fn listing(
        &mut self,
        v: Option<&V<'_>>,
        types: &[ContentType],
        content_root: &str,
    ) -> ListingRules {
        let mut out = ListingRules::default();
        let Some(v) = v else { return out };
        let Some(t) = self.as_table("listing", v) else {
            return out;
        };
        self.check_keys(
            "listing",
            t,
            &["default", "older-lines", "folders", "types"],
        );
        for (key, key_span, value) in entries(t) {
            if key == "default"
                && let Some(listing) = self.listing_value("listing.default", value)
            {
                out.default = Some(self.listing_rule(ListingLevel::Site, listing, key_span, value));
            }
            if key == "older-lines"
                && let Some(listing) = self.listing_value("listing.older-lines", value)
            {
                out.older_lines =
                    Some(self.listing_rule(ListingLevel::OlderLines, listing, key_span, value));
            }
        }
        if let Some(folders) = t.get("folders")
            && let Some(folders) = self.as_table("listing.folders", folders)
        {
            for (folder, key_span, value) in entries(folders) {
                let path = join("listing.folders", folder);
                let Some(listing) = self.listing_value(&path, value) else {
                    continue;
                };
                if !self.listing_folder(folder, key_span, content_root) {
                    continue;
                }
                let level = ListingLevel::Folder(folder.to_owned());
                out.folders
                    .push(self.listing_rule(level, listing, key_span, value));
            }
        }
        if let Some(named) = t.get("types")
            && let Some(named) = self.as_table("listing.types", named)
        {
            for (name, key_span, value) in entries(named) {
                let path = join("listing.types", name);
                let Some(listing) = self.listing_value(&path, value) else {
                    continue;
                };
                if !types.iter().any(|t| t.name == name) {
                    let names: Vec<&str> = types.iter().map(|t| t.name.as_str()).collect();
                    self.push(
                        self.issue(diagnostics::MODEL_LISTING_TYPE, key_span)
                            .with_arg("type", name)
                            .with_arg("values", list(&names)),
                    );
                    continue;
                }
                let level = ListingLevel::Type(name.to_owned());
                out.types
                    .push(self.listing_rule(level, listing, key_span, value));
            }
        }
        self.listing_looser(&out);
        out
    }

    /// A rule's value: one of `listing`'s three.
    fn listing_value(&mut self, path: &str, v: &V<'_>) -> Option<Listing> {
        let names: Vec<&str> = Listing::ALL.iter().map(|l| l.name()).collect();
        let name = self.choice(path, v, &names)?;
        Listing::from_name(&name)
    }

    fn listing_rule(
        &self,
        level: ListingLevel,
        listing: Listing,
        key_span: Span,
        value: &V<'_>,
    ) -> ListingRule {
        let span = Span::new(key_span.start(), sp(value).end());
        let line = self
            .src
            .get(..span.start())
            .unwrap_or_default()
            .matches('\n')
            .count()
            + 1;
        ListingRule {
            level,
            listing,
            text: self.text_of(span).to_owned(),
            line,
            span,
        }
    }

    /// Whether a key of `[listing.folders]` is a folder's path from the
    /// content root, ending in `/`, and the folder is there, with its name's
    /// case as written. It's looked for only when the project's folder is
    /// known.
    fn listing_folder(&mut self, folder: &str, span: Span, content_root: &str) -> bool {
        let segments: Vec<&str> = folder
            .strip_suffix('/')
            .unwrap_or_default()
            .split('/')
            .collect();
        let shaped = folder.ends_with('/')
            && !folder.contains(['\\', '*', '?', '[', ']', '{', '}'])
            && segments
                .iter()
                .all(|s| !s.is_empty() && *s != "." && *s != "..");
        if !shaped {
            self.push(
                self.issue(diagnostics::MODEL_LISTING_FOLDER, span)
                    .with_variant("shape")
                    .with_arg("folder", folder),
            );
            return false;
        }
        let Some(dir) = self.project_dir else {
            return true;
        };
        let mut at = dir.join(content_root);
        for segment in segments {
            match exact_entry(&at, segment) {
                Some(true) => at.push(segment),
                Some(false) => {
                    self.push(
                        self.issue(diagnostics::MODEL_LISTING_FOLDER, span)
                            .with_variant("not-directory")
                            .with_arg("folder", folder),
                    );
                    return false;
                }
                None => {
                    self.push(
                        self.issue(diagnostics::MODEL_LISTING_FOLDER, span)
                            .with_arg("folder", folder),
                    );
                    return false;
                }
            }
        }
        true
    }

    /// A rule under a stricter one: a folder under a stricter folder, and a
    /// folder, a type, or `older-lines` under a stricter `default`.
    fn listing_looser(&mut self, rules: &ListingRules) {
        let narrower = rules
            .older_lines
            .iter()
            .chain(&rules.folders)
            .chain(&rules.types);
        for rule in narrower {
            let wider = rules
                .folders
                .iter()
                .filter(|w| match (&w.level, &rule.level) {
                    (ListingLevel::Folder(w), ListingLevel::Folder(f)) => {
                        f.len() > w.len() && f.starts_with(w.as_str())
                    }
                    _ => false,
                })
                .chain(rules.default.iter())
                .filter(|w| w.listing > rule.listing)
                .max_by_key(|w| w.listing);
            let Some(wider) = wider else { continue };
            let mut issue = self
                .issue(diagnostics::MODEL_LISTING_LOOSER, rule.span)
                .with_arg("rule", rule.text.as_str())
                .with_arg("wider", wider.text.as_str())
                .with_arg("line", wider.line.to_string());
            if wider.level == ListingLevel::Site {
                issue = issue.with_variant("default");
            }
            self.push(issue);
        }
    }
}

/// Whether `dir` holds an entry named exactly `name`: `Some(true)` for a
/// folder, `Some(false)` for anything else, and `None` when there's none,
/// even one whose name differs only in case.
fn exact_entry(dir: &Path, name: &str) -> Option<bool> {
    // Outside FileSystem: the content model is checked before there's a
    // project. This asks only whether the folder it names is there, by its
    // exact name, as a page's path is matched against it.
    let entries = std::fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        if entry.file_name().to_str() == Some(name) {
            return Some(entry.path().is_dir());
        }
    }
    None
}
