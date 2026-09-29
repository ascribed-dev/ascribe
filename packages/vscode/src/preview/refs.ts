// Asset references, compared the way the webview and the extension both do.
//
// A reference in the page's HTML is a relative path (`./img/a.png`, an image)
// or a root-relative URL (`/docs/_ascribe/files/a.yaml`, a link target),
// percent-encoded as a URL is. The server sends each asset's reference as
// the emitter wrote it, and comrak escapes it again when it writes the
// `src`, so the two spellings may differ (`%20` for a space, say). Both sides
// resolve a reference against one dummy base, which normalizes the
// encoding, and compare the resulting paths.

const BASE = "http://ascribe.invalid/preview/";

/** Splits a reference at its first `#`. */
export function splitFragment(reference: string): { path: string; fragment: string } {
  const at = reference.indexOf("#");
  return at < 0
    ? { path: reference, fragment: "" }
    : { path: reference.slice(0, at), fragment: reference.slice(at) };
}

/** A reference without its fragment, in a form that is equal for equal references. */
export function canonicalReference(reference: string): string {
  const { path } = splitFragment(reference);
  try {
    const url = new URL(path, BASE);
    return `${url.pathname}${url.search}`;
  } catch {
    return path;
  }
}

/** Whether a destination has a URL scheme (`https:`, `mailto:`, …) or starts with `//`. */
export function isExternal(href: string): boolean {
  return /^[a-z][a-z0-9+.-]*:/i.test(href) || href.startsWith("//");
}
