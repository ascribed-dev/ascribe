# The same checks in CI

The editor reports what `ascribe check` reports. Run it in the project's folder, locally or in CI:

```sh
npx ascribe check
```

It fails when a page has an error, such as a link to a page that doesn't exist, so a pull request that adds one fails before it's merged.
