// A fake GitHub behind a fake `gh`: a Node script put first on the dev
// server's path, answering `gh api graphql` from a state file the test reads
// back. It knows one pull request and the operations review uses to read its
// threads, comment into a pending review, and submit it. POSIX only: Windows
// can't run the script as `gh` without a shell.
import { chmodSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

/** A thread as the fake stores it, in the API's shape. */
export interface FakeThread {
  id: string;
  path: string;
  line: number;
  comments: { id: string; body: string; state: "PENDING" | "SUBMITTED"; login: string }[];
}

export interface FakeState {
  pullRequest: {
    id: string;
    number: number;
    owner: string;
    name: string;
    headRefName: string;
    headRefOid: string;
    baseRefName: string;
    baseRefOid: string;
    files: string[];
  };
  threads: FakeThread[];
  /** The viewer's review, once there is one. */
  review: { id: string; state: string; body: string } | null;
  submitted: { event: string; body: string | null }[];
}

const SCRIPT = String.raw`#!/usr/bin/env node
const fs = require("node:fs");
const path = require("node:path");
const file = path.join(__dirname, "state.json");
const args = process.argv.slice(2);
if (args[0] === "--version") {
  process.stdout.write("gh version 2.60.0 (2026-01-01)\n");
  process.exit(0);
}
if (args[0] === "auth") process.exit(0);
let query = "";
try { query = fs.readFileSync(0, "utf8"); } catch {}
const vars = {};
for (let i = 0; i < args.length; i++) {
  if (args[i] !== "-f" && args[i] !== "-F") continue;
  const [name, ...rest] = args[++i].split("=");
  const value = rest.join("=");
  if (name === "query") continue;
  vars[name] = args[i - 1] === "-F" ? JSON.parse(value) : value;
}
const state = JSON.parse(fs.readFileSync(file, "utf8"));
const pr = state.pullRequest;
const op = (/\b(?:query|mutation) (\w+)/.exec(query) || [])[1];
const page = (nodes) => ({ pageInfo: { hasNextPage: false, endCursor: null }, nodes });
const author = (login) => ({ login, avatarUrl: null });
const comment = (c) => ({
  id: c.id, body: c.body, createdAt: "2026-10-01T12:00:00Z",
  url: "https://github.com/" + pr.owner + "/" + pr.name + "/pull/" + pr.number + "#" + c.id,
  state: c.state, diffHunk: "@@ -1,1 +1,1 @@", author: author(c.login),
  commit: { oid: pr.headRefOid }, originalCommit: { oid: pr.headRefOid },
});
const thread = (t) => ({
  id: t.id, path: t.path, line: t.line, startLine: null, originalLine: t.line, originalStartLine: null,
  diffSide: "RIGHT", startDiffSide: null, isResolved: false, isOutdated: false, subjectType: "LINE",
  viewerCanResolve: true, viewerCanUnresolve: true, viewerCanReply: true,
  comments: page(t.comments.map(comment)),
});
const repo = (field, nodes) => ({ repository: { pullRequest: { [field]: page(nodes) } } });
let data;
switch (op) {
  case "Viewer": data = { viewer: { login: "reviewer" } }; break;
  case "FindPullRequest":
    data = { repository: { pullRequests: { nodes: vars.head === pr.headRefName ? [{
      id: pr.id, number: pr.number, url: "https://github.com/" + pr.owner + "/" + pr.name + "/pull/" + pr.number,
      baseRefName: pr.baseRefName, baseRefOid: pr.baseRefOid, headRefName: pr.headRefName, headRefOid: pr.headRefOid,
      headRepositoryOwner: { login: pr.owner }, headRepository: { name: pr.name },
    }] : [] } } };
    break;
  case "ReviewThreads": data = repo("reviewThreads", state.threads.map(thread)); break;
  case "Conversation": data = repo("comments", []); break;
  case "Reviews":
    data = repo("reviews", state.review ? [{ id: state.review.id, state: state.review.state, body: state.review.body,
      createdAt: "2026-10-01T12:00:00Z", url: "https://github.com/x", viewerDidAuthor: true, author: author("reviewer") }] : []);
    break;
  case "Files": data = repo("files", pr.files.map((p) => ({ path: p }))); break;
  case "AddReview":
    state.review = { id: "PRR_1", state: "PENDING", body: vars.body };
    data = { addPullRequestReview: { pullRequestReview: { id: "PRR_1" } } };
    break;
  case "UpdateReview":
    state.review.body = vars.body;
    data = { updatePullRequestReview: { pullRequestReview: { id: state.review.id } } };
    break;
  case "AddThread": {
    const t = { id: "PRRT_" + (state.threads.length + 1), path: vars.path, line: vars.line,
      comments: [{ id: "PRRC_" + Date.now(), body: vars.body, state: "PENDING", login: "reviewer" }] };
    state.threads.push(t);
    data = { addPullRequestReviewThread: { thread: thread(t) } };
    break;
  }
  case "SubmitReview":
    state.review.state = "COMMENTED";
    for (const t of state.threads) for (const c of t.comments) c.state = "SUBMITTED";
    state.submitted.push({ event: vars.event, body: vars.body === undefined ? null : vars.body });
    data = { submitPullRequestReview: { pullRequestReview: { id: state.review.id, state: "COMMENTED" } } };
    break;
  default:
    process.stdout.write(JSON.stringify({ errors: [{ message: "the fake doesn't know " + op }] }));
    process.exit(1);
}
// Only a change is written, and atomically: review runs several gh at once,
// and one must never read another's half-written file.
if (["AddReview", "UpdateReview", "AddThread", "SubmitReview"].includes(op)) {
  const temp = file + "." + process.pid;
  fs.writeFileSync(temp, JSON.stringify(state));
  fs.renameSync(temp, file);
}
process.stdout.write(JSON.stringify({ data }));
`;

export interface FakeGitHub {
  /** The `PATH` with the fake `gh` first. */
  path: string;
  state(): FakeState;
  remove(): void;
}

/** Writes a fake `gh` serving `state`. */
export function fakeGitHub(state: FakeState): FakeGitHub {
  const dir = mkdtempSync(path.join(tmpdir(), "fake-github-"));
  writeFileSync(path.join(dir, "gh"), SCRIPT);
  chmodSync(path.join(dir, "gh"), 0o755);
  writeFileSync(path.join(dir, "state.json"), JSON.stringify(state));
  return {
    path: `${dir}${path.delimiter}${process.env["PATH"] ?? ""}`,
    state: () => JSON.parse(readFileSync(path.join(dir, "state.json"), "utf8")) as FakeState,
    remove: () => rmSync(dir, { recursive: true, force: true }),
  };
}
