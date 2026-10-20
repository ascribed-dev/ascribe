@memo: Generated from crates/ascribe-model/settings.toml by tests/conformance/tests/docs.rs; edit the registry, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-conformance --test docs` to rewrite it.
| Key | Type | Default | Description |
|---|---|---|---|
| `forms` | array of strings | **required** | The permitted forms ([SPEC §3.5]({repo}/blob/main/SPEC.md#35-forms)): `["line"]`, `["container"]`, or both. |
| `primary` | string | `"none"` | The primary ([SPEC §3.4]({repo}/blob/main/SPEC.md#34-the-primary)): `"none"`; `"identifier"` or `"text"`, which are required; or `"identifier?"` or `"text?"`, which are optional. |
| `binding` | string | required when `forms` includes `"line"`; not allowed otherwise | What the line form applies to ([SPEC §3.8]({repo}/blob/main/SPEC.md#38-binding)), one of the values below. |
| `title` | string | `"none"` | Whether the widget takes a title line ([SPEC §3.7]({repo}/blob/main/SPEC.md#37-titles)): `"none"`, `"accepted"`, or `"required"`. |
| `groupable` | boolean | `false` | Whether a run of the widget's openers forms a group of arms ([SPEC §3.6]({repo}/blob/main/SPEC.md#36-groups)). |
| `attributes` | table of attribute types | `{}` | The attribute schema: keys (`key` rule) and types (§5). Declaration order is canonical order ([SPEC §8.3]({repo}/blob/main/SPEC.md#83-canonical-form)). The keys `heading` and `primary`, HTML's global attributes (including `title`), `aria-` keys, and event-handler attributes such as `onclick`, are rejected (`model-attribute-reserved`): the site output writes these attributes onto the widget's element ([SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)). |
| `plain-fallback` | string | none | Plain-text fallback for the plain-markdown output (SPEC §6, §9.4): CommonMark text written in place of the widget. Phrases in it are substituted. It isn't a template: attribute values aren't inserted. Without it, the widget itself emits nothing. |
| `plain-content` | string | `"keep"` | For a widget that wraps content, whether the plain-markdown output keeps that content (`"keep"`) after the fallback, or drops it (`"drop"`). Allowed only when the widget wraps content: it has container form, or its binding is `block` or `heading-or-block`. |
| `description` | string | none | Help text for the editor's hover and completion. |
