//! `[crawlers]`: what a project says to the crawlers that read its site.
//! `lines` holds the project's own `robots.txt` lines, each one line and not
//! empty. What a line says isn't read here: a build writes the lines as they
//! are, with the base path put before each `Allow` and `Disallow` path, and
//! `ascribe check` reads the ones it can.

use ascribe_core::diagnostics;

use crate::loader::Loader;
use crate::model::{CrawlerLine, Crawlers};
use crate::toml_util::V;

impl Loader<'_> {
    /// `[crawlers]`: `lines`, an array of single lines.
    pub(crate) fn crawlers(&mut self, v: Option<&V<'_>>) -> Crawlers {
        let mut out = Crawlers::default();
        let Some(v) = v else { return out };
        let Some(t) = self.as_table("crawlers", v) else {
            return out;
        };
        self.check_keys("crawlers", t, &["lines"]);
        let Some(lines) = t.get("lines") else {
            return out;
        };
        let Some(lines) = self.strings("crawlers.lines", lines) else {
            return out;
        };
        for (n, (text, span)) in lines.into_iter().enumerate() {
            let key = format!("crawlers.lines[{n}]");
            if text.trim().is_empty() {
                self.push(
                    self.issue(diagnostics::MODEL_EMPTY_TEXT, span)
                        .with_arg("key", key),
                );
            } else if text.contains(['\n', '\r']) {
                self.push(
                    self.issue(diagnostics::MODEL_CRAWLERS_LINE, span)
                        .with_arg("key", key),
                );
            } else {
                out.lines.push(CrawlerLine { text, span });
            }
        }
        out
    }
}
