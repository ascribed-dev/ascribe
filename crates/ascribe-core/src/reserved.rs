//! Attribute keys a content model can't declare (SPEC §7.2).
//!
//! The site output writes image attributes onto `<img>` elements and project
//! widgets' attributes onto custom elements (SPEC §9.4), so a key HTML
//! already gives a meaning there would change the element's behavior or
//! clash with what Ascribe writes. The content-model loader rejects them with
//! `model-attribute-reserved`, using [`is_reserved_image_attribute`] and [`is_reserved_widget_attribute`].
//!
//! The lists are explicit rather than patterns, so that ordinary keys such as
//! `online`, `onboarding`, or `only-if` stay available. Only `aria-` is a
//! prefix, because every key it starts is an ARIA attribute.

/// Keys an image can't declare: CommonMark's image syntax already supplies them.
pub const IMAGE_KEYS: &[&str] = &["src", "alt", "title"];

/// Keys a widget can't declare: the site output uses them on the widget's
/// element for its title line and its identifier primary (element contract §6).
pub const WIDGET_KEYS: &[&str] = &["heading", "primary"];

/// The prefix of ARIA attributes, which no image or widget key may start with.
pub const ARIA_PREFIX: &str = "aria-";

/// HTML's global attributes (HTML Living Standard, "Global attributes"),
/// other than event handlers, which are in [`HTML_EVENT_HANDLER_ATTRIBUTES`].
pub const HTML_GLOBAL_ATTRIBUTES: &[&str] = &[
    "accesskey",
    "autocapitalize",
    "autocorrect",
    "autofocus",
    "class",
    "contenteditable",
    "dir",
    "draggable",
    "enterkeyhint",
    "hidden",
    "id",
    "inert",
    "inputmode",
    "is",
    "itemid",
    "itemprop",
    "itemref",
    "itemscope",
    "itemtype",
    "lang",
    "nonce",
    "part",
    "popover",
    "role",
    "slot",
    "spellcheck",
    "style",
    "tabindex",
    "title",
    "translate",
    "writingsuggestions",
];

/// Event handler content attributes: those the HTML Living Standard defines
/// (on every element, and on `<body>` for the window), plus the ones other
/// web platform specifications add to every element (Pointer Events, Touch
/// Events, CSS Animations and Transitions, and Selection), since browsers
/// run all of them. Sorted.
pub const HTML_EVENT_HANDLER_ATTRIBUTES: &[&str] = &[
    "onabort",
    "onafterprint",
    "onanimationcancel",
    "onanimationend",
    "onanimationiteration",
    "onanimationstart",
    "onauxclick",
    "onbeforeinput",
    "onbeforematch",
    "onbeforeprint",
    "onbeforetoggle",
    "onbeforeunload",
    "onblur",
    "oncancel",
    "oncanplay",
    "oncanplaythrough",
    "onchange",
    "onclick",
    "onclose",
    "oncommand",
    "oncontextlost",
    "oncontextmenu",
    "oncontextrestored",
    "oncopy",
    "oncuechange",
    "oncut",
    "ondblclick",
    "ondrag",
    "ondragend",
    "ondragenter",
    "ondragleave",
    "ondragover",
    "ondragstart",
    "ondrop",
    "ondurationchange",
    "onemptied",
    "onended",
    "onerror",
    "onfocus",
    "onformdata",
    "ongotpointercapture",
    "onhashchange",
    "oninput",
    "oninvalid",
    "onkeydown",
    "onkeypress",
    "onkeyup",
    "onlanguagechange",
    "onload",
    "onloadeddata",
    "onloadedmetadata",
    "onloadstart",
    "onlostpointercapture",
    "onmessage",
    "onmessageerror",
    "onmousedown",
    "onmouseenter",
    "onmouseleave",
    "onmousemove",
    "onmouseout",
    "onmouseover",
    "onmouseup",
    "onoffline",
    "ononline",
    "onpagehide",
    "onpagereveal",
    "onpageshow",
    "onpageswap",
    "onpaste",
    "onpause",
    "onplay",
    "onplaying",
    "onpointercancel",
    "onpointerdown",
    "onpointerenter",
    "onpointerleave",
    "onpointermove",
    "onpointerout",
    "onpointerover",
    "onpointerrawupdate",
    "onpointerup",
    "onpopstate",
    "onprogress",
    "onratechange",
    "onrejectionhandled",
    "onreset",
    "onresize",
    "onscroll",
    "onscrollend",
    "onsecuritypolicyviolation",
    "onseeked",
    "onseeking",
    "onselect",
    "onselectionchange",
    "onselectstart",
    "onslotchange",
    "onstalled",
    "onstorage",
    "onsubmit",
    "onsuspend",
    "ontimeupdate",
    "ontoggle",
    "ontouchcancel",
    "ontouchend",
    "ontouchmove",
    "ontouchstart",
    "ontransitioncancel",
    "ontransitionend",
    "ontransitionrun",
    "ontransitionstart",
    "onunhandledrejection",
    "onunload",
    "onvolumechange",
    "onwaiting",
    "onwebkitanimationend",
    "onwebkitanimationiteration",
    "onwebkitanimationstart",
    "onwebkittransitionend",
    "onwheel",
];

