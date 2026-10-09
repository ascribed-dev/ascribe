import { expect, it } from "vitest";
import { ServerStatus, type ServerState } from "../../src/serverState.js";

it("fires each change of a server's state, and only changes", () => {
  const status = new ServerStatus();
  const seen: ServerState[] = [];
  status.onDidChange((state) => seen.push(state));
  expect(status.state).toBe("stopped");
  status.set("starting");
  status.set("running");
  status.set("running");
  status.set("failed");
  status.set("stopped");
  status.set("starting");
  expect(seen).toEqual(["starting", "running", "failed", "stopped", "starting"]);
  expect(status.state).toBe("starting");
  status.dispose();
  status.set("running");
  expect(seen).toHaveLength(5);
});
