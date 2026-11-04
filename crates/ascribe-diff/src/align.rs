//! Aligning the blocks of two versions of a page, and working out what was
//! added, removed, changed, and moved.
//!
//! The blocks of one list (a page's top-level blocks, a container's blocks,
//! a list's items, a group's arms) are aligned by a longest-common-
//! subsequence diff over their fingerprints. In each run the diff couldn't
//! match, a block whose fingerprint is unmatched on the other side too is
//! left for the move pass; the rest are paired in order with a block of the
//! same kind whose text is similar enough, or keeps the other's words whole
//! at one end, as **changed**; what's left is
//! **added** or **removed**. A changed pair whose blocks contain other blocks
//! is compared the same way inside. Last, a removed block with the same
//! fingerprint as an added block anywhere on the page is **moved**.

use std::collections::HashMap;

use similar::{Algorithm, DiffTag, capture_diff_slices};

use crate::tree::{Anchor, Node};

/// Two blocks whose words overlap at least this much (the Dice coefficient
/// of their words) are a changed block rather than a removal and an
/// addition.
pub const SIMILARITY: f64 = 0.5;

/// Above this many blocks in either version of a list (a container's
/// blocks, a list's items), a changed pair isn't compared inside: the
/// container is marked changed as a whole.
pub const MAX_CHILDREN: usize = 1000;

/// Above this many candidate pairs in one unmatched run, blocks aren't
/// paired as changed: the run is removed blocks and added blocks.
pub const MAX_PAIRS: usize = 10_000;

/// A change found on a page, before moves are worked out.
#[derive(Clone, Debug)]
pub(crate) enum Found<'a> {
    /// A block whose own content changed. Its children may have changes of
    /// their own, listed separately.
    Changed { was: &'a Node, now: &'a Node },
    /// A block that's only in the new version.
    Added { now: &'a Node },
    /// A block that's only in the old version.
    Removed {
        was: &'a Node,
        /// The new version's block it came after, among its siblings.
        after: Option<Anchor>,
        /// The new version's block it was inside.
        parent: Option<Anchor>,
    },
    /// A block that's in both versions, somewhere else.
    Moved {
        was: &'a Node,
        now: &'a Node,
        after: Option<Anchor>,
        parent: Option<Anchor>,
    },
}

/// The changes between two versions of a page's blocks, in the new
/// version's order, a removed block where it was.
pub(crate) fn compare<'a>(old: &'a [Node], new: &'a [Node]) -> Vec<Found<'a>> {
    let mut found = Vec::new();
    align(old, new, None, &mut found);
    pair_moves(found)
}

/// An entry of one list's alignment, in the order the changes are listed.
enum Step {
    /// `old[i]` and `new[j]` are the same block, changed or not.
    Pair(usize, usize),
    Added(usize),
    Removed(usize),
}

