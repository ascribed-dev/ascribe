```toml
spec = "0.1"
```

| Key | Type | Default | Description |
|---|---|---|---|
| `spec` | string | **required** | The version of the Ascribe specification this project targets (SPEC §11). It MUST be quoted: `spec = 0.1` is a TOML float and is an error. A processor accepts only the spec versions it implements, compared as exact strings; this reference defines `"0.1"`. |

The only other top-level keys are the tables in §3–§20. Anything else is an unknown key.
