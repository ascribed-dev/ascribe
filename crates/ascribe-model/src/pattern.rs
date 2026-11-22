//! Glob patterns.

/// A compiled pattern, matched against a path relative to the content root.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pattern {
    source: String,
    /// One entry per `{a,b}` alternative; each is a list of segments.
    alternatives: Vec<Vec<Segment>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Segment {
    /// `**`: any number of whole segments.
    AnyDepth,
    /// A segment glob.
    Glob(Vec<Tok>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Tok {
    Char(char),
    Star,
    Any,
}

/// Why a pattern is invalid.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PatternError {
    /// Starts with `/`.
    LeadingSlash,
    /// Has a `..` segment.
    Parent,
    /// Any other syntax problem.
    Syntax(String),
}

impl Pattern {
    /// Compiles a pattern.
    pub fn new(source: &str) -> Result<Pattern, PatternError> {
        if source.is_empty() {
            return Err(PatternError::Syntax("the pattern is empty".into()));
        }
        if source.starts_with('/') {
            return Err(PatternError::LeadingSlash);
        }
        let mut alternatives = Vec::new();
        for alt in expand_braces(source)? {
            let mut segments = Vec::new();
            for seg in alt.split('/') {
                if seg == ".." {
                    return Err(PatternError::Parent);
                }
                if seg.is_empty() {
                    return Err(PatternError::Syntax(
                        "empty path segment (a doubled or trailing /)".into(),
                    ));
                }
                if seg == "**" {
                    segments.push(Segment::AnyDepth);
                    continue;
                }
                if seg.contains("**") {
                    return Err(PatternError::Syntax(
                        "`**` must be a whole path segment".into(),
                    ));
                }
                segments.push(Segment::Glob(compile_segment(seg)?));
            }
            alternatives.push(segments);
        }
        Ok(Pattern {
            source: source.to_owned(),
            alternatives,
        })
    }

    /// The pattern as written.
    pub fn as_str(&self) -> &str {
        &self.source
    }

    /// Whether `path` (relative to the content root, `/`-separated) matches.
    pub fn matches(&self, path: &str) -> bool {
        let segments: Vec<&str> = path.split('/').collect();
        self.alternatives
            .iter()
            .any(|alt| match_segments(alt, &segments))
    }

    /// Whether `path` matches, or is a folder that may hold a path that
    /// does: `a/b` for `a/b/**/*.rs`. The empty path is the folder the
    /// pattern is relative to, which holds any match.
    pub fn may_match_in(&self, path: &str) -> bool {
        let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        self.alternatives
            .iter()
            .any(|alt| match_leading_segments(alt, &segments))
    }
}

/// The most alternatives a pattern's `{a,b}` groups may expand to: the
/// product of each group's count. Past it, compiling and matching would
/// grow with the product, which doubles with each two-way group.
const MAX_ALTERNATIVES: usize = 256;

/// Splits `{a,b}` alternatives (no nesting) into separate patterns.
/// Escaped characters are kept escaped.
fn expand_braces(source: &str) -> Result<Vec<String>, PatternError> {
    // The pattern as literal runs and groups, in order: one pass, so the
    // work is linear in the pattern before anything is expanded.
    let mut parts: Vec<Vec<String>> = Vec::new();
    let mut literal = String::new();
    let mut group: Option<String> = None;
    let mut chars = source.chars();
    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                let text = group.as_mut().unwrap_or(&mut literal);
                text.push(c);
                if let Some(next) = chars.next() {
                    text.push(next);
                }
            }
            '{' => {
                if group.is_some() {
                    return Err(PatternError::Syntax(
                        "alternatives in {…} can't nest".into(),
                    ));
                }
                parts.push(vec![std::mem::take(&mut literal)]);
                group = Some(String::new());
            }
            '}' => {
                let Some(inner) = group.take() else {
                    return Err(PatternError::Syntax("`}` without a matching `{`".into()));
                };
                parts.push(split_unescaped(&inner, ','));
            }
            _ => group.as_mut().unwrap_or(&mut literal).push(c),
        }
    }
    if group.is_some() {
        return Err(PatternError::Syntax("`{` without a matching `}`".into()));
    }
    parts.push(vec![literal]);

    let count = parts
        .iter()
        .try_fold(1usize, |n, options| n.checked_mul(options.len()))
        .filter(|n| *n <= MAX_ALTERNATIVES);
    if count.is_none() {
        return Err(PatternError::Syntax(format!(
            "its {{…}} alternatives expand to more than {MAX_ALTERNATIVES} patterns"
        )));
    }
    let mut out = vec![String::new()];
    for options in &parts {
        out = match options.as_slice() {
            [only] => {
                for prefix in &mut out {
                    prefix.push_str(only);
                }
                out
            }
            _ => out
                .iter()
                .flat_map(|prefix| options.iter().map(move |o| format!("{prefix}{o}")))
                .collect(),
        };
    }
    Ok(out)
}