fn align<'a>(
    old: &'a [Node],
    new: &'a [Node],
    parent: Option<&'a Node>,
    found: &mut Vec<Found<'a>>,
) {
    let old_hashes: Vec<u64> = old.iter().map(|n| n.hash).collect();
    let new_hashes: Vec<u64> = new.iter().map(|n| n.hash).collect();
    // Which new block each old block is, when it's matched.
    let mut matched: Vec<Option<usize>> = vec![None; old.len()];
    let mut new_matched = vec![false; new.len()];
    // The runs the diff couldn't match.
    let mut runs: Vec<(Vec<usize>, Vec<usize>)> = Vec::new();
    let mut in_run = false;
    for op in capture_diff_slices(Algorithm::Myers, &old_hashes, &new_hashes) {
        let (tag, old_range, new_range) = op.as_tag_tuple();
        if tag == DiffTag::Equal {
            for (i, j) in old_range.zip(new_range) {
                matched[i] = Some(j);
                new_matched[j] = true;
            }
            in_run = false;
            continue;
        }
        // A deletion next to an insertion is one run.
        match runs.last_mut() {
            Some(last) if in_run => {
                last.0.extend(old_range);
                last.1.extend(new_range);
            }
            _ => runs.push((old_range.collect(), new_range.collect())),
        }
        in_run = true;
    }

    // A block unmatched on both sides with the same fingerprint moved: keep
    // it out of the pairing, for the move pass.
    let mut unmatched_new: HashMap<u64, Vec<usize>> = HashMap::new();
    for (_, news) in &runs {
        for &j in news {
            unmatched_new.entry(new[j].hash).or_default().push(j);
        }
    }
    let mut moved_old = vec![false; old.len()];
    let mut moved_new = vec![false; new.len()];
    for (olds, _) in &runs {
        for &i in olds {
            if let Some(js) = unmatched_new.get_mut(&old[i].hash)
                && !js.is_empty()
            {
                moved_old[i] = true;
                moved_new[js.remove(0)] = true;
            }
        }
    }

    // Pair the rest of each run by kind and similarity, in order.
    for (olds, news) in &runs {
        let olds: Vec<usize> = olds.iter().copied().filter(|&i| !moved_old[i]).collect();
        let news: Vec<usize> = news.iter().copied().filter(|&j| !moved_new[j]).collect();
        if olds.is_empty() || news.is_empty() || olds.len() * news.len() > MAX_PAIRS {
            continue;
        }
        for (k, l) in pair_run(old, new, &olds, &news) {
            matched[olds[k]] = Some(news[l]);
            new_matched[news[l]] = true;
        }
    }

    // The steps in the new version's order, each removed block just before
    // the block that follows the one it came after.
    let mut keyed: Vec<((usize, u8), Step)> = Vec::new();
    for (i, m) in matched.iter().enumerate() {
        match m {
            Some(j) => keyed.push(((*j, 1), Step::Pair(i, *j))),
            None => {
                let next = after_of(&matched, i).map_or(0, |j| j + 1);
                keyed.push(((next, 0), Step::Removed(i)));
            }
        }
    }
    for (j, taken) in new_matched.iter().enumerate() {
        if !taken {
            keyed.push(((j, 1), Step::Added(j)));
        }
    }
    keyed.sort_by_key(|(key, _)| *key);

    let parent_anchor = parent.map(|p| p.anchor.clone());
    for (_, step) in keyed {
        match step {
            Step::Pair(i, j) => {
                let (was, now) = (&old[i], &new[j]);
                if was.hash == now.hash {
                    continue;
                }
                let too_big =
                    was.children.len() > MAX_CHILDREN || now.children.len() > MAX_CHILDREN;
                if was.own != now.own || too_big {
                    found.push(Found::Changed { was, now });
                }
                if !too_big {
                    align(&was.children, &now.children, Some(now), found);
                }
            }
            Step::Added(j) => found.push(Found::Added { now: &new[j] }),
            Step::Removed(i) => found.push(Found::Removed {
                was: &old[i],
                after: after_of(&matched, i).map(|j| new[j].anchor.clone()),
                parent: parent_anchor.clone(),
            }),
        }
    }
}

/// Pairs the blocks of one unmatched run as changed: of the pairings that
/// keep both versions' order and pair only blocks of the same kind that are
/// alike (their words overlap at least [`SIMILARITY`], or one keeps the
/// other's words whole at one end), the one whose pairs' similarities add up to the most. So an
/// edited block pairs with its new text rather than with a block inserted
/// beside it that happens to share a few of its words. Returns indexes into
/// `olds` and `news`, in order.
fn pair_run(old: &[Node], new: &[Node], olds: &[usize], news: &[usize]) -> Vec<(usize, usize)> {
    let old_words: Vec<Vec<String>> = olds.iter().map(|&i| words(&old[i].all_text())).collect();
    let new_words: Vec<Vec<String>> = news.iter().map(|&j| words(&new[j].all_text())).collect();
    let (n, m) = (olds.len(), news.len());
    // A pair's score, when its blocks may pair. An extension scores above
    // nothing even when it shares few words, since it's still a pair.
    let score = |k: usize, l: usize| -> Option<f64> {
        if old[olds[k]].kind != new[news[l]].kind {
            return None;
        }
        let (a, b) = (&old_words[k], &new_words[l]);
        let s = dice(a, b);
        if s >= SIMILARITY {
            Some(s)
        } else if extends_words(a, b) {
            Some(s.max(f64::EPSILON))
        } else {
            None
        }
    };
    // best[k][l]: the most the pairs of olds[..k] and news[..l] can score,
    // and how: pairing the last of each, or leaving the last old or new one.
    #[derive(Clone, Copy)]
    enum Last {
        Pair,
        SkipOld,
        SkipNew,
    }
    let mut best = vec![vec![0.0_f64; m + 1]; n + 1];
    let mut how = vec![vec![Last::SkipOld; m + 1]; n + 1];
    for k in 1..=n {
        for l in 1..=m {
            let (mut top, mut last) = if best[k - 1][l] >= best[k][l - 1] {
                (best[k - 1][l], Last::SkipOld)
            } else {
                (best[k][l - 1], Last::SkipNew)
            };
            if let Some(s) = score(k - 1, l - 1)
                && best[k - 1][l - 1] + s > top
            {
                (top, last) = (best[k - 1][l - 1] + s, Last::Pair);
            }
            best[k][l] = top;
            how[k][l] = last;
        }
    }
    let mut out = Vec::new();
    let (mut k, mut l) = (n, m);
    while k > 0 && l > 0 {
        match how[k][l] {
            Last::Pair => {
                out.push((k - 1, l - 1));
                k -= 1;
                l -= 1;
            }
            Last::SkipOld => k -= 1,
            Last::SkipNew => l -= 1,
        }
    }
    out.reverse();
    out
}

