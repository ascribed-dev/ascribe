// A fake GitHub for the threads suite: one pull request, its review threads,
// and the viewer's pending review, kept as GitHub keeps them, so the session
// reads back what it wrote. Built from @ascribed/review's own test builders.
import type { GitHubTransport } from "@ascribed/review/github";
import { comment, paged, thread } from "../../../../review/test/helpers/github.js";

type RawThread = ReturnType<typeof thread>;
type RawComment = ReturnType<typeof comment>;

interface Review {
  id: string;
  state: "PENDING" | "COMMENTED" | "APPROVED" | "CHANGES_REQUESTED";
  body: string;
}

export class StatefulGitHub implements GitHubTransport {
  /** Every operation called, with its variables. */
  readonly calls: { operation: string; variables: Record<string, unknown> }[] = [];
  readonly threads: RawThread[] = [];
  readonly reviews: Review[] = [];
  private next = 0;

  constructor(
    private readonly pr: {
      number: number;
      baseOid: string;
      headOid: string;
      files: string[];
    },
  ) {}

  /** Adds a submitted thread from someone else, on a line at the head commit. */
  addThread(path: string, line: number, body: string): RawThread {
    const raw = thread({
      id: `PRRT_${++this.next}`,
      path,
      line,
      comments: [
        comment({ body, login: "ana", commit: this.pr.headOid, originalCommit: this.pr.headOid }),
      ],
    });
    this.threads.push(raw);
    return raw;
  }

  async graphql<T>(query: string, variables: Record<string, unknown>): Promise<T> {
    const operation = /\b(?:query|mutation) (\w+)/.exec(query)?.[1] ?? "?";
    this.calls.push({ operation, variables });
    return structuredClone(this.answer(operation, variables)) as T;
  }

  private pending(): Review | undefined {
    return this.reviews.find((r) => r.state === "PENDING");
  }

  private answer(operation: string, variables: Record<string, unknown>): unknown {
    switch (operation) {
      case "FindPullRequest":
        return {
          repository: {
            pullRequests: {
              nodes: [
                {
                  id: "PR_1",
                  number: this.pr.number,
                  url: `https://github.com/acme/quill/pull/${this.pr.number}`,
                  baseRefName: "main",
                  baseRefOid: this.pr.baseOid,
                  headRefName: "feature",
                  headRefOid: this.pr.headOid,
                  headRepositoryOwner: { login: "acme" },
                  headRepository: { name: "quill" },
                },
              ],
            },
          },
        };
      case "Viewer":
        return { viewer: { login: "me" } };
      case "ReviewThreads":
        return paged("reviewThreads", this.threads)(variables);
      case "Conversation":
        return paged("comments", [])(variables);
      case "Reviews":
        return paged(
          "reviews",
          this.reviews.map((review) => ({
            ...review,
            createdAt: "2026-10-04T12:00:00Z",
            url: `https://github.com/acme/quill/pull/${this.pr.number}#${review.id}`,
            viewerDidAuthor: true,
            author: { login: "me", avatarUrl: null },
          })),
        )(variables);
      case "Files":
        return paged(
          "files",
          this.pr.files.map((path) => ({ path })),
        )(variables);
      case "AddReview": {
        const review: Review = {
          id: `PRR_${++this.next}`,
          state: "PENDING",
          body: String(variables["body"]),
        };
        this.reviews.push(review);
        return { addPullRequestReview: { pullRequestReview: { id: review.id } } };
      }
      case "UpdateReview": {
        const review = this.reviews.find((r) => r.id === variables["reviewId"]);
        if (review) review.body = String(variables["body"]);
        return { updatePullRequestReview: { pullRequestReview: { id: variables["reviewId"] } } };
      }
      case "AddThread": {
        const line = Number(variables["line"]);
        const raw = thread({
          id: `PRRT_${++this.next}`,
          path: String(variables["path"]),
          line,
          startLine: (variables["startLine"] as number | null) ?? null,
          comments: [this.newComment(String(variables["body"]), "PENDING")],
        });
        this.threads.push(raw);
        return { addPullRequestReviewThread: { thread: raw } };
      }
      case "AddReply": {
        const raw = this.threads.find((t) => t.id === variables["threadId"]);
        const held = typeof variables["reviewId"] === "string";
        const reply = this.newComment(String(variables["body"]), held ? "PENDING" : "SUBMITTED");
        raw?.comments.nodes.push(reply);
        return { addPullRequestReviewThreadReply: { comment: reply } };
      }
      case "Resolve":
      case "Unresolve": {
        const raw = this.threads.find((t) => t.id === variables["threadId"]);
        if (raw) raw.isResolved = operation === "Resolve";
        return {
          [operation === "Resolve" ? "resolveReviewThread" : "unresolveReviewThread"]: {
            thread: { id: variables["threadId"], isResolved: operation === "Resolve" },
          },
        };
      }
      case "SubmitReview": {
        const review = this.pending();
        if (!review) throw new Error("no pending review to submit");
        const event = String(variables["event"]);
        review.state =
          event === "APPROVE"
            ? "APPROVED"
            : event === "REQUEST_CHANGES"
              ? "CHANGES_REQUESTED"
              : "COMMENTED";
        for (const raw of this.threads) {
          for (const c of raw.comments.nodes) c.state = "SUBMITTED";
        }
        return {
          submitPullRequestReview: { pullRequestReview: { id: review.id, state: review.state } },
        };
      }
      case "DeleteReview": {
        const at = this.reviews.findIndex((r) => r.id === variables["reviewId"]);
        if (at >= 0) this.reviews.splice(at, 1);
        return { deletePullRequestReview: { clientMutationId: null } };
      }
      default:
        throw new Error(`the fake GitHub has no ${operation}`);
    }
  }

  private newComment(body: string, state: "PENDING" | "SUBMITTED"): RawComment {
    return comment({
      id: `PRRC_${++this.next}`,
      body,
      login: "me",
      state,
      commit: this.pr.headOid,
      originalCommit: this.pr.headOid,
    });
  }
}
