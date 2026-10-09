// Delivering an agent prompt: **Prompt agent** puts the prompt the server
// built where the writer sends it to their agent, and never sends it. The
// targets are the clipboard, VS Code's chat, Claude Code's VS Code extension,
// and Cursor's chat; each fills the prompt in and leaves it unsent. A target
// that isn't there falls back to the clipboard, and says so. No VS Code in
// this module: the host does what VS Code does, so it's unit-tested as is.

/** The commands, as package.json declares them; a test holds the two to the same ids. */
export const PROMPT_COMMANDS = {
  problem: "ascribe.promptAgent",
  file: "ascribe.promptAgentFile",
  project: "ascribe.promptAgentProject",
} as const;

/** Where a prompt goes: the `ascribe.agents.promptTarget` setting's values. */
export const PROMPT_TARGETS = ["clipboard", "chat", "claude-code", "cursor"] as const;
export type PromptTarget = (typeof PROMPT_TARGETS)[number];

/** Each target's name, for messages. */
export const TARGET_NAMES: Record<PromptTarget, string> = {
  clipboard: "the clipboard",
  chat: "VS Code's chat",
  "claude-code": "Claude Code",
  cursor: "Cursor's chat",
};

/** VS Code's command that opens chat; with `isPartialQuery`, the query waits for the writer. */
export const CHAT_COMMAND = "workbench.action.chat.open";
/** Claude Code's VS Code extension, whose URI handler opens a tab with a prompt filled in. */
export const CLAUDE_CODE_EXTENSION = "anthropic.claude-code";
/** The longest link Cursor's deeplinks take, URL-encoded. */
const CURSOR_LINK_LIMIT = 10_000;

/** A link to another program on the machine, as `vscode.Uri.from` takes it. */
export interface PromptLink {
  scheme: string;
  authority: string;
  path: string;
  /** The query as the handler reads it: its values encoded once. */
  query: string;
}

/** What delivering needs from VS Code. */
export interface DeliveryHost {
  /** The scheme this editor's links use: `vscode`, `vscode-insiders`, `cursor`. */
  readonly uriScheme: string;
  copy(text: string): Promise<void>;
  /** Says something briefly, in the status bar. */
  status(message: string): void;
  /** Says something that needs reading, in a notification. */
  notice(message: string): void;
  hasCommand(id: string): Promise<boolean>;
  hasExtension(id: string): boolean;
  executeCommand(id: string, argument: unknown): Promise<unknown>;
  /** Opens a link; resolves to whether something handled it. */
  openLink(link: PromptLink): Promise<boolean>;
}

/** What a delivery did, for tests. */
export interface Delivery {
  /** The target asked for. */
  target: PromptTarget;
  /** Where the prompt went: the target, or the clipboard when it isn't there. */
  delivered: PromptTarget;
  /** What it told the writer. */
  messages: string[];
}

/** Claude Code's link, in an editor whose links use `uriScheme`. */
export function claudeCodeLink(prompt: string, uriScheme: string): PromptLink {
  return {
    scheme: uriScheme,
    authority: CLAUDE_CODE_EXTENSION,
    path: "/open",
    query: `prompt=${encodeURIComponent(prompt)}`,
  };
}

/** Cursor's link. Only its `cursor://` link, never its web one, which would send the prompt to a website. */
export function cursorLink(prompt: string): PromptLink {
  return {
    scheme: "cursor",
    authority: "anysphere.cursor-deeplink",
    path: "/prompt",
    query: `text=${encodeURIComponent(prompt)}`,
  };
}

/** A link's length as it's written. */
const linkLength = (link: PromptLink): number =>
  `${link.scheme}://${link.authority}${link.path}?${link.query}`.length;

/** The targets this editor has: the clipboard always. */
export async function availableTargets(host: DeliveryHost): Promise<PromptTarget[]> {
  const targets: PromptTarget[] = ["clipboard"];
  if (await host.hasCommand(CHAT_COMMAND)) targets.push("chat");
  if (host.hasExtension(CLAUDE_CODE_EXTENSION)) targets.push("claude-code");
  // Cursor's link is handled by Cursor; elsewhere nothing on the machine may.
  if (host.uriScheme === "cursor") targets.push("cursor");
  return targets;
}

