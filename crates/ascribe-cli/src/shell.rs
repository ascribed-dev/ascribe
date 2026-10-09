//! Writing a command a reader can paste into a shell: the `next_command`
//! that `check` and `refs` suggest when they cut a list.

/// A word as a POSIX shell reads it: as it is when it's plain, else in
/// single quotes.
pub fn quote(word: &str) -> String {
    let plain = !word.is_empty()
        && word
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "_-./:@%+=,".contains(c));
    if plain {
        word.to_owned()
    } else {
        format!("'{}'", word.replace('\'', r"'\''"))
    }
}

#[cfg(test)]
mod tests {
    use super::quote;

    #[test]
    fn words_are_quoted_only_when_a_shell_would_split_or_expand_them() {
        assert_eq!(quote("docs/guides/install.md"), "docs/guides/install.md");
        assert_eq!(quote("my guide.md"), "'my guide.md'");
        assert_eq!(quote("it's.md"), r"'it'\''s.md'");
        assert_eq!(quote(""), "''");
    }
}
