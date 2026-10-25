//! The pages most like a page that's gone, to offer as where to forward its
//! addresses: by the words of its headings and text, as the base had it,
//! against each page's now. A heading's words count twice, since a page's
//! headings say what it covers.

use std::collections::HashMap;

/// The paths of `candidates`, each a page's path and text, most like
/// `text` first, at most `limit` of them; a page with nothing in common
/// with it isn't one.
pub(super) fn most_like<'a>(
    text: &str,
    candidates: &[(&'a str, &str)],
    limit: usize,
) -> Vec<&'a str> {
    let gone = words(text);
    if gone.is_empty() {
        return Vec::new();
    }
    let mut scored: Vec<(f64, &str)> = candidates
        .iter()
        .map(|(path, text)| (likeness(&gone, &words(text)), *path))
        .filter(|(score, _)| *score > 0.0)
        .collect();
    // The higher score first; the path, for the same score.
    scored.sort_by(|a, b| b.0.total_cmp(&a.0).then_with(|| a.1.cmp(b.1)));
    scored
        .into_iter()
        .take(limit)
        .map(|(_, path)| path)
        .collect()
}

/// Words too common to say what a page is about.
const COMMON: &[&str] = &[
    "and", "are", "but", "can", "for", "from", "has", "have", "how", "its", "not", "one", "that",
    "the", "this", "use", "was", "what", "when", "which", "with", "you", "your",
];

/// Each word of `text` of three letters or more, but the most common, in lower case, with how
/// much it counts: once in the text, twice in a heading or the title.
fn words(text: &str) -> HashMap<String, f64> {
    let mut out: HashMap<String, f64> = HashMap::new();
    for line in text.lines() {
        let trimmed = line.trim_start();
        let (trimmed, weight) = match trimmed.strip_prefix("title:") {
            Some(title) => (title, 2.0),
            None if trimmed.starts_with('#') => (trimmed, 2.0),
            None => (trimmed, 1.0),
        };
        for word in trimmed
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| w.chars().count() >= 3)
            .map(str::to_lowercase)
            .filter(|w| !COMMON.contains(&w.as_str()))
        {
            *out.entry(word).or_default() += weight;
        }
    }
    out
}

/// How alike two pages' words are: the cosine of their weights, from 0 for
/// nothing in common to 1 for the same words in the same proportions.
fn likeness(a: &HashMap<String, f64>, b: &HashMap<String, f64>) -> f64 {
    let dot: f64 = a
        .iter()
        .filter_map(|(word, x)| b.get(word).map(|y| x * y))
        .sum();
    let norm = |m: &HashMap<String, f64>| m.values().map(|x| x * x).sum::<f64>().sqrt();
    let both = norm(a) * norm(b);
    if both == 0.0 { 0.0 } else { dot / both }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_page_with_the_most_words_in_common_comes_first() {
        let gone = "---\ntitle: Configure options\n---\n\n## Options\n\nSet the timeout option.\n";
        let candidates = [
            (
                "guides/install.md",
                "---\ntitle: Install\n---\n\nInstall it with npm.\n",
            ),
            (
                "guides/configure.md",
                "---\ntitle: Configure\n---\n\n## Options\n\nThe timeout option, and others.\n",
            ),
            (
                "index.md",
                "---\ntitle: Home\n---\n\nWelcome. Configure it later.\n",
            ),
        ];
        assert_eq!(
            most_like(gone, &candidates, 3),
            ["guides/configure.md", "index.md"]
        );
        assert_eq!(most_like(gone, &candidates, 1), ["guides/configure.md"]);
        assert!(most_like("", &candidates, 3).is_empty());
    }
}
