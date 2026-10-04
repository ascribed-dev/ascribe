// The toolbar app's panel, in the app's own shadow root: the page's styles
// don't reach it, and its don't reach the page.
export const PANEL_CSS = `
:host {
  --ink: #1d2330;
  --muted: #5b6475;
  --win: #ffffff;
  --side: #f5f7fb;
  --line: #d3d8e0;
  --accent: #1f6feb;
  --accent-ink: #ffffff;
  --notice: #fff1c2;
  --ui: system-ui, -apple-system, "Segoe UI", Roboto, sans-serif;
  --mono: ui-monospace, "SF Mono", Menlo, Consolas, monospace;
}
@media (prefers-color-scheme: dark) {
  :host {
    --ink: #d9dde5;
    --muted: #929bab;
    --win: #1e2026;
    --side: #23262d;
    --line: #343a45;
    --accent: #5aa2ff;
    --accent-ink: #0b1220;
    --notice: #3a2f0d;
  }
}
.panel {
  position: fixed;
  left: 50%;
  bottom: 72px;
  transform: translateX(-50%);
  width: min(960px, calc(100vw - 20px));
  box-sizing: border-box;
  max-height: 300px;
  overflow-y: auto;
  background: var(--win);
  color: var(--ink);
  border: 1px solid var(--ink);
  border-radius: 8px;
  box-shadow: 0 8px 28px rgba(0, 0, 0, 0.3);
  font: 12.5px/1.45 var(--ui);
  z-index: 1;
}
.ttl {
  display: flex;
  gap: 10px;
  align-items: center;
  padding: 4px 10px;
  font-size: 11px;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  font-weight: 700;
  background: var(--ink);
  color: var(--win);
  position: sticky;
  top: 0;
}
.ttl .grow {
  flex: 1;
}
.ttl button {
  border: 0;
  background: transparent;
  color: var(--win);
  padding: 0;
  font: inherit;
  text-transform: none;
  letter-spacing: 0;
  font-weight: 400;
  cursor: pointer;
}
.row,
.notice,
.unsent {
  display: flex;
  flex-wrap: wrap;
  gap: 6px 10px;
  align-items: center;
  padding: 6px 10px;
  border-bottom: 1px solid var(--line);
  font-size: 12px;
}
.row {
  background: var(--side);
}
.row .grow {
  flex: 1;
  min-width: 160px;
  color: var(--muted);
}
.row b {
  color: var(--ink);
  font-weight: 600;
}
.legend {
  flex-basis: 100%;
  color: var(--muted);
  font-size: 11.5px;
}
.notice,
.unsent {
  background: var(--notice);
  font-size: 12.5px;
}
.notice code {
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 0.95em;
}
.spacer {
  flex: 1;
}
.body {
  padding: 8px 10px;
  display: flex;
  flex-direction: column;
  gap: 6px;
  border-bottom: 1px solid var(--line);
}
.body:last-child,
.row:last-child,
.notice:last-child,
.unsent:last-child {
  border-bottom: 0;
}
.hint {
  color: var(--muted);
  font-size: 11.5px;
}
ul {
  margin: 0;
  padding-left: 18px;
}
li + li {
  margin-top: 2px;
}
.thread {
  border: 1px solid var(--line);
  border-radius: 6px;
  padding: 6px 8px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.thread .where {
  color: var(--muted);
  font-size: 11.5px;
}
.thread .comment p {
  margin: 0;
}
code {
  font-family: var(--mono);
  font-size: 0.92em;
}
button {
  font: inherit;
  cursor: pointer;
  color: inherit;
}
button:disabled {
  cursor: not-allowed;
  opacity: 0.5;
}
button:focus-visible,
a:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: 1px;
}
.ghost {
  border: 1px solid var(--line);
  background: var(--win);
  border-radius: 5px;
  padding: 2px 8px;
  font-size: 12px;
}
.ghost:hover {
  border-color: var(--accent);
  color: var(--accent);
}
.square {
  min-width: 28px;
  min-height: 24px;
}
.primary {
  border: 1px solid var(--accent);
  background: var(--accent);
  color: var(--accent-ink);
  border-radius: 5px;
  padding: 2px 10px;
  font-size: 12.5px;
  font-weight: 600;
}
.link,
a {
  border: 0;
  background: transparent;
  color: var(--accent);
  padding: 0;
  font-size: inherit;
  text-decoration: none;
}
.link:hover,
a:hover {
  text-decoration: underline;
}
.position {
  white-space: nowrap;
}
.segmented {
  display: inline-flex;
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
}
.segmented button + button {
  border-left: 1px solid var(--line);
}
.segmented button[aria-pressed="true"] {
  background: var(--accent);
  color: var(--accent-ink);
}
.sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  overflow: hidden;
  clip: rect(0 0 0 0);
  white-space: nowrap;
}
`;