/// The new version's index of the nearest block before `old[i]` that's in
/// both versions.
fn after_of(matched: &[Option<usize>], i: usize) -> Option<usize> {
    matched[..i].iter().rev().find_map(|m| *m)
}

/// A text's words: runs of letters and digits, lowercased, so punctuation
/// and code spans don't count, and "later." and "later," are one word.
fn words(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_lowercase)
        .collect()
}

/// Whether one text's words, in order, start or end the other's: "Run
/// `x`." starts "Run `x` to remove it.".
#[cfg(test)]
fn extends(a: &str, b: &str) -> bool {
    extends_words(&words(a), &words(b))
}

fn extends_words(a: &[String], b: &[String]) -> bool {
    let (short, long) = if a.len() <= b.len() { (a, b) } else { (b, a) };
    !short.is_empty() && (long.starts_with(short) || long.ends_with(short))
}

/// How much two texts' words overlap: the Dice coefficient of their
/// multisets of [`words`]. Two texts without words are the same.
#[cfg(test)]
fn similarity(a: &str, b: &str) -> f64 {
    dice(&words(a), &words(b))
}

fn dice(a: &[String], b: &[String]) -> f64 {
    let total = a.len() + b.len();
    if total == 0 {
        return 1.0;
    }
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for w in a {
        *counts.entry(w).or_insert(0) += 1;
    }
    let mut common = 0;
    for w in b {
        if let Some(n) = counts.get_mut(w.as_str())
            && *n > 0
        {
            *n -= 1;
            common += 1;
        }
    }
    (2 * common) as f64 / total as f64
}

/// Turns each removed block with the same fingerprint as an added block
/// into a move, listed where the block is now.
fn pair_moves(found: Vec<Found<'_>>) -> Vec<Found<'_>> {
    let mut added: HashMap<u64, Vec<usize>> = HashMap::new();
    for (k, f) in found.iter().enumerate() {
        if let Found::Added { now } = f {
            added.entry(now.hash).or_default().push(k);
        }
    }
    // For each added block that's a move's new end, the removed block it
    // came from.
    let mut from: HashMap<usize, usize> = HashMap::new();
    let mut gone = vec![false; found.len()];
    for (k, f) in found.iter().enumerate() {
        if let Found::Removed { was, .. } = f
            && let Some(ks) = added.get_mut(&was.hash)
            && !ks.is_empty()
        {
            from.insert(ks.remove(0), k);
            gone[k] = true;
        }
    }
    let mut out = Vec::with_capacity(found.len());
    for (k, f) in found.iter().enumerate() {
        if gone[k] {
            continue;
        }
        match (f, from.get(&k).map(|r| &found[*r])) {
            (
                Found::Added { now },
                Some(Found::Removed {
                    was, after, parent, ..
                }),
            ) => out.push(Found::Moved {
                was,
                now,
                after: after.clone(),
                parent: parent.clone(),
            }),
            _ => out.push(f.clone()),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn similarity_is_word_overlap() {
        assert!((similarity("a b c d", "a b c e") - 0.75).abs() < 1e-9);
        assert!(similarity("one two", "three four") < SIMILARITY);
        assert!((similarity("", "") - 1.0).abs() < 1e-9);
        assert!((similarity("Same Words", "same words") - 1.0).abs() < 1e-9);
    }

    #[test]
    fn a_text_extends_another_kept_whole_at_one_end() {
        assert!(extends(
            "Run `widgetly uninstall`.",
            "Run `widgetly uninstall` to remove it."
        ));
        assert!(extends("Then restart it.", "If it stops, then restart it."));
        assert!(extends("Run it now, or later.", "Run it."));
        assert!(!extends("Run it.", "Run the installer and then it."));
        assert!(!extends("", "Anything."));
    }
}
