import { describe, expect, it } from "vitest";
import {
  CHAT_COMMAND,
  CLAUDE_CODE_EXTENSION,
  asTarget,
  availableTargets,
  claudeCodeLink,
  codeOf,
  cursorLink,
  deliverPrompt,
  promptActionTitle,
  targetsToOffer,
  type DeliveryHost,
  type PromptLink,
} from "../../src/actions/prompt.js";

/** A host that records what delivery did, with the targets given. */
function host(
  options: { chat?: boolean; claude?: boolean; scheme?: string; opens?: boolean } = {},
) {
  const done = {
    copied: [] as string[],
    status: [] as string[],
    notices: [] as string[],
    commands: [] as [string, unknown][],
    links: [] as PromptLink[],
  };
  const fake: DeliveryHost = {
    uriScheme: options.scheme ?? "vscode",
    copy: (text) => {
      done.copied.push(text);
      return Promise.resolve();
    },
    status: (message) => done.status.push(message),
    notice: (message) => done.notices.push(message),
    hasCommand: (id) => Promise.resolve(id === CHAT_COMMAND && options.chat === true),
    hasExtension: (id) => id === CLAUDE_CODE_EXTENSION && options.claude === true,
    executeCommand: (id, argument) => {
      done.commands.push([id, argument]);
      return Promise.resolve();
    },
    openLink: (link) => {
      done.links.push(link);
      return Promise.resolve(options.opens ?? true);
    },
  };
  return { fake, done };
}

const PROMPT = "Fix this problem in `docs/install.md`.\n\nWhere: docs/install.md:5 & more\n";

describe("delivering a prompt", () => {
  it("copies it by default and says to paste it", async () => {
    const { fake, done } = host();
    const delivery = await deliverPrompt(PROMPT, "clipboard", fake);
    expect(done.copied).toEqual([PROMPT]);
    expect(delivery).toEqual({
      target: "clipboard",
      delivered: "clipboard",
      messages: ["Prompt copied. Paste it into your agent."],
    });
  });

  it("opens VS Code's chat with the prompt as a query that waits to be sent", async () => {
    const { fake, done } = host({ chat: true });
    const delivery = await deliverPrompt(PROMPT, "chat", fake);
    expect(done.commands).toEqual([[CHAT_COMMAND, { query: PROMPT, isPartialQuery: true }]]);
    expect(done.copied).toEqual([]);
    expect(delivery.delivered).toBe("chat");
  });

  it("opens Claude Code through its URI handler, in this editor's scheme", async () => {
    const { fake, done } = host({ claude: true, scheme: "vscode-insiders" });
    await deliverPrompt(PROMPT, "claude-code", fake);
    expect(done.links).toEqual([claudeCodeLink(PROMPT, "vscode-insiders")]);
    const [link] = done.links;
    expect(link?.authority).toBe("anthropic.claude-code");
    expect(link?.path).toBe("/open");
    // Encoded once, so the handler's query parameter reads back as the prompt.
    expect(new URLSearchParams(link?.query).get("prompt")).toBe(PROMPT);
  });

  it("opens Cursor's chat through its cursor:// link only, in Cursor", async () => {
    const { fake, done } = host({ scheme: "cursor" });
    await deliverPrompt(PROMPT, "cursor", fake);
    expect(done.links).toEqual([cursorLink(PROMPT)]);
    expect(done.links[0]?.scheme).toBe("cursor");
    expect(new URLSearchParams(done.links[0]?.query).get("text")).toBe(PROMPT);
  });

  it("copies instead when the target isn't there, and says so", async () => {
    for (const target of ["chat", "claude-code", "cursor"] as const) {
      const { fake, done } = host();
      const delivery = await deliverPrompt(PROMPT, target, fake);
      expect(done.copied).toEqual([PROMPT]);
      expect(done.links).toEqual([]);
      expect(done.commands).toEqual([]);
      expect(delivery.delivered).toBe("clipboard");
      expect(delivery.messages[0]).toMatch(/isn't available, so the prompt was copied\./);
    }
  });

  it("copies instead when nothing handles the link", async () => {
    const { fake, done } = host({ claude: true, opens: false });
    const delivery = await deliverPrompt(PROMPT, "claude-code", fake);
    expect(done.copied).toEqual([PROMPT]);
    expect(delivery.delivered).toBe("clipboard");
  });

  it("copies a prompt too long for Cursor's link", async () => {
    const { fake, done } = host({ scheme: "cursor" });
    const long = "`".repeat(4_000);
    const delivery = await deliverPrompt(long, "cursor", fake);
    expect(done.links).toEqual([]);
    expect(done.copied).toEqual([long]);
    expect(delivery.messages[0]).toMatch(/too long for Cursor's link/);
  });

  it("says to save a file with unsaved changes", async () => {
    const { fake, done } = host();
    await deliverPrompt(PROMPT, "clipboard", fake, true);
    expect(done.notices).toEqual([
      "The file has unsaved changes; save it before your agent starts.",
    ]);
  });
});

describe("the targets", () => {
  it("are the clipboard, and those this editor has", async () => {
    expect(await availableTargets(host().fake)).toEqual(["clipboard"]);
    expect(await availableTargets(host({ chat: true, claude: true }).fake)).toEqual([
      "clipboard",
      "chat",
      "claude-code",
    ]);
    expect(await availableTargets(host({ scheme: "cursor" }).fake)).toEqual([
      "clipboard",
      "cursor",
    ]);
  });

  it("offer the ones besides the clipboard and the one in use", () => {
    expect(targetsToOffer(["clipboard"], "clipboard")).toEqual([]);
    expect(targetsToOffer(["clipboard", "chat", "claude-code"], "chat")).toEqual(["claude-code"]);
  });

  it("read the setting, the clipboard for anything else", () => {
    expect(asTarget("claude-code")).toBe("claude-code");
    expect(asTarget("web")).toBe("clipboard");
    expect(asTarget(undefined)).toBe("clipboard");
  });
});

describe("the code action", () => {
  it("is titled for the task, with the code when several problems are at the cursor", () => {
    expect(promptActionTitle("ASC036", false)).toBe("Prompt agent: fix this problem");
    expect(promptActionTitle("ASC036", true)).toBe("Prompt agent: fix this problem (ASC036)");
  });

  it("reads a diagnostic's code in either form the client gives", () => {
    expect(codeOf("ASC036")).toBe("ASC036");
    expect(codeOf({ value: "ASC036", target: "https://example.com" })).toBe("ASC036");
    expect(codeOf(36)).toBeUndefined();
  });
});
