# A project

A project is a folder with an `ascribe.toml`. The pages it describes are Markdown files under its content root, `docs/` unless it says otherwise:

```text
my-docs/
├── ascribe.toml
└── docs/
    ├── index.md
    └── install.md
```

A workspace can hold several projects, such as a site's and a handbook's. Each page belongs to the nearest `ascribe.toml` above it.
