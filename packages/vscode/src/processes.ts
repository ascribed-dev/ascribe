import { execFileSync, spawnSync, type ChildProcess } from "node:child_process";
import { readdirSync, readFileSync } from "node:fs";

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
  if (platform === "win32") {
    try {
      execFileSync("taskkill", ["/T", "/F", "/PID", String(pid)], {
        stdio: "ignore",
        windowsHide: true,
      });
    } catch {
      // Gone already: the kill below is all there is.
    }
  } else {
    // Found before any is killed: a killed process's children are handed
    // to another parent.
    for (const descendant of descendants(pid, platform)) {
      try {
        process.kill(descendant, "SIGKILL");
      } catch {
        // Gone already.
      }
    }
  }
  try {
    child.kill("SIGKILL");
  } catch {
    // Gone already.
  }
}

/**
 * The processes a process started, and theirs, from `/proc` on Linux, which
 * has no `pgrep` in a container without procps, and from `pgrep -P`
 * elsewhere.
 */
function descendants(pid: number, platform: NodeJS.Platform): number[] {
  const children = platform === "linux" ? childrenFromProc() : undefined;
  const found: number[] = [];
  const queue = [pid];
  for (let next = queue.shift(); next !== undefined; next = queue.shift()) {
    const direct = children ? (children.get(next) ?? []) : pgrepChildren(next);
    found.push(...direct);
    queue.push(...direct);
  }
  return found;
}

/** Every process's children, by parent, read from `/proc/<pid>/stat`. */
function childrenFromProc(): Map<number, number[]> {
  const children = new Map<number, number[]>();
  let entries: string[] = [];
  try {
    // Outside FileSystem: the system's process table, not a project's files.
    entries = readdirSync("/proc").filter((name) => /^\d+$/.test(name));
  } catch {
    return children;
  }
  for (const entry of entries) {
    try {
      const stat = readFileSync(`/proc/${entry}/stat`, "utf8");
      // The name, in parentheses, can hold spaces: the fields after it are fixed.
      const parent = Number(stat.slice(stat.lastIndexOf(")") + 2).split(" ")[1]);
      children.set(parent, [...(children.get(parent) ?? []), Number(entry)]);
    } catch {
      // Exited meanwhile.
    }
  }
  return children;
}

function pgrepChildren(pid: number): number[] {
  const found = spawnSync("pgrep", ["-P", String(pid)], { encoding: "utf8" });
  return (found.stdout ?? "")
    .split("\n")
    .filter((line) => /^\d+$/.test(line.trim()))
    .map(Number);
}

/** Kills every server process still running: what the host does as it exits. */
export function killAll(): void {
  for (const child of living) killTree(child);
}
