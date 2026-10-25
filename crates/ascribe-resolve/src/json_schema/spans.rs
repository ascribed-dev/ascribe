//! Where each value of a JSON text is. `serde_json` keeps no positions, so
//! one pass over the text, after `serde_json` has read it, maps each value's
//! JSON pointer to its byte range, and each object member's to its key's.

use std::collections::HashMap;
use std::ops::Range;

/// Where one value is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Place {
    /// The value, from its first byte to just after its last.
    pub value: Range<usize>,
    /// For an object's member, its key, quotes included.
    pub key: Option<Range<usize>>,
}

/// Each value of `text` by its JSON pointer (`""` for the whole text), or
/// `None` when `text` isn't one JSON value.
pub(crate) fn places(text: &str) -> Option<HashMap<String, Place>> {
    let mut walker = Walker {
        bytes: text.as_bytes(),
        at: 0,
        out: HashMap::new(),
    };
    walker.space();
    walker.value(String::new(), None)?;
    walker.space();
    (walker.at == walker.bytes.len()).then_some(walker.out)
}

/// A key as a JSON pointer token: `~0` for `~`, `~1` for `/`.
pub(crate) fn token(key: &str) -> String {
    key.replace('~', "~0").replace('/', "~1")
}

struct Walker<'a> {
    bytes: &'a [u8],
    at: usize,
    out: HashMap<String, Place>,
}

impl Walker<'_> {
    fn space(&mut self) {
        while self
            .bytes
            .get(self.at)
            .is_some_and(|b| matches!(b, b' ' | b'\t' | b'\n' | b'\r'))
        {
            self.at += 1;
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.at).copied()
    }

    fn value(&mut self, pointer: String, key: Option<Range<usize>>) -> Option<()> {
        let start = self.at;
        match self.peek()? {
            b'{' => self.object(&pointer)?,
            b'[' => self.array(&pointer)?,
            b'"' => {
                self.string()?;
            }
            _ => {
                // A number or a literal: `serde_json` has read it already.
                while self.peek().is_some_and(|b| {
                    !matches!(b, b',' | b'}' | b']' | b' ' | b'\t' | b'\n' | b'\r')
                }) {
                    self.at += 1;
                }
                if self.at == start {
                    return None;
                }
            }
        }
        self.out.insert(
            pointer,
            Place {
                value: start..self.at,
                key,
            },
        );
        Some(())
    }

    fn object(&mut self, pointer: &str) -> Option<()> {
        self.at += 1;
        self.space();
        if self.peek() == Some(b'}') {
            self.at += 1;
            return Some(());
        }
        loop {
            self.space();
            let key_start = self.at;
            let name = self.string()?;
            let key = key_start..self.at;
            self.space();
            if self.peek() != Some(b':') {
                return None;
            }
            self.at += 1;
            self.space();
            self.value(format!("{pointer}/{}", token(&name)), Some(key))?;
            self.space();
            match self.peek()? {
                b',' => self.at += 1,
                b'}' => {
                    self.at += 1;
                    return Some(());
                }
                _ => return None,
            }
        }
    }

    fn array(&mut self, pointer: &str) -> Option<()> {
        self.at += 1;
        self.space();
        if self.peek() == Some(b']') {
            self.at += 1;
            return Some(());
        }
        let mut item = 0;
        loop {
            self.space();
            self.value(format!("{pointer}/{item}"), None)?;
            item += 1;
            self.space();
            match self.peek()? {
                b',' => self.at += 1,
                b']' => {
                    self.at += 1;
                    return Some(());
                }
                _ => return None,
            }
        }
    }

    /// A string's decoded text, leaving the cursor after its closing quote.
    fn string(&mut self) -> Option<String> {
        if self.peek() != Some(b'"') {
            return None;
        }
        let start = self.at;
        self.at += 1;
        while self.peek()? != b'"' {
            if self.peek() == Some(b'\\') {
                self.at += 1;
            }
            self.at += 1;
        }
        self.at += 1;
        serde_json::from_slice(self.bytes.get(start..self.at)?).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_value_and_key_is_placed() {
        let text = r#"{ "name": "Ada", "tags": [1, {"a/b": null}] }"#;
        let map = places(text).unwrap();
        assert_eq!(&text[map[""].value.clone()], text);
        assert_eq!(&text[map["/name"].value.clone()], "\"Ada\"");
        assert_eq!(&text[map["/name"].key.clone().unwrap()], "\"name\"");
        assert_eq!(&text[map["/tags/0"].value.clone()], "1");
        assert_eq!(&text[map["/tags/1/a~1b"].value.clone()], "null");
    }

    #[test]
    fn text_after_the_value_is_not_one_value() {
        assert!(places("{} {}").is_none());
        assert!(places("  [1, 2]\n").is_some());
    }
}
