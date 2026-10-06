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
}

/// Splits `{a,b}` alternatives (no nesting) into separate patterns.
/// Escaped characters are kept escaped.
fn expand_braces(source: &str) -> Result<Vec<String>, PatternError> {
    let chars: Vec<char> = source.chars().collect();
    let mut i = 0;
    let mut open: Option<usize> = None;
    while i < chars.len() {
        match chars[i] {
            '\\' => i += 1,
            '{' => {
                if open.is_some() {
                    return Err(PatternError::Syntax(
                        "alternatives in {…} can't nest".into(),
                    ));
                }
                open = Some(i);
            }
            '}' => {
                let Some(start) = open else {
                    return Err(PatternError::Syntax("`}` without a matching `{`".into()));
                };
                let prefix: String = chars[..start].iter().collect();
                let suffix: String = chars[i + 1..].iter().collect();
                let inner: String = chars[start + 1..i].iter().collect();
                let mut out = Vec::new();
                for option in split_unescaped(&inner, ',') {
                    for rest in expand_braces(&format!("{option}{suffix}"))? {
                        out.push(format!("{prefix}{rest}"));
                    }
                }
                return Ok(out);
            }
            _ => {}
        }
        i += 1;
    }
    if open.is_some() {
        return Err(PatternError::Syntax("`{` without a matching `}`".into()));
    }
    Ok(vec![source.to_owned()])
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
