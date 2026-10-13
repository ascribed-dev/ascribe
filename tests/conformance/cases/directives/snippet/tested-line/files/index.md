An example its observation lists as it is:

@snippet: code:examples/hello.ts

With a title:

@snippet {title="Say hello"}: code:examples/hello.ts

An example checked out with CRLF line endings, observed with LF:

@snippet: code:examples/crlf.ts

An example changed since it was observed:

@snippet: code:examples/edited.ts

A file the config's `ignore` matches, and one its `include` doesn't:

@snippet: code:examples/scratch.ts

@snippet: code:helper.ts
