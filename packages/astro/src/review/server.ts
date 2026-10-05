// Review in the site preview, on the dev server's side: everything the
// toolbar app asks for, answered over Astro's dev toolbar channel. Every
// `git`, `ascribe`, and `gh` call happens here; the page gets the changes and
// the threads, never a token. There are no HTTP routes: any page open in the
// same browser could call a route on localhost, while the toolbar's channel
// takes only clients that have the dev server's token (`channelProblem`).
//
// Nothing runs until the reviewer turns the app on. Then the server opens
// the pull request's review (when the GitHub CLI finds one), compares the
// build with the pull request's base (or the default branch), and compares
// again after each rebuild.
import path from "node:path";
import type { OverlayMethod, ReviewSession } from "@ascribed/review/github";
import type { PageRef } from "@ascribed/review/place";
import type { DiffResult } from "./diff.js";
import { messageOf, type Connection } from "./github.js";
import {
  CHANGED_EVENT,
  REQUEST_EVENT,
  RESULT_EVENT,
  type ChangedPage,
  type DiffPage,
  type PageView,
  type Request,
  type Result,
  type Status,
  type ThreadsState,
} from "./protocol.js";
import { normalizeRoute, type RoutedPage } from "./routes.js";

/** A page's end of the channel: the server answers it alone. */
export interface ChannelClient {
  send(event: string, payload?: unknown): void;
}

/**
 * The toolbar's channel, as `astro:server:setup`'s `toolbar` has it. Its
 * listeners get the sending page's client too (it is Vite's `server.hot`);
 * without one, an answer goes to every page, which tell theirs by `tab`.
 */
export interface ToolbarChannel {
  on(event: string, callback: (data: unknown, client?: ChannelClient) => void): void;
  send(event: string, payload: unknown): void;
}

export interface ReviewServerOptions {
  channel: ToolbarChannel;
  logger: { info(message: string): void; warn(message: string): void };
  build: string;
  /** The content root, absolute. */
  contentRoot: string;
  /** Why someone else could use the channel, if they could: then GitHub stays off. */
  channelProblem: string | undefined;
  /** Compares the build with `base` (the default branch when `undefined`). */
  diff(base: string | undefined): Promise<DiffResult>;
  /** Opens the pull request's review. */
  connect(): Promise<Connection>;
  /** Writes the build's JSON output, in turn with the dev server's builds: it has the routes. */
  writeRoutes(): Promise<void>;
  /** Every page of the build, by normalized route. */
  readRoutes(): Promise<Map<string, RoutedPage>>;
  /** How long to wait after a rebuild before comparing again. Default 150 ms. */
  debounceMs?: number;
}

/** The overlay's requests, which go to the session. */
const OVERLAY: ReadonlySet<string> = new Set<OverlayMethod>([
  "load",
  "commentTarget",
  "comment",
  "reply",
  "allThreads",
  "resolve",
  "submit",
  "discard",
]);

export class ReviewServer {
  private on = false;
  private problem: string | undefined;
  private connection: Connection | undefined;
  private connecting: Promise<Connection> | undefined;
  private diffed: { result: DiffResult } | { error: string } | undefined;
  private diffing: Promise<void> | undefined;
  private stale = false;
  private timer: ReturnType<typeof setTimeout> | undefined;
  private routes: Promise<Map<string, RoutedPage>> | undefined;
  /** The page each overlay last read, by content path: comments are made on it. */
  private readonly pages = new Map<string, PageRef>();
  /** How long the last comparison took, in milliseconds. */
  lastDiffMs: number | undefined;

  constructor(private readonly options: ReviewServerOptions) {
    this.problem = options.channelProblem;
    options.channel.on(REQUEST_EVENT, (data, client) => {
      void this.receive(data, client);
    });
  }

  /**
   * The address the dev server listens on, once it does: one other machines
   * can reach turns comments off, as `server.host` does.
   */
  listening(address: string): void {
    if (this.problem !== undefined || isLoopback(address)) return;
    this.problem = NETWORK_PROBLEM;
    this.connection = undefined;
    this.connecting = undefined;
  }

  /** Whether review is on: then the dev server's builds write JSON output too. */
  get active(): boolean {
    return this.on;
  }

  /** The dev server rebuilt: compare again soon, and read the routes again when asked. */
  rebuilt(): void {
    if (!this.on) return;
    this.stale = true;
    this.routes = undefined;
    if (this.timer) clearTimeout(this.timer);
    this.timer = setTimeout(() => {
      this.timer = undefined;
      void this.changes();
    }, this.options.debounceMs ?? 150);
  }

  /** The dev server stopped. */
  dispose(): void {
    if (this.timer) clearTimeout(this.timer);
    this.on = false;
  }

