import { describe, expect, it } from "vitest";
import { Handshake } from "../../src/ui/handshake.js";

describe("a webview view's handshake", () => {
  it("sends nothing to a webview before it's ready, then the newest answer", () => {
    const handshake = new Handshake<string>();
    handshake.reset();
    // The answer changes between the webview's creation and its ready message.
    expect(handshake.answer("keys.md")).toBeUndefined();
    expect(handshake.answer("quickstart.md")).toBeUndefined();
    expect(handshake.readied()).toEqual({ type: "answer", seq: 2, answer: "quickstart.md" });
  });

  it("sends each answer once the webview is ready, its number rising", () => {
    const handshake = new Handshake<string>();
    expect(handshake.readied()).toBeUndefined();
    expect(handshake.answer("a")).toEqual({ type: "answer", seq: 1, answer: "a" });
    expect(handshake.answer("b")).toEqual({ type: "answer", seq: 2, answer: "b" });
  });

  it("answers every ready, so a webview made again is drawn again", () => {
    const handshake = new Handshake<string>();
    handshake.readied();
    handshake.answer("a");
    // Hidden and shown: its document is made again, and says it's ready again.
    handshake.reset();
    expect(handshake.answer("b")).toBeUndefined();
    expect(handshake.readied()).toEqual({ type: "answer", seq: 2, answer: "b" });
    expect(handshake.readied()).toEqual({ type: "answer", seq: 2, answer: "b" });
  });

  it("keeps the numbers rising past an answer it drops", () => {
    const handshake = new Handshake<string>();
    handshake.answer("a");
    handshake.forget();
    expect(handshake.readied()).toBeUndefined();
    expect(handshake.answer("b")).toEqual({ type: "answer", seq: 2, answer: "b" });
  });
});