fn push_last(out: &mut [String], c: char) {
    if let Some(l) = out.last_mut() {
        l.push(c);
    }
}

fn split_unescaped(s: &str, sep: char) -> Vec<String> {
    let mut out = vec![String::new()];
    let mut escaped = false;
    for c in s.chars() {
        if escaped {
            push_last(&mut out, c);
            escaped = false;
        } else if c == '\\' {
            push_last(&mut out, c);
            escaped = true;
        } else if c == sep {
            out.push(String::new());
        } else {
            push_last(&mut out, c);
        }
    }
    out
}

fn compile_segment(seg: &str) -> Result<Vec<Tok>, PatternError> {
    let mut toks = Vec::new();
    let mut chars = seg.chars();
    while let Some(c) = chars.next() {
        match c {
            '\\' => match chars.next() {
                Some(next) => toks.push(Tok::Char(next)),
                None => {
                    return Err(PatternError::Syntax(
                        "a trailing `\\` escapes nothing".into(),
                    ));
                }
            },
            '*' => {
                if toks.last() != Some(&Tok::Star) {
                    toks.push(Tok::Star);
                }
            }
            '?' => toks.push(Tok::Any),
            other => toks.push(Tok::Char(other)),
        }
    }
    Ok(toks)
}

fn match_segments(pattern: &[Segment], path: &[&str]) -> bool {
    match pattern.split_first() {
        None => path.is_empty(),
        Some((Segment::AnyDepth, rest)) => {
            (0..=path.len()).any(|skip| match_segments(rest, &path[skip..]))
        }
        Some((Segment::Glob(toks), rest)) => match path.split_first() {
            Some((first, tail)) => match_glob(toks, first) && match_segments(rest, tail),
            None => false,
        },
    }
}

/// [`match_segments`], or `path` running out first: a folder the rest of
/// the pattern may match below.
fn match_leading_segments(pattern: &[Segment], path: &[&str]) -> bool {
    match pattern.split_first() {
        None => path.is_empty(),
        _ if path.is_empty() => true,
        Some((Segment::AnyDepth, rest)) => {
            (0..=path.len()).any(|skip| match_leading_segments(rest, &path[skip..]))
        }
        Some((Segment::Glob(toks), rest)) => {
            let (first, tail) = (path[0], &path[1..]);
            match_glob(toks, first) && match_leading_segments(rest, tail)
        }
    }
}