/** The setting's value as a target: the clipboard for anything else. */
export const asTarget = (value: unknown): PromptTarget =>
  PROMPT_TARGETS.includes(value as PromptTarget) ? (value as PromptTarget) : "clipboard";

/**
 * Puts `prompt` where `target` says, filled in and not sent. `unsaved` is
 * whether the prompt is about a file with unsaved changes: the agent reads the
 * file on disk, so the writer is told to save it.
 */
export async function deliverPrompt(
  prompt: string,
  target: PromptTarget,
  host: DeliveryHost,
  unsaved = false,
): Promise<Delivery> {
  const delivery: Delivery = { target, delivered: target, messages: [] };
  const status = (message: string): void => {
    delivery.messages.push(message);
    host.status(message);
  };
  const copy = async (why?: string): Promise<void> => {
    await host.copy(prompt);
    delivery.delivered = "clipboard";
    status(
      why
        ? `${why}, so the prompt was copied. Paste it into your agent.`
        : "Prompt copied. Paste it into your agent.",
    );
  };
  const missing = `${TARGET_NAMES[target]} isn't available`;

  switch (target) {
    case "clipboard":
      await copy();
      break;
    case "chat":
      if (await host.hasCommand(CHAT_COMMAND)) {
        await host.executeCommand(CHAT_COMMAND, { query: prompt, isPartialQuery: true });
        status("The prompt is in the chat. Read it, then send it.");
      } else {
        await copy(missing);
      }
      break;
    case "claude-code":
      if (
        host.hasExtension(CLAUDE_CODE_EXTENSION) &&
        (await host.openLink(claudeCodeLink(prompt, host.uriScheme)))
      ) {
        status("The prompt is in Claude Code. Read it, then send it.");
      } else {
        await copy(missing);
      }
      break;
    case "cursor": {
      const link = cursorLink(prompt);
      if (host.uriScheme !== "cursor") {
        await copy(missing);
      } else if (linkLength(link) > CURSOR_LINK_LIMIT) {
        await copy("The prompt is too long for Cursor's link");
      } else if (await host.openLink(link)) {
        status("The prompt is in Cursor's chat. Read it, then send it.");
      } else {
        await copy(missing);
      }
      break;
    }
  }
  if (unsaved) {
    const message = "The file has unsaved changes; save it before your agent starts.";
    delivery.messages.push(message);
    host.notice(message);
  }
  return delivery;
}

/**
 * The targets to offer the first time a prompt is delivered: those this
 * editor has besides the clipboard and the one in use. Empty when there's
 * nothing to choose.
 */
export const targetsToOffer = (
  available: readonly PromptTarget[],
  current: PromptTarget,
): PromptTarget[] => available.filter((t) => t !== "clipboard" && t !== current);

/** What `ascribe/agentPrompt` is asked, from the editor. */
export interface PromptRequest {
  kind: "problem" | "file" | "project" | "pageChanges" | "fragmentReach";
  textDocument?: { uri: string };
  diagnostic?: unknown;
  /** For `pageChanges` and `fragmentReach`: the build whose pages are compared. */
  build?: string;
  /** For `fragmentReach`: the fragment's content path. */
  fragment?: string;
  unsaved: string[];
}

/** A diagnostic's code as the server sent it: a string, or the `value` of `{ value, target }`. */
export function codeOf(code: unknown): string | undefined {
  if (typeof code === "string") return code;
  if (typeof code === "object" && code !== null && "value" in code) {
    const value = (code as { value: unknown }).value;
    return typeof value === "string" ? value : undefined;
  }
  return undefined;
}

/** The title of the code action on a diagnostic, with its code when several are at the cursor. */
export const promptActionTitle = (code: string | undefined, several: boolean): string =>
  several && code ? `Prompt agent: fix this problem (${code})` : "Prompt agent: fix this problem";
