/**
 * Counts language server crashes and decides whether to restart it. The count
 * covers the time since the last manual restart, since a server that keeps
 * dying on start-up is a problem to report, not to retry forever.
 */
export class CrashCounter {
  private crashes = 0;

  /** @param limit The crash that stops restarts: the server is restarted after crashes 1 to `limit - 1`. */
  constructor(private limit: number) {}

  /** Changes the limit, for a changed setting. */
  setLimit(limit: number): void {
    this.limit = limit;
  }

  /** Records a crash. Returns whether the server should be started again. */
  recordCrash(): boolean {
    this.crashes += 1;
    return this.crashes < this.limit;
  }

  /** How many crashes have been recorded. */
  get count(): number {
    return this.crashes;
  }

  /** Forgets earlier crashes, when the author restarts the server. */
  reset(): void {
    this.crashes = 0;
  }
}
