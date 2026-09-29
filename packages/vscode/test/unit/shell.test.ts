import { describe, expect, it } from "vitest";
import { shellCommand, usesShell } from "../../src/environment.js";

describe("usesShell", () => {
  it("is true only for a .cmd or .bat shim on Windows", () => {
    expect(usesShell("C:\\p\\node_modules\\.bin\\tessera.cmd", "win32")).toBe(true);
    expect(usesShell("C:\\p\\tessera.BAT", "win32")).toBe(true);
    expect(usesShell("C:\\p\\tessera.exe", "win32")).toBe(false);
    expect(usesShell("/p/node_modules/.bin/tessera.cmd", "linux")).toBe(false);
    expect(usesShell("/p/tessera", "darwin")).toBe(false);
  });
});

describe("shellCommand", () => {
  it("quotes a shim's path, so spaces in it don't split the command", () => {
    const shim = "C:\\Users\\Jane Doe\\proj\\node_modules\\.bin\\tessera.cmd";
    expect(shellCommand(shim, "win32")).toBe(`"${shim}"`);
  });

  it("leaves everything that doesn't use a shell as it is", () => {
    expect(shellCommand("/home/jane doe/tessera", "linux")).toBe("/home/jane doe/tessera");
    expect(shellCommand("C:\\Program Files\\tessera.exe", "win32")).toBe(
      "C:\\Program Files\\tessera.exe",
    );
  });
});
