- A widget with container form MUST NOT have a required primary, since a container opener's primary is empty ([SPEC §3.5]({repo}/blob/main/SPEC.md#35-forms)). A container-only widget's primary MUST be `"none"` (`model-widget-container-primary`).
- A groupable widget MUST be container-only, since group arms are containers ([SPEC §3.6]({repo}/blob/main/SPEC.md#36-groups)) (`model-widget-groupable-form`).
- `binding` is required with line form and not allowed without it (`model-widget-binding`).

The site output's element for a widget (tag name and attributes) is defined by the [element contract]({repo}/blob/main/packages/elements/CONTRACT.md), not here.
