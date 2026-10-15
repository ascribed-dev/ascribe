import { Emitter, type Event } from "./emitter.js";

/** Where a project's language server is in its life. */
export type ServerState = "stopped" | "starting" | "running" | "failed";

/** A server's state, with an event for each change. */
export class ServerStatus {
  private current: ServerState = "stopped";
  private readonly changed = new Emitter<ServerState>();

  get state(): ServerState {
    return this.current;
  }

  /** Fires with the new state each time it changes; setting the same state again fires nothing. */
  get onDidChange(): Event<ServerState> {
    return this.changed.event;
  }

  set(state: ServerState): void {
    if (state === this.current) return;
    this.current = state;
    this.changed.fire(state);
  }

  dispose(): void {
    this.changed.dispose();
  }
}