  private async receive(data: unknown, client: ChannelClient | undefined): Promise<void> {
    const request = data as Partial<Request> | null;
    if (typeof request?.id !== "number" || typeof request.method !== "string") return;
    const params =
      typeof request.params === "object" && request.params !== null ? request.params : {};
    const tab = typeof request.tab === "string" ? request.tab : "";
    const answer: Result = { tab, id: request.id };
    try {
      answer.result = await this.handle(request.method, params, tab);
    } catch (error) {
      const code = (error as { code?: unknown }).code;
      answer.error =
        typeof code === "string"
          ? { message: messageOf(error), code }
          : { message: messageOf(error) };
    }
    if (typeof client?.send === "function") client.send(RESULT_EVENT, answer);
    else this.options.channel.send(RESULT_EVENT, answer);
  }

  /** Answers one request. Rejects with a sentence to show. */
  async handle(method: string, params: Record<string, unknown>, tab = ""): Promise<unknown> {
    switch (method) {
      case "status":
        return { on: this.on } satisfies Status;
      case "start":
        await this.start();
        return { on: true } satisfies Status;
      case "stop":
        this.stop();
        this.changed(tab);
        return { on: false } satisfies Status;
      case "page":
        return this.pageView(params);
      case "refresh":
        await this.refresh();
        this.changed(tab);
        return null;
    }
    if (!OVERLAY.has(method)) throw new Error(`Unknown request: ${method}.`);
    const connection = this.on ? await this.connected() : undefined;
    if (connection?.state !== "on") throw new Error("Review comments aren't on.");
    const github = await import("@ascribed/review/github");
    const pagePath = typeof params["path"] === "string" ? params["path"] : undefined;
    const answer = await github.answerRequest(
      {
        session: connection.session,
        page: pagePath === undefined ? undefined : { build: this.options.build, path: pagePath },
        lastPage: pagePath === undefined ? undefined : this.pages.get(pagePath),
        changedPages: async () => {
          const changes = await this.changes();
          return "result" in changes ? changes.result.pages : [];
        },
      },
      method as OverlayMethod,
      params,
    );
    if (answer.page) this.pages.set(answer.page.path, answer.page);
    if (answer.changed) this.changed(tab);
    return answer.result;
  }

  private async start(): Promise<void> {
    if (this.on) return;
    this.on = true;
    this.stale = true;
    this.routes = undefined;
    this.connection = undefined;
    const started = Date.now();
    // The routes come from the JSON output, which the next builds write too.
    const routes = this.options.writeRoutes().catch((error: unknown) => {
      this.options.logger.warn(`couldn't write the routes for review: ${messageOf(error)}`);
    });
    await this.connected();
    const changes = await this.changes();
    await routes;
    if ("result" in changes) {
      const base = changes.result.base;
      this.options.logger.info(
        `review: compared "${this.options.build}" with ${base.requested} in ${this.lastDiffMs ?? 0} ms (started in ${Date.now() - started} ms)`,
      );
    } else {
      this.options.logger.warn(`review: ${changes.error}`);
    }
  }

  private stop(): void {
    this.on = false;
    this.connection = undefined;
    this.connecting = undefined;
    this.diffed = undefined;
    this.routes = undefined;
    this.pages.clear();
    if (this.timer) clearTimeout(this.timer);
    this.timer = undefined;
  }

  private async refresh(): Promise<void> {
    if (!this.on) return;
    const connection = await this.connected();
    if (connection.state === "on") await connection.session.refresh();
    else this.connection = undefined;
    // A pull request found now changes the base.
    await this.connected();
    this.stale = true;
    await this.changes();
  }

  /** The pull request's review, opened once while review is on. */
  private connected(): Promise<Connection> {
    if (this.connection) return Promise.resolve(this.connection);
    if (this.problem !== undefined) {
      this.connection = {
        state: "unprotected",
        message: `Showing changes only. Comments are off because ${this.problem}.`,
      };
      return Promise.resolve(this.connection);
    }
    this.connecting ??= this.options
      .connect()
      .catch((error: unknown): Connection => ({ state: "error", message: messageOf(error) }))
      .then((connection) => {
        this.connecting = undefined;
        // The server turned out to listen on the network meanwhile.
        if (this.problem !== undefined) return this.connected();
        if (this.on) this.connection = connection;
        return connection;
      });
    return this.connecting;
  }

  /** The build's changes, compared again first if a rebuild made them stale. */
  private async changes(): Promise<{ result: DiffResult } | { error: string }> {
    while (this.diffing) await this.diffing;
    if (this.stale || this.diffed === undefined) {
      this.stale = false;
      this.diffing = this.compare().finally(() => {
        this.diffing = undefined;
      });
      await this.diffing;
    }
    return this.diffed ?? { error: "Review is off." };
  }

