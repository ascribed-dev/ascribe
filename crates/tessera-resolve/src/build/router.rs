//! A simple default [`Router`], for tests and for callers with no consumer
//! profile of their own.
//!
//! The `astro` profile has its own ([`crate::AstroRouter`]). This one follows
//! the common convention: a page's route is the base path, the page's path
//! without `.md`, and a trailing slash; `index.md` is its directory's route.

use tessera_core::{RelPath, Router};
use tessera_model::{Consumer, TrailingSlash};

/// Routes pages by their source paths under a base path.
///
/// `guides/setup.md` is `<base>guides/setup/`, and `guides/index.md` is
/// `<base>guides/`. Segments are percent-encoded where a URL path needs it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DefaultRouter {
    base: String,
    trailing_slash: bool,
}

impl DefaultRouter {
    /// A router with base path `/` and trailing slashes.
    pub fn new() -> DefaultRouter {
        DefaultRouter {
            base: "/".to_owned(),
            trailing_slash: true,
        }
    }

    /// A router under a base path, such as `/docs/` (a missing leading or
    /// trailing slash is added).
    pub fn with_base(base: &str, trailing_slash: bool) -> DefaultRouter {
        let mut base = base.trim().to_owned();
        if !base.starts_with('/') {
            base.insert(0, '/');
        }
        if !base.ends_with('/') {
            base.push('/');
        }
        DefaultRouter {
            base,
            trailing_slash,
        }
    }

    /// A router with the content model's `[consumer]` base path and
    /// trailing-slash policy.
    pub fn from_consumer(consumer: &Consumer) -> DefaultRouter {
        DefaultRouter::with_base(
            &consumer.base_path,
            consumer.trailing_slash == TrailingSlash::Always,
        )
    }
}

impl Default for DefaultRouter {
    fn default() -> DefaultRouter {
        DefaultRouter::new()
    }
}

impl Router for DefaultRouter {
    fn route(&self, page: &RelPath) -> String {
        let mut segments: Vec<&str> = page.segments().collect();
        if let Some(last) = segments.pop() {
            let stem = last.strip_suffix(".md").unwrap_or(last);
            if stem != "index" {
                segments.push(stem);
            }
        }
        let mut url = self.base.clone();
        let encoded: Vec<String> = segments.iter().map(|s| encode(s)).collect();
        url.push_str(&encoded.join("/"));
        if !segments.is_empty() && self.trailing_slash {
            url.push('/');
        }
        url
    }
}

/// Percent-encodes a path segment: everything but unreserved characters.
fn encode(segment: &str) -> String {
    let mut out = String::with_capacity(segment.len());
    for byte in segment.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            out.push(char::from(byte));
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn route(router: &DefaultRouter, path: &str) -> String {
        router.route(&RelPath::parse(path).expect("a path"))
    }

    #[test]
    fn pages_route_by_their_paths() {
        let router = DefaultRouter::new();
        assert_eq!(route(&router, "index.md"), "/");
        assert_eq!(route(&router, "keys.md"), "/keys/");
        assert_eq!(route(&router, "guides/index.md"), "/guides/");
        assert_eq!(route(&router, "guides/setup.md"), "/guides/setup/");
    }

    #[test]
    fn a_base_path_and_no_trailing_slash() {
        let router = DefaultRouter::with_base("docs", false);
        assert_eq!(route(&router, "index.md"), "/docs/");
        assert_eq!(route(&router, "guides/setup.md"), "/docs/guides/setup");
    }

    #[test]
    fn segments_are_percent_encoded() {
        let router = DefaultRouter::new();
        assert_eq!(route(&router, "my guides/a#b.md"), "/my%20guides/a%23b/");
    }
}
