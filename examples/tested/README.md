# A tested example

A docs project, `docs/`, whose page shows a TypeScript example and the output it prints, from `code/`, a folder beside it. `code/ascribe.examples.toml` says the files in `code/examples/` are examples, and that `node --test test/greet.test.mjs` tests them: it runs the example and compares what it prints with `greet.out.txt`.

```sh
cd code && ascribe observe                  # runs the test, and writes examples.observations/default.toml
ascribe observe --name node-22              # the same, under Node 22: examples.observations/node-22.toml
cd .. && ascribe build                      # the page's code blocks say "Tested with Node 22… and 24… on Linux"
```

Edit `code/examples/greet.ts` without running `ascribe observe` again, and `ascribe check` reports `example-stale` at the `@snippet` line. CI runs `ascribe observe` here on Linux, macOS, and Windows, once with the Node in `.nvmrc` and once with Node 22, and compares each run's `[files]` with the committed observation's. See [Show what an example was tested with](https://ascribed-dev.com/docs/examples/tested/).
