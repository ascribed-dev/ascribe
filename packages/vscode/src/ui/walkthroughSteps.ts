/**
 * The commands behind the getting-started walkthrough's buttons that act on
 * a page, each with the command it runs. A button runs from the Welcome page,
 * where no text editor is active, so the command it names would find no page:
 * these bring back the page the writer last had open first. Each has its own
 * id because a walkthrough step completes on a command's id, never on its
 * arguments, and a command this extension runs itself isn't one VS Code
 * counts, so a step lists both its button's command and the one it runs.
 */
export const PAGE_STEPS: Record<string, string> = {
  "ascribe.walkthrough.preview": "ascribe.openPreview",
  "ascribe.walkthrough.actions": "ascribe.actions",
  "ascribe.walkthrough.lens": "ascribe.toggleBuildLens",
};
