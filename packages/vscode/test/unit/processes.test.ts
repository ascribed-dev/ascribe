import { spawn } from "node:child_process";
import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { alive, follow, killAll, killTree } from "../../src/processes.js";

/**
 * Whether a process id is running. One killed and waiting to be reaped isn't:
 * in a container whose first process doesn't reap, it waits for good.
 */
function running(pid: number): boolean {
  try {
    process.kill(pid, 0);
  } catch {
    return false;
  }
  try {
    return readFileSync(`/proc/${pid}/stat`, "utf8").split(") ")[1]?.[0] !== "Z";
  } catch {
    return true;
  }
}

async function until(check: () => boolean, timeout = 5_000): Promise<void> {
  const deadline = Date.now() + timeout;
  while (!check()) {
    if (Date.now() > deadline) throw new Error("timed out");
    await new Promise((resolve) => setTimeout(resolve, 50));
  }
}

/**
 * A process that starts another and ignores being asked to stop, as
 * `@ascribed/cli`'s script starts the native binary. Resolves with both
 * once the inner one says its id.
 */
async function wrapper() {
  const inner = "process.on('SIGTERM', () => {}); setInterval(() => {}, 1e9);";
  const outer = `
    const { spawn } = require("node:child_process");
    const child = spawn(process.execPath, ["-e", ${JSON.stringify(inner)}], { stdio: "ignore" });
    process.on("SIGTERM", () => {});
    console.log(child.pid);
    setInterval(() => {}, 1e9);
  `;
  const child = spawn(process.execPath, ["-e", outer], { stdio: ["ignore", "pipe", "ignore"] });
  const innerPid = await new Promise<number>((resolve) =>
    child.stdout.once("data", (data: Buffer) => resolve(Number(String(data).trim()))),
  );
  return { child, innerPid };
}

describe.skipIf(process.platform === "win32")("killing a server's processes", () => {
  it("kills a process and the one it started", async () => {
    const { child, innerPid } = await wrapper();
    expect(running(innerPid)).toBe(true);
    killTree(child);
    await until(() => !alive(child) && !running(innerPid));
  });

  it("kills every process it follows that's still running", async () => {
    const first = await wrapper();
    const second = await wrapper();
    follow(first.child);
    follow(second.child);
    killAll();
    await until(
      () =>
        !alive(first.child) &&
        !alive(second.child) &&
        !running(first.innerPid) &&
        !running(second.innerPid),
    );
  });

  it("leaves a process that exited alone", async () => {
    const child = spawn(process.execPath, ["-e", ""], { stdio: "ignore" });
    await new Promise((resolve) => child.once("exit", resolve));
    expect(alive(child)).toBe(false);
    expect(() => killTree(child)).not.toThrow();
  });
});
