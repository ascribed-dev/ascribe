import { execFileSync, spawnSync, type ChildProcess } from "node:child_process";

/**
 * The language servers' processes that are running, each with what it
 * started: killed when the extension host exits, so none outlives it.
 */
const living = new Set<ChildProcess>();

/** Whether a process hasn't exited yet. */
export function alive(child: ChildProcess): boolean {
  return child.exitCode === null && child.signalCode === null;
}

/** Follows a server's process until it exits, to kill it with the host if it hasn't. */
export function follow(child: ChildProcess): void {
  if (!alive(child) || living.has(child)) return;
  living.add(child);
  child.once("exit", () => living.delete(child));
}

/**
 * Kills a process and the processes it started, at once: a server that
 * didn't stop when asked, or one still running when the host exits.
 * `@ascribed/cli`'s `ascribe` is a Node script that starts the native
 * binary, and a `.cmd` shim on Windows runs in a shell, so killing the
 * process alone would leave the server running. Synchronous, so it can run
 * as the host exits.
 */
export function killTree(child: ChildProcess, platform: NodeJS.Platform = process.platform): void {
  living.delete(child);
  const pid = child.pid;
  if (pid === undefined || !alive(child)) return;
  try {
    if (platform === "win32") {
      execFileSync("taskkill", ["/T", "/F", "/PID", String(pid)], {
        stdio: "ignore",
        windowsHide: true,
      });
    } else {
      // Children first, while they're still the process's to find.
      spawnSync("/bin/sh", ["-c", KILL_TREE, "kill-tree", String(pid)], { stdio: "ignore" });
    }
  } catch {
    // Gone already, or the tool is missing: the kill below is all there is.
  }
  try {
    child.kill("SIGKILL");
  } catch {
    // Gone already.
  }
}

/** Kills every server process still running: what the host does as it exits. */
export function killAll(): void {
  for (const child of living) killTree(child);
}

/** A shell function that kills a process's descendants, then the process, with `pgrep -P`. */
const KILL_TREE = `
tree() {
  for child in $(pgrep -P "$1"); do tree "$child"; done
  kill -9 "$1" 2>/dev/null
}
tree "$1"
`;
