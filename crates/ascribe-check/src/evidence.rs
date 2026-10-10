//! What an agent prompt about a content check carries beyond the message:
//! the evidence its registry entry names, gathered when the prompt is made,
//! since only a prompt needs it.

use ascribe_core::RelPath;

use crate::{Diagnostic, Project};

/// The pages that mention a page's title or share its folder, for
/// `page-orphan`.
pub const MENTIONS: &str = "mentions";

/// An image's size, and its width and height when its header says them, for
/// `image-large`.
pub const IMAGE_SIZE: &str = "image-size";

/// The descriptions of the pages a diagnostic names, for `title-duplicate`.
pub const DESCRIPTIONS: &str = "descriptions";

/// The evidence gathered when a prompt is made, by name, besides what an
/// issue carries in its arguments ([`EVIDENCE`](crate::EVIDENCE)).
pub const GATHERED: &[&str] = &[MENTIONS, IMAGE_SIZE, DESCRIPTIONS];

/// The most pages a line of evidence lists.
const MAX_PAGES: usize = 10;

/// The lines of evidence `name` gives about `d`, for its prompt.
pub(crate) fn lines(project: &Project, d: &Diagnostic, name: &str) -> Vec<String> {
    match name {
        MENTIONS => mentions(project, d),
        IMAGE_SIZE => image_size(project, d),
        DESCRIPTIONS => descriptions(project, d),
        _ => Vec::new(),
    }
}

/// The pages whose text mentions the page's title, and the pages in its
/// folder: where a link to it might belong.
fn mentions(project: &Project, d: &Diagnostic) -> Vec<String> {
    let Some(page) = project.file(d.location.file).and_then(|f| f.content_path) else {
        return Vec::new();
    };
    let title = project
        .source_at(page)
        .and_then(|s| field(&s.text, "title"))
        .filter(|t| !t.trim().is_empty());
    let folder = page.parent();
    let mut mentioning = Vec::new();
    let mut beside = Vec::new();
    for source in project.sources() {
        if &source.path == page || source.unreadable.is_some() {
            continue;
        }
        if let Some(title) = &title
            && contains_ignoring_case(&source.text, title)
        {
            mentioning.push(shown(project, &source.path));
        }
        if source.path.parent() == folder {
            beside.push(shown(project, &source.path));
        }
    }
    let mut out = Vec::new();
    if let Some(title) = &title
        && !mentioning.is_empty()
    {
        out.push(format!(
            "Files that mention \"{title}\": {}",
            listed(&mentioning)
        ));
    }
    if !beside.is_empty() {
        out.push(format!("Files in the same folder: {}", listed(&beside)));
    }
    out
}

/// The image's size, and its dimensions when its header says them.
fn image_size(project: &Project, d: &Diagnostic) -> Vec<String> {
    let Some(entry) = project.file(d.location.file) else {
        return Vec::new();
    };
    let Ok(path) = RelPath::parse(&entry.display_path) else {
        return Vec::new();
    };
    let fs = project.file_system();
    let Some(size) = fs.size(&path) else {
        return Vec::new();
    };
    let mut line = format!("Size: {} ({size} bytes)", crate::size_text(size));
    if let Ok(bytes) = fs.read_file(&path)
        && let Some((width, height)) = dimensions(&bytes)
    {
        line.push_str(&format!("; {width} by {height} pixels"));
    }
    vec![line]
}

/// The description of the page a diagnostic is in and of each page its
/// related places are in.
fn descriptions(project: &Project, d: &Diagnostic) -> Vec<String> {
    let mut files = vec![d.location.file];
    files.extend(d.related.iter().map(|r| r.location.file));
    let mut out = Vec::new();
    for file in files {
        let Some(entry) = project.file(file) else {
            continue;
        };
        let description = field(entry.text, "description")
            .filter(|t| !t.trim().is_empty())
            .unwrap_or_else(|| "(none)".to_owned());
        let line = format!("Description of {}: {description}", entry.display_path);
        if !out.contains(&line) {
            out.push(line);
        }
    }
    out
}

/// A page's path as a prompt shows it, from the project root.
fn shown(project: &Project, path: &RelPath) -> String {
    format!("`{}`", project.layout().project_path(path))
}

/// Up to [`MAX_PAGES`] items, and how many more there are.
fn listed(items: &[String]) -> String {
    let mut text = items
        .iter()
        .take(MAX_PAGES)
        .cloned()
        .collect::<Vec<_>>()
        .join(", ");
    if items.len() > MAX_PAGES {
        text.push_str(&format!(", and {} more", items.len() - MAX_PAGES));
    }
    text
}

