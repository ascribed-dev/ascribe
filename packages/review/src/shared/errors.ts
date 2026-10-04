// The errors this package throws, each with a code a host can switch on to
// explain it.

/** Why a request to GitHub, or a step around it, failed. */
export type ReviewErrorCode =
  /** `gh` isn't on the path. */
  | "gh-missing"
  /** `gh` is older than the oldest version this package supports. */
  | "gh-too-old"
  /** `gh` isn't signed in to the host, or the host refused the token. */
  | "not-signed-in"
  /** GitHub's secondary rate limit: wait `retryAfter` seconds and try again. */
  | "rate-limited"
  /** GitHub answered with an error. */
  | "refused"
  /** The request didn't reach GitHub, or the answer couldn't be read. */
  | "network"
  /** A comment's block has lines that aren't at the pull request's head commit. */
  | "push-first"
  /** A reply can't be sent at once while the viewer has a pending review. */
  | "reply-held"
  /**
   * A comment GitHub can't anchor can't be held in the viewer's pending
   * review, which was started on GitHub without a summary.
   */
  | "cant-hold"
  /** The thread, or another object named, isn't in the pull request. */
  | "not-found"
  /** `git` failed, or the checkout isn't a git repository. */
  | "git";

/** An error from this package. `message` is a sentence a host can show. */
export class ReviewError extends Error {
  override name = "ReviewError";
  readonly code: ReviewErrorCode;
  /** For `rate-limited`: the seconds to wait, when GitHub said. */
  readonly retryAfter: number | undefined;

  constructor(
    code: ReviewErrorCode,
    message: string,
    options: { cause?: unknown; retryAfter?: number } = {},
  ) {
    super(scrub(message), options.cause === undefined ? undefined : { cause: options.cause });
    this.code = code;
    this.retryAfter = options.retryAfter;
  }
}

// GitHub's token formats: classic (`ghp_`), OAuth (`gho_`), user-to-server
// (`ghu_`), server-to-server (`ghs_`), refresh (`ghr_`), and fine-grained.
const TOKEN = /\b(?:gh[opusr]_[A-Za-z0-9_]{16,}|github_pat_[A-Za-z0-9_]{20,})\b/g;

/** `text` with anything shaped like a GitHub token replaced. */
export function scrub(text: string): string {
  return text.replace(TOKEN, "[token]");
}
