// The overlay's stylesheet. It applies inside the overlay's shadow roots only,
// so the page's styles don't reach the overlay and the overlay's don't reach
// the page. Colors come from the page's --ascribe-review-* custom properties
// (`marks.css` sets them, with light and dark values; a host can set its own),
// which reach inside shadow roots by inheritance.

export const OVERLAY_CSS = `
:host {
  all: initial;
  --text: var(--ascribe-review-text, CanvasText);
  --muted: var(--ascribe-review-muted, #5b6475);
  --line: var(--ascribe-review-border, #d3d8e0);
  --surface: var(--ascribe-review-surface, Canvas);
  --thread: var(--ascribe-review-thread, #f5f7fb);
  --accent: var(--ascribe-review-focus, #1f6feb);
  --accent-text: var(--ascribe-review-accent-text, #ffffff);
  --input: var(--ascribe-review-input, var(--surface));
  --flash: var(--ascribe-review-flash, #fff6bf);
  --added: var(--ascribe-review-added, #1a7f37);
  --changed: var(--ascribe-review-changed, #8a5a00);
  --changed-background: var(--ascribe-review-changed-background, #fff1c2);
  --removed: var(--ascribe-review-removed, #c4222d);
  --removed-background: var(--ascribe-review-removed-background, #ffe4e2);
  --mono: var(--ascribe-review-mono, ui-monospace, monospace);
  font-family: var(--ascribe-review-font, system-ui, -apple-system, "Segoe UI", sans-serif);
  font-size: 12.5px;
  line-height: 1.45;
  color: var(--text);
}
* {
  box-sizing: border-box;
}
[hidden] {
  display: none !important;
}
button,
textarea,
input {
  font: inherit;
  color: inherit;
}
button {
  cursor: pointer;
}
button:disabled,
button[aria-disabled="true"] {
  cursor: not-allowed;
  opacity: 0.5;
}
button:focus-visible,
textarea:focus-visible,
input:focus-visible,
.thread:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: 1px;
}
.sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  overflow: hidden;
  clip-path: inset(50%);
  white-space: nowrap;
}

/* Buttons */
.ghost {
  border: 1px solid var(--line);
  background: var(--surface);
  border-radius: 5px;
  padding: 2px 8px;
  font-size: 12px;
  min-height: 24px;
}
.ghost:hover:not(:disabled):not([aria-disabled="true"]) {
  border-color: var(--accent);
  color: var(--accent);
}
.primary {
  border: 1px solid var(--accent);
  background: var(--accent);
  color: var(--accent-text);
  border-radius: 5px;
  padding: 2px 10px;
  font-size: 12.5px;
  font-weight: 600;
  min-height: 24px;
}
.danger {
  border: 1px solid var(--removed);
  background: var(--removed);
  color: var(--surface);
  border-radius: 5px;
  padding: 2px 10px;
  font-size: 12.5px;
  font-weight: 600;
  min-height: 24px;
}
.link {
  border: 0;
  background: transparent;
  color: var(--accent);
  padding: 0;
  font-size: inherit;
  text-align: inherit;
}
a.link {
  text-decoration: none;
}
.link:hover {
  text-decoration: underline;
}
.hint {
  color: var(--muted);
  font-size: 11.5px;
}
.refuse,
.error {
  color: var(--removed);
  font-size: 12px;
}
.spacer {
  flex: 1;
}

/* The layer over the page: the column, markers' panel, dialogs. */
.layer {
  position: absolute;
  inset: 0;
  pointer-events: none;
}
.layer > *,
.column > * {
  pointer-events: auto;
}
.column {
  position: absolute;
  top: 0;
  width: 250px;
  pointer-events: none;
}
.slot {
  position: absolute;
  left: 0;
  right: 0;
}
.slot > .thread,
.slot > .composer {
  margin: 0 0 6px;
}
.slot.linked .thread,
.slot.linked .composer {
  border-color: var(--accent);
}
.highlight {
  position: absolute;
  pointer-events: none;
  background: color-mix(in srgb, var(--flash) 55%, transparent);
  border-radius: 3px;
  /* Over the page's text, a tint that keeps the text as it was: multiplied
     on a light page, screened on a dark one. */
  mix-blend-mode: var(--ascribe-review-blend, multiply);
}
svg.connector {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  overflow: visible;
  pointer-events: none;
}
svg.connector path {
  fill: none;
  stroke: var(--accent);
  stroke-width: 1.5;
}

/* The block tools: the Comment button, and the count marker at narrow widths. */
.tool {
  border: 1px solid var(--line);
  background: var(--surface);
  border-radius: 5px;
  font-size: 11.5px;
  padding: 0 7px;
  min-height: 22px;
  color: var(--accent);
  white-space: nowrap;
}
.marker {
  border: 1px solid var(--accent);
  background: var(--surface);
  color: var(--accent);
  border-radius: 999px;
  font-size: 11px;
  font-weight: 700;
  padding: 0 8px;
  min-width: 30px;
  min-height: 24px;
}

/* Threads */
.thread {
  background: var(--thread);
  border: 1px solid var(--line);
  border-radius: 6px;
  margin: 6px 0 10px;
}
.th {
  display: flex;
  flex-wrap: wrap;
  gap: 3px 8px;
  align-items: center;
  padding: 5px 9px 2px;
  font-size: 11.5px;
  color: var(--muted);
}
.acts {
  display: flex;
  flex-wrap: wrap;
  gap: 2px 12px;
  padding: 0 9px 5px;
  border-bottom: 1px solid var(--line);
  font-size: 11.5px;
}
.thread.collapsed .acts {
  border-bottom: 0;
}
.orig,
.inside {
  padding: 5px 9px;
  border-bottom: 1px solid var(--line);
  font-size: 11.5px;
  color: var(--muted);
}
.orig q {
  color: var(--text);
  white-space: pre-wrap;
}
.badge {
  border-radius: 999px;
  padding: 0 7px;
  font-size: 10.5px;
  font-weight: 600;
  border: 1px solid currentColor;
  white-space: nowrap;
}
.badge.unsent {
  color: var(--changed);
}
.badge.resolved {
  color: var(--added);
}
.badge.outdated {
  color: var(--muted);
}
.badge.summary {
  color: var(--accent);
}
.badge.removed,
.badge.detached {
  color: var(--removed);
}
.comment {
  display: grid;
  grid-template-columns: 22px minmax(0, 1fr);
  gap: 7px;
  padding: 6px 9px;
}
.comment + .comment {
  border-top: 1px dashed var(--line);
}
.avatar {
  width: 22px;
  height: 22px;
  border-radius: 50%;
  background: var(--accent);
  color: var(--accent-text);
  display: grid;
  place-items: center;
  font-size: 9.5px;
  font-weight: 700;
}
.who {
  font-weight: 600;
}
.when {
  color: var(--muted);
  font-size: 11px;
  margin-left: 6px;
}
.body {
  overflow-wrap: anywhere;
}
.body > :first-child {
  margin-top: 1px;
}
.body > :last-child {
  margin-bottom: 0;
}
.body p,
.body ul,
.body ol,
.body blockquote,
.body pre {
  margin: 4px 0;
}
.body ul,
.body ol {
  padding-left: 18px;
}
.body blockquote {
  border-left: 3px solid var(--line);
  padding-left: 8px;
  color: var(--muted);
}
.body code {
  font-family: var(--mono);
  font-size: 0.92em;
  background: color-mix(in srgb, var(--muted) 14%, transparent);
  padding: 0.05em 0.3em;
  border-radius: 3px;
}
.body pre {
  overflow-x: auto;
  padding: 6px 8px;
  border-radius: 4px;
  background: color-mix(in srgb, var(--muted) 14%, transparent);
}
.body pre code {
  background: none;
  padding: 0;
}
.body a {
  color: var(--accent);
}
.reply {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  padding: 6px 9px 8px;
  border-top: 1px solid var(--line);
}
.reply textarea {
  flex: 1 1 100%;
  min-width: 80px;
  padding: 2px 8px;
  border: 1px solid var(--line);
  border-radius: 5px;
  background: var(--input);
  resize: vertical;
  field-sizing: content;
  min-height: 26px;
  max-height: 160px;
}
.reply .error,
.reply .hint {
  flex-basis: 100%;
}
.composer {
  background: var(--thread);
  border: 1px solid var(--accent);
  border-radius: 6px;
  padding: 8px 10px;
  margin: 6px 0 10px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.composer textarea {
  width: 100%;
  min-height: 56px;
  resize: vertical;
  padding: 6px 8px;
  border: 1px solid var(--line);
  border-radius: 5px;
  background: var(--input);
}
.row {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
  align-items: center;
}

/* The detached list, above the page */
.detached {
  margin: 12px 0;
  border: 1px dashed var(--line);
  border-radius: 6px;
  padding: 8px 10px 2px;
  color: var(--muted);
  max-width: 520px;
}
.detached .thread {
  color: var(--text);
}

/* The unsent bar, at the bottom */
.unsent-bar {
  background: var(--changed-background);
  border-top: 1px solid var(--line);
  padding: 6px 10px;
  display: flex;
  flex-wrap: wrap;
  gap: 6px 10px;
  align-items: center;
}
.unsent-bar .hint {
  font-size: 12px;
}

/* The panel for one block's threads, at narrow widths */
.sheet {
  position: fixed;
  left: 0;
  right: 0;
  bottom: 0;
  max-height: 62vh;
  overflow: auto;
  background: var(--surface);
  border-top: 2px solid var(--accent);
  box-shadow: 0 -8px 24px rgba(0, 0, 0, 0.22);
  padding: 8px 12px 10px;
  z-index: 4;
}
.sheet .head {
  display: flex;
  gap: 8px;
  align-items: center;
  font-size: 12px;
  color: var(--muted);
}

/* Dialogs */
.backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.4);
  z-index: 8;
  display: grid;
  place-items: center;
  padding: 12px;
}
.dialog {
  background: var(--surface);
  border: 1px solid var(--line);
  border-radius: 8px;
  width: min(480px, 100%);
  max-height: 100%;
  overflow: auto;
  padding: 12px 14px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  font-size: 13px;
}
.dialog h2 {
  margin: 0;
  font-size: 14px;
}
.dialog ul.unsent-list {
  margin: 0;
  padding-left: 18px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.dialog fieldset {
  border: 0;
  padding: 0;
  margin: 0;
  display: flex;
  flex-wrap: wrap;
  gap: 4px 14px;
}
.dialog textarea {
  width: 100%;
  min-height: 56px;
  resize: vertical;
  padding: 6px 8px;
  border: 1px solid var(--line);
  border-radius: 5px;
  background: var(--input);
}
.confirm {
  border: 1px solid var(--removed);
  background: var(--removed-background);
  border-radius: 6px;
  padding: 8px 10px;
}
.segmented {
  display: inline-flex;
  flex-wrap: wrap;
  align-self: flex-start;
  border: 1px solid var(--line);
  border-radius: 5px;
  overflow: hidden;
}
.segmented button {
  border: 0;
  background: transparent;
  padding: 2px 9px;
  font-size: 12px;
  color: var(--muted);
  min-height: 24px;
}
.segmented button + button {
  border-left: 1px solid var(--line);
}
.segmented button[aria-pressed="true"] {
  background: var(--accent);
  color: var(--accent-text);
}
.thread-list {
  list-style: none;
  margin: 0;
  padding: 0;
}
.thread-list li + li {
  border-top: 1px solid var(--line);
}
.thread-list button {
  display: flex;
  flex-direction: column;
  gap: 1px;
  width: 100%;
  text-align: left;
  border: 0;
  background: transparent;
  padding: 6px 4px;
}
.thread-list button:hover,
.thread-list button:focus-visible {
  background: color-mix(in srgb, var(--muted) 12%, transparent);
}
.thread-list .where {
  font-size: 11.5px;
  color: var(--muted);
}
.thread-list .first {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.flash {
  animation: flash 1.4s ease-out;
}
@keyframes flash {
  from {
    box-shadow: 0 0 0 3px var(--accent);
  }
  to {
    box-shadow: 0 0 0 0 transparent;
  }
}
@media (prefers-reduced-motion: reduce) {
  .flash {
    animation: none;
    outline: 2px solid var(--accent);
  }
}
`;
