// A minimal LSP client over the stdio of `ascribe lsp`: just enough to send
// `ascribe/preview` requests, without the VS Code client library.
import { spawn, type ChildProcessWithoutNullStreams } from "node:child_process";
import { pathToFileURL } from "node:url";

interface Pending {
  resolve: (value: unknown) => void;
  reject: (error: Error) => void;
}

export class LspClient {
  private buffer = Buffer.alloc(0);
  private nextId = 0;
  private readonly pending = new Map<number, Pending>();

  private constructor(private readonly child: ChildProcessWithoutNullStreams) {
    child.stdout.on("data", (chunk: Buffer) => this.receive(chunk));
    child.stderr.on("data", () => undefined);
    child.on("exit", () => {
      for (const { reject } of this.pending.values()) reject(new Error("the server exited"));
      this.pending.clear();
    });
  }

  /** Starts `ascribe lsp` on the project at `root` and completes the handshake. */
  static async start(binary: string, root: string): Promise<LspClient> {
    const child = spawn(binary, ["lsp"], { cwd: root, stdio: ["pipe", "pipe", "pipe"] });
    const client = new LspClient(child);
    const uri = pathToFileURL(root).toString();
    await client.request("initialize", {
      processId: null,
      rootUri: uri,
      workspaceFolders: [{ uri, name: "project" }],
      capabilities: {},
    });
    client.notify("initialized", {});
    return client;
  }

  request(method: string, params: unknown): Promise<unknown> {
    const id = ++this.nextId;
    return new Promise((resolve, reject) => {
      this.pending.set(id, { resolve, reject });
      this.send({ jsonrpc: "2.0", id, method, params });
    });
  }

  notify(method: string, params: unknown): void {
    this.send({ jsonrpc: "2.0", method, params });
  }

  async close(): Promise<void> {
    try {
      await this.request("shutdown", null);
      this.notify("exit", null);
    } catch {
      // Already gone.
    }
    await new Promise<void>((resolve) => {
      if (this.child.exitCode !== null) return resolve();
      this.child.once("exit", () => resolve());
      setTimeout(() => this.child.kill(), 2_000).unref();
    });
  }

  private send(message: unknown): void {
    const body = Buffer.from(JSON.stringify(message), "utf8");
    this.child.stdin.write(
      Buffer.concat([Buffer.from(`Content-Length: ${body.length}\r\n\r\n`), body]),
    );
  }

  private receive(chunk: Buffer): void {
    this.buffer = Buffer.concat([this.buffer, chunk]);
    for (;;) {
      const end = this.buffer.indexOf("\r\n\r\n");
      if (end < 0) return;
      const header = this.buffer.subarray(0, end).toString("ascii");
      const length = Number(/Content-Length: (\d+)/i.exec(header)?.[1]);
      if (this.buffer.length < end + 4 + length) return;
      const body = this.buffer.subarray(end + 4, end + 4 + length).toString("utf8");
      this.buffer = this.buffer.subarray(end + 4 + length);
      this.dispatch(JSON.parse(body) as Record<string, unknown>);
    }
  }

  private dispatch(message: Record<string, unknown>): void {
    const id = message["id"];
    if (typeof id === "number" && "method" in message === false) {
      const pending = this.pending.get(id);
      if (!pending) return;
      this.pending.delete(id);
      if ("error" in message) {
        pending.reject(new Error(JSON.stringify(message["error"])));
      } else {
        pending.resolve(message["result"]);
      }
    } else if (id !== undefined && "method" in message) {
      // A request from the server (registerCapability, …): accept.
      this.send({ jsonrpc: "2.0", id, result: null });
    }
  }
}
