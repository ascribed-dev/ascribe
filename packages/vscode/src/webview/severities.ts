// Problems as VS Code's status bar shows them: an icon per severity, the one
// the Problems panel uses, with its count beside it, leaving out a severity
// there's none of. Each is told by its shape and its count, not by color
// alone. The icons mean nothing to a screen reader: the words the extension
// sends (`../problems.ts`) are their tooltip and accessible name.

import type { ProblemCounts } from "./viewProtocol.js";

/** Each severity's codicon, as the Problems panel draws it: advice is Information. */
const ICONS = [
  ["errors", "error"],
  ["warnings", "warning"],
  ["advice", "info"],
] as const satisfies readonly (readonly [keyof ProblemCounts, string])[];

/** The icons and counts for `counts`, named by `words`, which say them. */
export function severities(counts: ProblemCounts, words: string): HTMLElement {
  const made = document.createElement("span");
  made.className = "severities";
  made.title = words;
  made.setAttribute("role", "img");
  made.setAttribute("aria-label", words);
  for (const [key, name] of ICONS) {
    if (counts[key] === 0) continue;
    const severity = document.createElement("span");
    severity.className = `severity severity-${name}`;
    const icon = document.createElement("span");
    icon.className = `codicon codicon-${name}`;
    icon.setAttribute("aria-hidden", "true");
    const count = document.createElement("span");
    count.className = "severity-count";
    count.textContent = String(counts[key]);
    severity.append(icon, count);
    made.append(severity);
  }
  return made;
}

/** The icons drawn under `root`, each as its codicon and count, for a report: "warning 1". */
export function drawnSeverities(root: ParentNode): string[] {
  return [...root.querySelectorAll(".severity")].map((severity) => {
    const icon = [...(severity.querySelector(".codicon")?.classList ?? [])].find((name) =>
      name.startsWith("codicon-"),
    );
    return `${icon?.slice("codicon-".length) ?? ""} ${severity.textContent ?? ""}`;
  });
}