fn reserved_on_every_element(key: &str) -> bool {
    key.starts_with(ARIA_PREFIX)
        || HTML_GLOBAL_ATTRIBUTES.contains(&key)
        || HTML_EVENT_HANDLER_ATTRIBUTES.contains(&key)
}

/// Whether `[images.attributes]` can't declare `key`.
pub fn is_reserved_image_attribute(key: &str) -> bool {
    IMAGE_KEYS.contains(&key) || reserved_on_every_element(key)
}

/// Whether a widget's `attributes` can't declare `key`.
pub fn is_reserved_widget_attribute(key: &str) -> bool {
    WIDGET_KEYS.contains(&key) || reserved_on_every_element(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_that_only_start_with_on_are_allowed() {
        for key in ["online", "only-if", "onboarding", "one", "on"] {
            assert!(!is_reserved_image_attribute(key), "{key}");
            assert!(!is_reserved_widget_attribute(key), "{key}");
        }
    }

    #[test]
    fn event_handlers_are_rejected() {
        for key in [
            "onclick",
            "onload",
            "onerror",
            "onpointerdown",
            "ontransitionend",
        ] {
            assert!(is_reserved_image_attribute(key), "{key}");
            assert!(is_reserved_widget_attribute(key), "{key}");
        }
    }

    #[test]
    fn each_element_has_its_own_reserved_keys() {
        assert!(is_reserved_image_attribute("src"));
        assert!(!is_reserved_widget_attribute("src"));
        assert!(is_reserved_widget_attribute("heading"));
        assert!(!is_reserved_image_attribute("heading"));
        for key in ["title", "style", "id", "class", "aria-label"] {
            assert!(
                is_reserved_image_attribute(key) && is_reserved_widget_attribute(key),
                "{key}"
            );
        }
        for key in ["width", "loading", "lab", "height"] {
            assert!(
                !is_reserved_image_attribute(key) && !is_reserved_widget_attribute(key),
                "{key}"
            );
        }
    }

    #[test]
    fn lists_are_sorted_lowercase_keys() {
        for list in [HTML_GLOBAL_ATTRIBUTES, HTML_EVENT_HANDLER_ATTRIBUTES] {
            assert!(
                list.windows(2).all(|w| w[0] < w[1]),
                "sorted, no duplicates"
            );
            assert!(
                list.iter()
                    .all(|k| k.chars().all(|c| c.is_ascii_lowercase()))
            );
        }
        assert!(
            HTML_EVENT_HANDLER_ATTRIBUTES
                .iter()
                .all(|k| k.starts_with("on"))
        );
    }
}