fn contains_ignoring_case(text: &str, needle: &str) -> bool {
    text.to_lowercase().contains(&needle.to_lowercase())
}

/// A string field of a file's frontmatter, when it has one.
fn field(text: &str, key: &str) -> Option<String> {
    let body = text
        .strip_prefix("---\n")
        .or_else(|| text.strip_prefix("---\r\n"))?;
    let end = body
        .match_indices("\n---")
        .map(|(at, _)| at + 1)
        .find(|&at| body[at + 3..].starts_with(['\n', '\r']) || body.len() == at + 3)
        .or_else(|| body.starts_with("---").then_some(0))?;
    let value: serde_yaml_ng::Value = serde_yaml_ng::from_str(&body[..end]).ok()?;
    value.get(key)?.as_str().map(str::to_owned)
}

/// An image's width and height, from a PNG, GIF, JPEG, or WebP header.
pub(crate) fn dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    let be16 = |at: usize| -> Option<u32> {
        Some(u32::from(u16::from_be_bytes([
            *bytes.get(at)?,
            *bytes.get(at + 1)?,
        ])))
    };
    let le16 = |at: usize| -> Option<u32> {
        Some(u32::from(u16::from_le_bytes([
            *bytes.get(at)?,
            *bytes.get(at + 1)?,
        ])))
    };
    let be32 = |at: usize| -> Option<u32> {
        Some(u32::from_be_bytes(bytes.get(at..at + 4)?.try_into().ok()?))
    };
    let le24 = |at: usize| -> Option<u32> {
        Some(
            u32::from(*bytes.get(at)?)
                | u32::from(*bytes.get(at + 1)?) << 8
                | u32::from(*bytes.get(at + 2)?) << 16,
        )
    };
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Some((be32(16)?, be32(20)?));
    }
    if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        return Some((le16(6)?, le16(8)?));
    }
    if bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WEBP") {
        return match bytes.get(12..16)? {
            b"VP8X" => Some((le24(24)? + 1, le24(27)? + 1)),
            b"VP8 " => Some((le16(26)? & 0x3fff, le16(28)? & 0x3fff)),
            b"VP8L" => {
                let b = |i: usize| bytes.get(21 + i).map(|v| u32::from(*v));
                let width = 1 + (b(0)? | (b(1)? & 0x3f) << 8);
                let height = 1 + (b(1)? >> 6 | b(2)? << 2 | (b(3)? & 0x0f) << 10);
                Some((width, height))
            }
            _ => None,
        };
    }
    if bytes.starts_with(&[0xff, 0xd8]) {
        // Walk the segments to the first start-of-frame marker.
        let mut at = 2;
        while at + 9 < bytes.len() {
            if bytes[at] != 0xff {
                return None;
            }
            let marker = bytes[at + 1];
            let length = be16(at + 2)? as usize;
            let frame = matches!(marker, 0xc0..=0xcf) && !matches!(marker, 0xc4 | 0xc8 | 0xcc);
            if frame {
                return Some((be16(at + 7)?, be16(at + 5)?));
            }
            at += 2 + length;
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dimensions_come_from_the_header() {
        let mut png = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR".to_vec();
        png.extend(640u32.to_be_bytes());
        png.extend(480u32.to_be_bytes());
        assert_eq!(dimensions(&png), Some((640, 480)));
        let gif = b"GIF89a\x20\x00\x10\x00";
        assert_eq!(dimensions(gif), Some((32, 16)));
        let jpeg = [
            0xff, 0xd8, 0xff, 0xe0, 0x00, 0x04, 0x00, 0x00, 0xff, 0xc0, 0x00, 0x11, 0x08, 0x00,
            0x64, 0x00, 0xc8, 0x03,
        ];
        assert_eq!(dimensions(&jpeg), Some((200, 100)));
        assert_eq!(dimensions(b"not an image"), None);
    }

    #[test]
    fn a_frontmatter_field_is_read() {
        let text = "---\ntitle: Install\ndescription: How to install it.\n---\n\nText.\n";
        assert_eq!(field(text, "title").as_deref(), Some("Install"));
        assert_eq!(
            field(text, "description").as_deref(),
            Some("How to install it.")
        );
        assert_eq!(field("No frontmatter.\n", "title"), None);
    }
}