  private async compare(): Promise<void> {
    const connection = await this.connected();
    const base = connection.state === "on" ? connection.base : undefined;
    const started = Date.now();
    try {
      this.diffed = { result: await this.options.diff(base) };
    } catch (error) {
      this.diffed = { error: messageOf(error) };
    }
    this.lastDiffMs = Date.now() - started;
  }

  private async pageView(params: Record<string, unknown>): Promise<PageView> {
    if (!this.on) throw new Error("Review is off.");
    const route = normalizeRoute(typeof params["route"] === "string" ? params["route"] : "/");
    const fromPage = typeof params["path"] === "string" ? params["path"] : null;
    this.routes ??= this.options.readRoutes().catch(() => new Map<string, RoutedPage>());
    const [changes, connection, routes] = await Promise.all([
      this.changes(),
      this.connected(),
      this.routes,
    ]);
    const pages = "result" in changes ? changes.result.pages : [];
    const shown = pages.filter((p) => p.status !== "removed");
    let page = shown.find((p) => normalizeRoute(p.route) === route) ?? null;
    const pagePath = page?.path ?? fromPage ?? routes.get(route)?.path ?? null;
    if (page === null && pagePath !== null) page = shown.find((p) => p.path === pagePath) ?? null;
    return {
      kind: pagePath === null ? "not-page" : "page",
      path: pagePath,
      page,
      base: "result" in changes ? changes.result.base : null,
      problem: "error" in changes ? changes.error : null,
      errors: "result" in changes ? changes.result.errors : 0,
      threads: threadsState(connection),
      changedPages: pages.map((p) => listed(p, routes.get(normalizeRoute(p.route))?.title ?? null)),
      contentRoot: this.options.contentRoot,
      separator: path.sep,
    };
  }

  /** Tells every page the threads or the changes changed; `from` is the page that did it. */
  private changed(from: string): void {
    this.options.channel.send(CHANGED_EVENT, { from });
  }
}

/** A changed page as the list shows it: without its changes, with its title. */
function listed(page: DiffPage, title: string | null): ChangedPage {
  return {
    title,
    path: page.path,
    route: page.route,
    status: page.status,
    own_file_changed: page.own_file_changed,
    because: page.because,
    page_changed: page.page_changed,
    counts: page.counts,
  };
}

/** What the page shows about the threads. */
function threadsState(connection: Connection): ThreadsState {
  if (connection.state !== "on") return connection;
  const pr: ReviewSession["pullRequest"] = connection.session.pullRequest;
  return {
    state: "on",
    pullRequest: { number: pr.number, url: pr.url, baseRefName: pr.baseRefName },
    local: connection.local,
  };
}

/**
 * Why someone other than the developer could use the dev server's toolbar
 * channel, or `undefined` when they can't. Vite takes a browser's connection
 * only with the token in its client script, which the page's own origin can
 * read and, by default, only localhost origins can fetch. These settings undo
 * that, and a server listening on the network gives its pages, token and all,
 * to anyone who can reach it.
 */
export function channelProblem(config: {
  server: { cors?: unknown; allowedHosts?: unknown; host?: unknown };
  legacy?: { skipWebSocketTokenCheck?: unknown } | undefined;
}): string | undefined {
  const pages = "other web pages could talk to this dev server";
  if (config.legacy?.skipWebSocketTokenCheck === true) {
    return `${pages}: \`legacy.skipWebSocketTokenCheck\` is on`;
  }
  if (config.server.allowedHosts === true) return `${pages}: \`server.allowedHosts\` is \`true\``;
  const cors = config.server.cors;
  const origin =
    typeof cors === "object" && cors !== null ? (cors as { origin?: unknown }).origin : undefined;
  if (cors === true || origin === true || origin === "*") {
    return `${pages}: \`server.cors\` lets any origin read its pages`;
  }
  if (!isLoopback(config.server.host)) return NETWORK_PROBLEM;
  return undefined;
}

const NETWORK_PROBLEM =
  "the dev server is listening on the network (`--host`), so anyone who can reach it could comment as you";

/** Whether Vite's `server.host`, or the address the server listens on, keeps the server on this machine: unset, `false`, or a loopback name. */
export function isLoopback(host: unknown): boolean {
  if (host === undefined || host === false) return true;
  if (typeof host !== "string") return false;
  const name = host.toLowerCase().replace(/^\[(.*)\]$/, "$1");
  return name === "localhost" || name === "::1" || /^127(\.\d{1,3}){3}$/.test(name);
}
