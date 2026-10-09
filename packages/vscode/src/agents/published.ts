// The diagnostics a project's language server last published for each file,
// as the protocol has them: with the document version they're for, and the
// `data` the Problems panel drops. The tool that reports problems to agents
// reads them here, and waits here for the server to catch up with an edit.

import { comparable } from "../projects.js";
import type { LspRange } from "../shapes.js";

/** A diagnostic as the server publishes it, with Ascribe's `data` (`crates/ascribe-lsp/src/compute.rs`). */
export interface ProtocolDiagnostic {
  range: LspRange;
  /** 1 for an error, 2 for a warning. */
  severity?: number;
  code?: string | number;
  codeDescription?: { href: string };
  message: string;
  relatedInformation?: { location: { uri: string; range: LspRange }; message: string }[];
  data?: unknown;
}

/** One file's diagnostics, as last published. */
export interface Publication {
  /** The file, as a path. */
  file: string;
  /**
   * The version of the open document they're for; `null` for a file that
   * isn't open in an editor, whose text is the one on disk.
   */
  version: number | null;
  diagnostics: ProtocolDiagnostic[];
  /** When they arrived, in milliseconds since the epoch. */
  at: number;
}

/** What waiting for a file's diagnostics found. */
export interface Waited {
  /** The file's diagnostics, or `undefined` when the server never published any for it. */
  publication: Publication | undefined;
  /** Whether they're for the file as it is now; `false` when the wait ran out first. */
  current: boolean;
}

/**
 * The server's last publication for each file. `record` takes every
 * `textDocument/publishDiagnostics` notification's parameters before the
 * language client handles them, so nothing here depends on the client's
 * own copy, which keeps no version.
 */
export class PublishedDiagnostics {
  private readonly byFile = new Map<string, Publication>();
  private readonly waiting = new Set<() => void>();

  constructor(
    /** A `file:` URI as a path; `undefined` for a URI that isn't a file's. */
    private readonly toPath: (uri: string) => string | undefined,
    /** Whether a file belongs to another project, whose diagnostics this server doesn't report. */
    private readonly excluded: (file: string) => boolean = () => false,
    private readonly now: () => number = Date.now,
  ) {}

  /** Records a publication. Parameters that aren't one are ignored. */
  record(params: unknown): void {
    if (!isRecord(params) || typeof params.uri !== "string" || !Array.isArray(params.diagnostics)) {
      return;
    }
    const file = this.toPath(params.uri);
    if (file === undefined || this.excluded(file)) return;
    this.byFile.set(comparable(file), {
      file,
      version: typeof params.version === "number" ? params.version : null,
      diagnostics: params.diagnostics as ProtocolDiagnostic[],
      at: this.now(),
    });
    for (const wake of this.waiting) wake();
  }

  /** A file's last publication. */
  get(file: string): Publication | undefined {
    return this.byFile.get(comparable(file));
  }

  /** Every file's last publication, those with no diagnostics left included. */
  all(): Publication[] {
    return [...this.byFile.values()];
  }

  /** Forgets everything: the server stopped, and its diagnostics went with it. */
  clear(): void {
    this.byFile.clear();
  }

  /**
   * Resolves with a file's publication once `current` says it's for the
   * file as it is now, or with the last one when `timeoutMs` passes first.
   * `current` is asked again at each publication, for any file, so it can
   * read what's current at that moment.
   */
  waitFor(
    file: string,
    current: (publication: Publication | undefined) => boolean,
    timeoutMs: number,
  ): Promise<Waited> {
    const done = (): Waited => {
      const publication = this.get(file);
      return { publication, current: current(publication) };
    };
    const now = done();
    if (now.current || timeoutMs <= 0) return Promise.resolve(now);
    return new Promise((resolve) => {
      const finish = () => {
        clearTimeout(timer);
        this.waiting.delete(wake);
        resolve(done());
      };
      const wake = () => {
        if (current(this.get(file))) finish();
      };
      const timer = setTimeout(finish, timeoutMs);
      this.waiting.add(wake);
    });
  }
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