fn match_glob(toks: &[Tok], text: &str) -> bool {
    let text: Vec<char> = text.chars().collect();
    // Classic wildcard matching with backtracking on the last star.
    let (mut t, mut p) = (0, 0);
    let mut star: Option<(usize, usize)> = None;
    while t < text.len() {
        match toks.get(p) {
            Some(Tok::Star) => {
                star = Some((p, t));
                p += 1;
            }
            Some(Tok::Any) => {
                t += 1;
                p += 1;
            }
            Some(Tok::Char(c)) if *c == text[t] => {
                t += 1;
                p += 1;
            }
            _ => match star {
                Some((sp, st)) => {
                    p = sp + 1;
                    t = st + 1;
                    star = Some((sp, st + 1));
                }
                None => return false,
            },
        }
    }
    toks[p..].iter().all(|t| *t == Tok::Star)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m(pattern: &str, path: &str) -> bool {
        Pattern::new(pattern).unwrap().matches(path)
    }

    #[test]
    fn matching() {
        assert!(m("reference/**", "reference/a/b.md"));
        assert!(m("reference/**", "reference/a.md"));
        assert!(!m("reference/**", "guides/a.md"));
        assert!(m("**/*.partial.md", "a/b/x.partial.md"));
        assert!(m("**/*.partial.md", "x.partial.md"));
        assert!(m("a/**/b.md", "a/b.md") && m("a/**/b.md", "a/x/y/b.md"));
        assert!(m("*.md", "a.md") && !m("*.md", "d/a.md"));
        assert!(m("a?.md", "ab.md") && !m("a?.md", "a.md"));
        assert!(m("{a,b}/*.md", "b/x.md") && !m("{a,b}/*.md", "c/x.md"));
        assert!(m("a\\*.md", "a*.md") && !m("a\\*.md", "ab.md"));
        assert!(!m("Ref/**", "ref/a.md"));
        assert!(m("a*b*c", "aXbYc") && !m("a*b*c", "aXbY"));
    }

    #[test]
    fn a_folder_that_may_hold_a_match() {
        let may = |pattern: &str, path: &str| Pattern::new(pattern).unwrap().may_match_in(path);
        assert!(may("crates/cli/tests/output/**", "crates"));
        assert!(may("crates/cli/tests/output/**", "crates/cli"));
        assert!(may(
            "crates/cli/tests/output/**",
            "crates/cli/tests/output/a.txt"
        ));
        assert!(!may("crates/cli/tests/output/**", "target"));
        assert!(!may("crates/cli/tests/output/**", "crates/core"));
        assert!(may("**/*.rs", "target/debug"));
        assert!(may("{a,b}/*.md", "b") && !may("{a,b}/*.md", "c"));
        assert!(!may("a/*.md", "a/b.md/c"));
        assert!(may("a/*.md", ""));
    }

    #[test]
    fn several_groups_match_every_combination() {
        let p = Pattern::new("{a,b}/{x,y,z}/*.{md,txt}").unwrap();
        assert_eq!(p.alternatives.len(), 12);
        for path in ["a/x/1.md", "b/z/2.txt", "a/y/3.txt"] {
            assert!(p.matches(path), "{path}");
        }
        for path in ["c/x/1.md", "a/w/1.md", "a/x/1.rs", "a/1.md"] {
            assert!(!p.matches(path), "{path}");
        }
        assert!(m("{a/b,c}/*.md", "a/b/x.md") && m("{a/b,c}/*.md", "c/x.md"));
        assert!(m("x{,y}.md", "x.md") && m("x{,y}.md", "xy.md"));
        assert!(m("\\{a,b\\}.md", "{a,b}.md") && !m("\\{a,b\\}.md", "a.md"));
        assert!(m("{a\\,b,c}.md", "a,b.md") && m("{a\\,b,c}.md", "c.md"));
    }

    #[test]
    fn alternatives_are_capped() {
        // 2^8 = 256 is the most; one more two-way group is past it.
        let groups = |n: usize| "{a,b}".repeat(n);
        assert_eq!(Pattern::new(&groups(8)).unwrap().alternatives.len(), 256);
        assert!(matches!(
            Pattern::new(&groups(9)),
            Err(PatternError::Syntax(detail)) if detail.contains("more than 256")
        ));
        // So many groups that the product overflows still fails fast.
        assert!(matches!(
            Pattern::new(&groups(200)),
            Err(PatternError::Syntax(_))
        ));
        // A group of one multiplies nothing, however many there are.
        let p = Pattern::new(&format!("{}.md", "{a}".repeat(10_000))).unwrap();
        assert_eq!(p.alternatives.len(), 1);
        assert!(p.matches(&format!("{}.md", "a".repeat(10_000))));
    }

    #[test]
    fn errors() {
        assert_eq!(Pattern::new("/a"), Err(PatternError::LeadingSlash));
        assert_eq!(Pattern::new("a/../b"), Err(PatternError::Parent));
        for bad in ["", "a/**b", "{a,b", "a}", "{{a}}", "a\\", "a//b"] {
            assert!(
                matches!(Pattern::new(bad), Err(PatternError::Syntax(_))),
                "{bad}"
            );
        }
    }
}
