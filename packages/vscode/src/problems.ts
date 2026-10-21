// Problems counted by severity, in words: what the Project view's and This
// page's icons mean, said in their tooltips and to a screen reader, and what
// Check Every Build's notification says. No VS Code in this module.

import type { ProblemCounts } from "./webview/viewProtocol.js";

/**
 * "2 errors, 1 warning, and advice on 3 lines", leaving out a severity
 * there's none of; empty for none at all. Advice is a mass noun, so it's
 * counted by the lines it's on.
 */
export function problemsText({ errors, warnings, advice }: ProblemCounts): string {
  const parts = [
    errors > 0 ? `${errors} ${errors === 1 ? "error" : "errors"}` : "",
    warnings > 0 ? `${warnings} ${warnings === 1 ? "warning" : "warnings"}` : "",
    advice > 0 ? `advice on ${advice} ${advice === 1 ? "line" : "lines"}` : "",
  ].filter(Boolean);
  if (parts.length < 3) return parts.join(" and ");
  return `${parts.slice(0, -1).join(", ")}, and ${parts.at(-1)}`;
}
