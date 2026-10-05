// The GraphQL documents this package sends.

const AUTHOR = "author { login avatarUrl }";

const COMMENT = `
fragment ReviewComment on PullRequestReviewComment {
  id
  body
  createdAt
  url
  state
  diffHunk
  ${AUTHOR}
  commit { oid }
  originalCommit { oid }
}`;

const THREAD = `
fragment ReviewThread on PullRequestReviewThread {
  id
  path
  line
  startLine
  originalLine
  originalStartLine
  diffSide
  startDiffSide
  isResolved
  isOutdated
  subjectType
  viewerCanResolve
  viewerCanUnresolve
  viewerCanReply
  comments(first: 100) {
    pageInfo { hasNextPage endCursor }
    nodes { ...ReviewComment }
  }
}`;

const PAGE = "pageInfo { hasNextPage endCursor }";

export const FIND_PULL_REQUEST = `
query FindPullRequest($owner: String!, $name: String!, $head: String!) {
  repository(owner: $owner, name: $name) {
    pullRequests(headRefName: $head, states: [OPEN], first: 20, orderBy: { field: CREATED_AT, direction: DESC }) {
      nodes {
        id
        number
        url
        baseRefName
        baseRefOid
        headRefName
        headRefOid
        headRepositoryOwner { login }
        headRepository { name }
      }
    }
  }
}`;

export const REVIEW_THREADS = `
query ReviewThreads($owner: String!, $name: String!, $number: Int!, $after: String) {
  repository(owner: $owner, name: $name) {
    pullRequest(number: $number) {
      reviewThreads(first: 100, after: $after) {
        ${PAGE}
        nodes { ...ReviewThread }
      }
    }
  }
}
${THREAD}
${COMMENT}`;

export const THREAD_COMMENTS = `
query ThreadComments($id: ID!, $after: String) {
  node(id: $id) {
    ... on PullRequestReviewThread {
      comments(first: 100, after: $after) {
        ${PAGE}
        nodes { ...ReviewComment }
      }
    }
  }
}
${COMMENT}`;

export const CONVERSATION = `
query Conversation($owner: String!, $name: String!, $number: Int!, $after: String) {
  repository(owner: $owner, name: $name) {
    pullRequest(number: $number) {
      comments(first: 100, after: $after) {
        ${PAGE}
        nodes { id body createdAt url ${AUTHOR} }
      }
    }
  }
}`;

export const REVIEWS = `
query Reviews($owner: String!, $name: String!, $number: Int!, $after: String) {
  repository(owner: $owner, name: $name) {
    pullRequest(number: $number) {
      reviews(first: 100, after: $after) {
        ${PAGE}
        nodes { id state body createdAt url viewerDidAuthor ${AUTHOR} }
      }
    }
  }
}`;

export const FILES = `
query Files($owner: String!, $name: String!, $number: Int!, $after: String) {
  repository(owner: $owner, name: $name) {
    pullRequest(number: $number) {
      files(first: 100, after: $after) {
        ${PAGE}
        nodes { path }
      }
    }
  }
}`;

export const ADD_REVIEW = `
mutation AddReview($pullRequestId: ID!, $commit: GitObjectID, $body: String!) {
  addPullRequestReview(input: { pullRequestId: $pullRequestId, commitOID: $commit, body: $body }) {
    pullRequestReview { id }
  }
}`;

export const ADD_THREAD = `
mutation AddThread($reviewId: ID!, $path: String!, $body: String!, $line: Int!, $startLine: Int) {
  addPullRequestReviewThread(
    input: { pullRequestReviewId: $reviewId, path: $path, body: $body, line: $line, side: RIGHT, startLine: $startLine, subjectType: LINE }
  ) {
    thread { ...ReviewThread }
  }
}
${THREAD}
${COMMENT}`;

export const ADD_REPLY = `
mutation AddReply($threadId: ID!, $body: String!, $reviewId: ID) {
  addPullRequestReviewThreadReply(
    input: { pullRequestReviewThreadId: $threadId, body: $body, pullRequestReviewId: $reviewId }
  ) {
    comment { ...ReviewComment }
  }
}
${COMMENT}`;

export const DELETE_COMMENT = `
mutation DeleteComment($id: ID!) {
  deletePullRequestReviewComment(input: { id: $id }) { clientMutationId }
}`;

export const RESOLVE = `
mutation Resolve($threadId: ID!) {
  resolveReviewThread(input: { threadId: $threadId }) {
    thread { id isResolved viewerCanResolve viewerCanUnresolve }
  }
}`;

export const UNRESOLVE = `
mutation Unresolve($threadId: ID!) {
  unresolveReviewThread(input: { threadId: $threadId }) {
    thread { id isResolved viewerCanResolve viewerCanUnresolve }
  }
}`;

export const UPDATE_REVIEW = `
mutation UpdateReview($reviewId: ID!, $body: String!) {
  updatePullRequestReview(input: { pullRequestReviewId: $reviewId, body: $body }) {
    pullRequestReview { id }
  }
}`;

export const SUBMIT_REVIEW = `
mutation SubmitReview($reviewId: ID!, $event: PullRequestReviewEvent!, $body: String) {
  submitPullRequestReview(input: { pullRequestReviewId: $reviewId, event: $event, body: $body }) {
    pullRequestReview { id state }
  }
}`;

export const DELETE_REVIEW = `
mutation DeleteReview($reviewId: ID!) {
  deletePullRequestReview(input: { pullRequestReviewId: $reviewId }) { clientMutationId }
}`;

export const VIEWER = `
query Viewer {
  viewer { login }
}`;
