//! Slugs for every diagnostic in the registry.
//!
//! One constant per entry of `tests/conformance/diagnostics.toml`, in the
//! same order, named after the slug. A crate that finds a problem reports it
//! with one of these ([`Issue::new`](crate::Issue::new)); since
//! [`DiagnosticSlug`] can't be constructed outside this crate, a misspelled or
//! unregistered slug is a compile error. A test keeps this list equal to the
//! registry, so adding a diagnostic means adding it in both places.
//!
//! The registry holds each diagnostic's code, severity, level, SPEC section,
//! and message templates; the checks read them from there.

use crate::DiagnosticSlug;

/// `ASC001`, error, file level: SPEC §8.2, "Attributes | Unknown key for the directive or image".
pub const ATTRIBUTE_UNKNOWN_KEY: DiagnosticSlug = DiagnosticSlug("attribute-unknown-key");

/// `ASC002`, error, file level: SPEC §8.2, "Attributes | Value doesn't match the key's declared type".
pub const ATTRIBUTE_TYPE_MISMATCH: DiagnosticSlug = DiagnosticSlug("attribute-type-mismatch");

/// `ASC003`, error, file level: SPEC §8.2, "Attributes | Bare key without a value".
pub const ATTRIBUTE_BARE_KEY: DiagnosticSlug = DiagnosticSlug("attribute-bare-key");

/// `ASC004`, error, file level: SPEC §8.2, "Attributes | Unquoted value containing a reserved character".
pub const ATTRIBUTE_UNQUOTED_RESERVED: DiagnosticSlug =
    DiagnosticSlug("attribute-unquoted-reserved");

/// `ASC005`, warning, file level: SPEC §8.2, "Directives | Directive-shaped line (`@word` followed by `{`, `:`, or end of line) with an unknown name".
pub const DIRECTIVE_UNKNOWN: DiagnosticSlug = DiagnosticSlug("directive-unknown");

/// `ASC006`, error, file level: SPEC §8.2, "Directives | Primary given to a directive that takes none, or a required primary missing".
pub const DIRECTIVE_PRIMARY: DiagnosticSlug = DiagnosticSlug("directive-primary");

/// `ASC007`, error, file level: SPEC §8.2, "Container | Container not closed before its enclosing block ends".
pub const CONTAINER_UNCLOSED: DiagnosticSlug = DiagnosticSlug("container-unclosed");

/// `ASC008`, error, file level: SPEC §8.2, "Container | Trailing `:` on a directive with no container form".
pub const CONTAINER_COLON_UNEXPECTED: DiagnosticSlug = DiagnosticSlug("container-colon-unexpected");

/// `ASC009`, error, file level: SPEC §8.2, "Container | Container-only directive without a trailing `:`".
pub const CONTAINER_COLON_MISSING: DiagnosticSlug = DiagnosticSlug("container-colon-missing");

/// `ASC010`, error, file level: SPEC §8.2, "Container | Container still open when the next arm of its group begins (reported at that arm's opener)".
pub const CONTAINER_OPEN_AT_ARM: DiagnosticSlug = DiagnosticSlug("container-open-at-arm");

/// `ASC011`, error, file level: SPEC §8.2, "Container | End line with no open container".
pub const END_UNMATCHED: DiagnosticSlug = DiagnosticSlug("end-unmatched");

/// `ASC012`, error, file level: SPEC §8.2, "Container | End line indented differently from its opener".
pub const END_INDENT_MISMATCH: DiagnosticSlug = DiagnosticSlug("end-indent-mismatch");

/// `ASC013`, warning, file level: SPEC §8.2, "Container | Nesting deeper than two levels".
pub const CONTAINER_NESTING_DEEP: DiagnosticSlug = DiagnosticSlug("container-nesting-deep");

/// `ASC014`, error, file level: SPEC §8.2, "Binding | Following-block directive with no following block in its container".
pub const BINDING_NO_BLOCK: DiagnosticSlug = DiagnosticSlug("binding-no-block");

/// `ASC015`, error, file level: SPEC §8.2, "Binding | Following-block directive bound to a heading".
pub const BINDING_HEADING: DiagnosticSlug = DiagnosticSlug("binding-heading");

/// `ASC016`, warning, file level: SPEC §8.2, "Binding | Blank line between a following-block directive and its block".
pub const BINDING_BLANK_LINE: DiagnosticSlug = DiagnosticSlug("binding-blank-line");

/// `ASC017`, error, file level: SPEC §8.2, "Binding | Heading-bound directive that isn't at the top of its section".
pub const BINDING_NOT_SECTION_TOP: DiagnosticSlug = DiagnosticSlug("binding-not-section-top");

/// `ASC018`, warning, file level: SPEC §8.2, "Title | Title given to a directive that doesn't accept one".
pub const TITLE_NOT_ACCEPTED: DiagnosticSlug = DiagnosticSlug("title-not-accepted");

/// `ASC019`, warning, file level: SPEC §8.2, "Title | A `. ` line (dot and space) directly above a directive that accepts a title".
pub const TITLE_DOT_SPACE: DiagnosticSlug = DiagnosticSlug("title-dot-space");

/// `ASC020`, error, page level: SPEC §8.2, "`@id` | Duplicate id on a page, including ids from included content (page level)".
pub const ID_DUPLICATE: DiagnosticSlug = DiagnosticSlug("id-duplicate");

/// `ASC021`, error, file level: SPEC §8.2, "`@include` | Target file doesn't exist".
pub const INCLUDE_TARGET_MISSING: DiagnosticSlug = DiagnosticSlug("include-target-missing");

/// `ASC022`, error, page level: SPEC §8.2, "`@include` | Target id doesn't exist in the target file (page level)".
pub const INCLUDE_ID_MISSING: DiagnosticSlug = DiagnosticSlug("include-id-missing");

/// `ASC023`, error, page level: SPEC §8.2, "`@include` | Include cycle".
pub const INCLUDE_CYCLE: DiagnosticSlug = DiagnosticSlug("include-cycle");

/// `ASC024`, warning, page level: SPEC §8.2, "`@variant` | No arm of a group survives a build's selection (page level)".
pub const VARIANT_NO_ARM_SURVIVES: DiagnosticSlug = DiagnosticSlug("variant-no-arm-survives");

/// `ASC025`, error, file level: SPEC §8.2, "`@variant` | Unknown dimension or value".
pub const VARIANT_UNKNOWN: DiagnosticSlug = DiagnosticSlug("variant-unknown");

/// `ASC026`, error, file level: SPEC §8.2, "`@variant` | Group mixes labeled and dimensional arms".
pub const VARIANT_MIXED_ARMS: DiagnosticSlug = DiagnosticSlug("variant-mixed-arms");

/// `ASC027`, error, file level: SPEC §8.2, "`@variant` | Arm has both a title and attributes, or neither".
pub const VARIANT_ARM_KIND: DiagnosticSlug = DiagnosticSlug("variant-arm-kind");

/// `ASC028`, error, file level: SPEC §8.2, "`@variant` | Dimensional arms share no dimension key".
pub const VARIANT_NO_SHARED_DIMENSION: DiagnosticSlug =
    DiagnosticSlug("variant-no-shared-dimension");

/// `ASC029`, error, file level: SPEC §8.2, "`@available` | Unknown target or state".
pub const AVAILABLE_UNKNOWN: DiagnosticSlug = DiagnosticSlug("available-unknown");

/// `ASC030`, error, file level: SPEC §8.2, "`@available` | History out of chronological order".
pub const AVAILABLE_HISTORY_ORDER: DiagnosticSlug = DiagnosticSlug("available-history-order");

/// `ASC031`, error, file level: SPEC §8.2, "`@available` | Versions given for a versionless target".
pub const AVAILABLE_VERSIONLESS: DiagnosticSlug = DiagnosticSlug("available-versionless");

/// `ASC032`, error, page level: SPEC §8.2, "`@available` | Spec exceeds its enclosing scope".
pub const AVAILABLE_EXCEEDS_SCOPE: DiagnosticSlug = DiagnosticSlug("available-exceeds-scope");

/// `ASC033`, error, file level: SPEC §8.2, "`@steps` | Bound block isn't an ordered list".
pub const STEPS_NOT_ORDERED_LIST: DiagnosticSlug = DiagnosticSlug("steps-not-ordered-list");

/// `ASC034`, error, file level: SPEC §8.2, "`@details` | Missing title".
pub const DETAILS_TITLE_MISSING: DiagnosticSlug = DiagnosticSlug("details-title-missing");

/// `ASC035`, error, file level: SPEC §8.2, "Project widget | Violates its declared schema".
pub const WIDGET_SCHEMA: DiagnosticSlug = DiagnosticSlug("widget-schema");

/// `ASC036`, error, file level: SPEC §8.2, "Links | Target file doesn't exist".
pub const LINK_TARGET_MISSING: DiagnosticSlug = DiagnosticSlug("link-target-missing");

/// `ASC037`, error, page level: SPEC §8.2, "Links | Target id doesn't exist in the target file (page level)".
pub const LINK_ID_MISSING: DiagnosticSlug = DiagnosticSlug("link-id-missing");

/// `ASC038`, error, file level: SPEC §8.2, "Links | Target is a fragment".
pub const LINK_TO_FRAGMENT: DiagnosticSlug = DiagnosticSlug("link-to-fragment");

/// `ASC039`, retired: a link can name a heading from a fragment the target page includes (SPEC §4.2).
pub const LINK_ID_IN_FRAGMENT: DiagnosticSlug = DiagnosticSlug("link-id-in-fragment");

/// `ASC040`, error, page level: SPEC §8.2, "Links | Target id is removed by a build (page level, per build)".
pub const LINK_ID_REMOVED: DiagnosticSlug = DiagnosticSlug("link-id-removed");

/// `ASC041`, warning, file level: SPEC §8.2, "Links | Destination is a route rather than a file path".
pub const LINK_ROUTE: DiagnosticSlug = DiagnosticSlug("link-route");

/// `ASC042`, error, file level: SPEC §8.2, "Images | Local source doesn't exist".
pub const IMAGE_SOURCE_MISSING: DiagnosticSlug = DiagnosticSlug("image-source-missing");

/// `ASC043`, warning, file level: SPEC §8.2, "Images | Missing alt text".
pub const IMAGE_ALT_MISSING: DiagnosticSlug = DiagnosticSlug("image-alt-missing");

/// `ASC044`, warning, file level: SPEC §8.2, "Phrases | `{key}` in prose whose key isn't declared".
pub const PHRASE_UNDECLARED: DiagnosticSlug = DiagnosticSlug("phrase-undeclared");

/// `ASC045`, warning, file level: SPEC §8.2, "Headings | No `@id`, and the heading contains a phrase".
pub const HEADING_PHRASE_WITHOUT_ID: DiagnosticSlug = DiagnosticSlug("heading-phrase-without-id");

/// `ASC046`, warning, page level: SPEC §8.2, "Headings | No `@id`, and the heading duplicates another heading's text on the page (page level)".
pub const HEADING_DUPLICATE_WITHOUT_ID: DiagnosticSlug =
    DiagnosticSlug("heading-duplicate-without-id");

/// `ASC047`, error, file level: SPEC §8.2, "Frontmatter | Key the file's content type or the fragment schema doesn't declare, other than a reserved key on a page".
pub const FRONTMATTER_UNKNOWN_KEY: DiagnosticSlug = DiagnosticSlug("frontmatter-unknown-key");

/// `ASC048`, error, file level: SPEC §8.2, "Frontmatter | Required field missing".
pub const FRONTMATTER_MISSING_FIELD: DiagnosticSlug = DiagnosticSlug("frontmatter-missing-field");

/// `ASC049`, error, file level: SPEC §8.2, "Frontmatter | Value doesn't match the field's declared type".
pub const FRONTMATTER_TYPE_MISMATCH: DiagnosticSlug = DiagnosticSlug("frontmatter-type-mismatch");

/// `ASC050`, error, file level: SPEC §8.2, "Frontmatter | Reserved key (`available`, `variant`) on a fragment".
pub const FRONTMATTER_RESERVED_IN_FRAGMENT: DiagnosticSlug =
    DiagnosticSlug("frontmatter-reserved-in-fragment");

/// `ASC051`, error, file level: SPEC §8.2, "Frontmatter | Page matches more than one content type, or matches none and there's no default type".
pub const CONTENT_TYPE_UNRESOLVED: DiagnosticSlug = DiagnosticSlug("content-type-unresolved");

/// `ASC052`, warning, file level: SPEC §8.2, "Lists | Unindented directive line ends a list".
pub const LIST_ENDED_BY_DIRECTIVE: DiagnosticSlug = DiagnosticSlug("list-ended-by-directive");

/// `ASC053`, warning, file level: SPEC §8.2, "Lists | Directive line over-indented into an indented code block".
pub const DIRECTIVE_INDENTED_CODE: DiagnosticSlug = DiagnosticSlug("directive-indented-code");

/// `ASC054`, warning, file level: SPEC §8.2, "Lists | An ordered list continues the numbering of a list bound by `@steps` right after it ends (usually an unindented directive split the list)".
pub const STEPS_NUMBERING_CONTINUED: DiagnosticSlug = DiagnosticSlug("steps-numbering-continued");

/// `ASC055`, error, file level: SPEC §8.2, "Attributes | Attribute block that doesn't parse (such as an unclosed quote or brace, or `=` with no value)".
pub const ATTRIBUTE_SYNTAX: DiagnosticSlug = DiagnosticSlug("attribute-syntax");

/// `ASC056`, error, file level: SPEC §8.2, "Attributes | The same key given more than once".
pub const ATTRIBUTE_DUPLICATE_KEY: DiagnosticSlug = DiagnosticSlug("attribute-duplicate-key");

/// `ASC057`, error, file level: SPEC §8.2, "`@available` | Spec that doesn't parse, in a directive or in `available` frontmatter".
pub const AVAILABLE_SYNTAX: DiagnosticSlug = DiagnosticSlug("available-syntax");

/// `ASC058`, error, file level: SPEC §8.2, "`@id` | Id containing characters other than letters, digits, hyphens, underscores, and periods".
pub const ID_INVALID: DiagnosticSlug = DiagnosticSlug("id-invalid");

/// `ASC059`, error, file level: SPEC §8.2, "Images | Required image attribute missing".
pub const IMAGE_ATTRIBUTE_MISSING: DiagnosticSlug = DiagnosticSlug("image-attribute-missing");

/// `ASC060`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_TOML_SYNTAX: DiagnosticSlug = DiagnosticSlug("model-toml-syntax");

/// `ASC061`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_UNKNOWN_KEY: DiagnosticSlug = DiagnosticSlug("model-unknown-key");

/// `ASC062`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_MISSING_KEY: DiagnosticSlug = DiagnosticSlug("model-missing-key");

/// `ASC063`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_WRONG_TYPE: DiagnosticSlug = DiagnosticSlug("model-wrong-type");

/// `ASC064`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_INVALID_VALUE: DiagnosticSlug = DiagnosticSlug("model-invalid-value");

/// `ASC065`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_SPEC_UNSUPPORTED: DiagnosticSlug = DiagnosticSlug("model-spec-unsupported");

/// `ASC066`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_INVALID_NAME: DiagnosticSlug = DiagnosticSlug("model-invalid-name");

/// `ASC067`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_EMPTY_TEXT: DiagnosticSlug = DiagnosticSlug("model-empty-text");

/// `ASC068`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_PATH_ABSOLUTE: DiagnosticSlug = DiagnosticSlug("model-path-absolute");

/// `ASC069`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_CONTENT_ROOT_MISSING: DiagnosticSlug = DiagnosticSlug("model-content-root-missing");

/// `ASC070`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_OUTPUT_OVERLAPS_CONTENT: DiagnosticSlug =
    DiagnosticSlug("model-output-overlaps-content");

/// `ASC071`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_TYPE_MULTIPLE_DEFAULTS: DiagnosticSlug =
    DiagnosticSlug("model-type-multiple-defaults");

/// `ASC072`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_TYPE_UNREACHABLE: DiagnosticSlug = DiagnosticSlug("model-type-unreachable");

/// `ASC073`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_TYPE_TITLE: DiagnosticSlug = DiagnosticSlug("model-type-title");

/// `ASC074`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_FIELD_RESERVED: DiagnosticSlug = DiagnosticSlug("model-field-reserved");

/// `ASC075`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_TYPE_SYNTAX: DiagnosticSlug = DiagnosticSlug("model-type-syntax");

/// `ASC076`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_TYPE_FIELDS: DiagnosticSlug = DiagnosticSlug("model-type-fields");

/// `ASC077`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_ENUM_VALUES: DiagnosticSlug = DiagnosticSlug("model-enum-values");

/// `ASC078`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_SET_TOKEN: DiagnosticSlug = DiagnosticSlug("model-set-token");

/// `ASC079`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_DEFAULT_TYPE: DiagnosticSlug = DiagnosticSlug("model-default-type");

/// `ASC080`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_PHRASES_FIELD_TYPE: DiagnosticSlug = DiagnosticSlug("model-phrases-field-type");

/// `ASC081`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_PATTERN_SYNTAX: DiagnosticSlug = DiagnosticSlug("model-pattern-syntax");

/// `ASC082`, error, file level: SPEC §8.2, "Content model | A name used in more than one role (dimension name, dimension value, lifecycle state, or feature key), or a dimension value in more than one dimension".
pub const MODEL_NAME_MULTIPLE_ROLES: DiagnosticSlug = DiagnosticSlug("model-name-multiple-roles");

/// `ASC083`, warning, file level: a rule for loading `ascribe.toml`.
pub const MODEL_NAME_CASE: DiagnosticSlug = DiagnosticSlug("model-name-case");

/// `ASC084`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_DIMENSION_EMPTY: DiagnosticSlug = DiagnosticSlug("model-dimension-empty");

/// `ASC085`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_DIMENSION_VALUE_DUPLICATE: DiagnosticSlug =
    DiagnosticSlug("model-dimension-value-duplicate");

/// `ASC086`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_DIMENSION_VALUE_SHARED: DiagnosticSlug =
    DiagnosticSlug("model-dimension-value-shared");

/// `ASC087`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_LABEL_UNDECLARED: DiagnosticSlug = DiagnosticSlug("model-label-undeclared");

/// `ASC088`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_VERSIONLESS_UNDECLARED: DiagnosticSlug =
    DiagnosticSlug("model-versionless-undeclared");

/// `ASC089`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_LIFECYCLE_AVAILABLE_REQUIRED: DiagnosticSlug =
    DiagnosticSlug("model-lifecycle-available-required");

/// `ASC090`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_LIFECYCLE_GA_UNAVAILABLE: DiagnosticSlug =
    DiagnosticSlug("model-lifecycle-ga-unavailable");

/// `ASC091`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_NOTE_LABEL_REQUIRED: DiagnosticSlug = DiagnosticSlug("model-note-label-required");

/// `ASC092`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_AVAILABILITY_SYNTAX: DiagnosticSlug = DiagnosticSlug("model-availability-syntax");

/// `ASC093`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_AVAILABILITY_UNKNOWN_NAME: DiagnosticSlug =
    DiagnosticSlug("model-availability-unknown-name");

/// `ASC094`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_AVAILABILITY_VERSIONLESS: DiagnosticSlug =
    DiagnosticSlug("model-availability-versionless");

/// `ASC095`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_AVAILABILITY_HISTORY_ORDER: DiagnosticSlug =
    DiagnosticSlug("model-availability-history-order");

/// `ASC096`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_FEATURE_NESTED: DiagnosticSlug = DiagnosticSlug("model-feature-nested");

/// `ASC097`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_PHRASE_VALUE_TYPE: DiagnosticSlug = DiagnosticSlug("model-phrase-value-type");

/// `ASC098`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_GLOSSARY_DUPLICATE_TERM: DiagnosticSlug =
    DiagnosticSlug("model-glossary-duplicate-term");

/// `ASC099`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_GLOSSARY_LINK: DiagnosticSlug = DiagnosticSlug("model-glossary-link");

/// `ASC100`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_WIDGET_RESERVED_NAME: DiagnosticSlug = DiagnosticSlug("model-widget-reserved-name");

/// `ASC101`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_WIDGET_FORMS: DiagnosticSlug = DiagnosticSlug("model-widget-forms");

/// `ASC102`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_WIDGET_BINDING: DiagnosticSlug = DiagnosticSlug("model-widget-binding");

/// `ASC103`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_WIDGET_CONTAINER_PRIMARY: DiagnosticSlug =
    DiagnosticSlug("model-widget-container-primary");

/// `ASC104`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_WIDGET_GROUPABLE_FORM: DiagnosticSlug =
    DiagnosticSlug("model-widget-groupable-form");

/// `ASC105`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_WIDGET_PLAIN_CONTENT: DiagnosticSlug = DiagnosticSlug("model-widget-plain-content");

/// `ASC106`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_CONSUMER_UNSUPPORTED: DiagnosticSlug = DiagnosticSlug("model-consumer-unsupported");

/// `ASC107`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_CONSUMER_SITE: DiagnosticSlug = DiagnosticSlug("model-consumer-site");

/// `ASC108`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_CONSUMER_BASE_PATH: DiagnosticSlug = DiagnosticSlug("model-consumer-base-path");

/// `ASC109`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_BUILD_NAME_CASE: DiagnosticSlug = DiagnosticSlug("model-build-name-case");

/// `ASC110`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_BUILD_VARIANTS: DiagnosticSlug = DiagnosticSlug("model-build-variants");

/// `ASC111`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_BUILD_UNKNOWN_DIMENSION: DiagnosticSlug =
    DiagnosticSlug("model-build-unknown-dimension");

/// `ASC112`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_BUILD_UNKNOWN_VALUE: DiagnosticSlug = DiagnosticSlug("model-build-unknown-value");

/// `ASC113`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_BUILD_AVAILABILITY: DiagnosticSlug = DiagnosticSlug("model-build-availability");

/// `ASC114`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_BUILD_FILTER_TARGET: DiagnosticSlug = DiagnosticSlug("model-build-filter-target");

/// `ASC115`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_BUILD_FILTER_VERSION: DiagnosticSlug = DiagnosticSlug("model-build-filter-version");

/// `ASC116`, warning, file level: a rule for loading `ascribe.toml`.
pub const MODEL_BUILD_FILTER_EXCLUDED: DiagnosticSlug =
    DiagnosticSlug("model-build-filter-excluded");

/// `ASC117`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_EDITOR_BUILD_UNKNOWN: DiagnosticSlug = DiagnosticSlug("model-editor-build-unknown");

/// `ASC118`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_EDITOR_BUILD_REQUIRED: DiagnosticSlug =
    DiagnosticSlug("model-editor-build-required");

/// `ASC119`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_ATTRIBUTE_RESERVED: DiagnosticSlug = DiagnosticSlug("model-attribute-reserved");

/// `ASC120`, error, file level: SPEC §8.2, "Directives | Text on a directive line that fits no part of it: after the name or attributes, after `@end`, or after an identifier primary".
pub const DIRECTIVE_EXTRA_TEXT: DiagnosticSlug = DiagnosticSlug("directive-extra-text");

/// `ASC121`, error, page level: SPEC §8.2, "Links | Target page isn't published by a build (page level, per build)".
pub const LINK_PAGE_DROPPED: DiagnosticSlug = DiagnosticSlug("link-page-dropped");

/// `ASC122`, error, file level: SPEC §8.2, "Frontmatter | Frontmatter that isn't valid YAML".
pub const FRONTMATTER_SYNTAX: DiagnosticSlug = DiagnosticSlug("frontmatter-syntax");

/// `ASC123`, error, file level: SPEC §8.2, "Files | Source file that can't be read, or isn't valid UTF-8".
pub const SOURCE_UNREADABLE: DiagnosticSlug = DiagnosticSlug("source-unreadable");

/// `ASC124`, warning, file level: SPEC §8.2, "Headings | No `@id`, and the heading's slug is empty (its text is only punctuation or emoji)".
pub const HEADING_EMPTY_SLUG: DiagnosticSlug = DiagnosticSlug("heading-empty-slug");

/// `ASC125`, warning, file level: SPEC §8.2, "`@include` | `{heading=false}` without an `#id`, which has no effect".
pub const INCLUDE_HEADING_WITHOUT_ID: DiagnosticSlug = DiagnosticSlug("include-heading-without-id");

/// `ASC126`, warning, file level: SPEC §8.2, "Phrases | A declared `{key}` directly between two more braces (`{{key}}`), usually a substitution left over from another tool".
pub const PHRASE_DOUBLE_BRACES: DiagnosticSlug = DiagnosticSlug("phrase-double-braces");

/// `ASC127`, error, file level: SPEC §8.2, "`@snippet` | Address that isn't `<source>:<path>`, optionally with `#<region>`".
pub const SNIPPET_ADDRESS: DiagnosticSlug = DiagnosticSlug("snippet-address");

/// `ASC128`, error, file level: SPEC §8.2, "`@snippet` | Source the content model doesn't declare".
pub const SNIPPET_SOURCE_UNKNOWN: DiagnosticSlug = DiagnosticSlug("snippet-source-unknown");

/// `ASC129`, error, file level: SPEC §8.2, "`@snippet` | File doesn't exist, or its source doesn't include it".
pub const SNIPPET_FILE_MISSING: DiagnosticSlug = DiagnosticSlug("snippet-file-missing");

/// `ASC130`, error, file level: SPEC §8.2, "`@snippet` | File isn't text".
pub const SNIPPET_FILE_NOT_TEXT: DiagnosticSlug = DiagnosticSlug("snippet-file-not-text");

/// `ASC131`, error, file level: SPEC §8.2, "`@snippet` | Region doesn't exist in the file".
pub const SNIPPET_REGION_MISSING: DiagnosticSlug = DiagnosticSlug("snippet-region-missing");

/// `ASC132`, error, file level: SPEC §8.2, "`@snippet` | The file's tags are unbalanced, name a region twice, or use a reserved tag".
pub const SNIPPET_TAGS: DiagnosticSlug = DiagnosticSlug("snippet-tags");

/// `ASC133`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_SOURCE_PATH_MISSING: DiagnosticSlug = DiagnosticSlug("model-source-path-missing");

/// `ASC134`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_SOURCE_OUTSIDE_REPOSITORY: DiagnosticSlug =
    DiagnosticSlug("model-source-outside-repository");

/// `ASC135`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_SOURCE_REMOTE: DiagnosticSlug = DiagnosticSlug("model-source-remote");

/// `ASC136`, error, file level: SPEC §8.2, "Source copies | `ascribe.lock` isn't valid TOML, or doesn't have the shape §7.4 gives it".
pub const LOCK_INVALID: DiagnosticSlug = DiagnosticSlug("lock-invalid");

/// `ASC137`, error, file level: SPEC §8.2, "Source copies | `ascribe.lock` pins a source the content model doesn't declare with `git`, or pins it with another `git`".
pub const LOCK_SOURCE_UNKNOWN: DiagnosticSlug = DiagnosticSlug("lock-source-unknown");

/// `ASC138`, error, file level: SPEC §8.2, "Source copies | A file `ascribe.lock` lists is missing from the source's folder, or its hash differs".
pub const SOURCE_COPY_CHANGED: DiagnosticSlug = DiagnosticSlug("source-copy-changed");

/// `ASC139`, error, file level: SPEC §8.2, "Source copies | A file in a source's copies folder that `ascribe.lock` doesn't list".
pub const SOURCE_COPY_UNLOCKED: DiagnosticSlug = DiagnosticSlug("source-copy-unlocked");

/// `ASC140`, warning, file level: SPEC §8.2, "Source copies | A copy that no snippet uses".
pub const SOURCE_COPY_UNUSED: DiagnosticSlug = DiagnosticSlug("source-copy-unused");

/// `ASC141`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_INLINE_FIELD: DiagnosticSlug = DiagnosticSlug("model-inline-field");

/// `ASC142`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_CHECK_NOT_CONFIGURABLE: DiagnosticSlug =
    DiagnosticSlug("model-check-not-configurable");

/// `ASC143`, advice, page level: a content check.
pub const PAGE_ORPHAN: DiagnosticSlug = DiagnosticSlug("page-orphan");

/// `ASC144`, advice, file level: a content check.
pub const FRAGMENT_UNUSED: DiagnosticSlug = DiagnosticSlug("fragment-unused");

/// `ASC145`, advice, file level: a content check.
pub const PHRASE_UNUSED: DiagnosticSlug = DiagnosticSlug("phrase-unused");

/// `ASC146`, advice, file level: a content check.
pub const FEATURE_UNUSED: DiagnosticSlug = DiagnosticSlug("feature-unused");

/// `ASC147`, advice, file level: a content check.
pub const GLOSSARY_TERM_UNUSED: DiagnosticSlug = DiagnosticSlug("glossary-term-unused");

/// `ASC148`, advice, file level: a content check.
pub const IMAGE_UNUSED: DiagnosticSlug = DiagnosticSlug("image-unused");

/// `ASC149`, advice, file level: a content check.
pub const IMAGE_LARGE: DiagnosticSlug = DiagnosticSlug("image-large");

/// `ASC150`, advice, page level: a content check.
pub const TITLE_DUPLICATE: DiagnosticSlug = DiagnosticSlug("title-duplicate");

/// Every slug, in registry order.
pub const ALL: &[DiagnosticSlug] = &[
    ATTRIBUTE_UNKNOWN_KEY,
    ATTRIBUTE_TYPE_MISMATCH,
    ATTRIBUTE_BARE_KEY,
    ATTRIBUTE_UNQUOTED_RESERVED,
    DIRECTIVE_UNKNOWN,
    DIRECTIVE_PRIMARY,
    CONTAINER_UNCLOSED,
    CONTAINER_COLON_UNEXPECTED,
    CONTAINER_COLON_MISSING,
    CONTAINER_OPEN_AT_ARM,
    END_UNMATCHED,
    END_INDENT_MISMATCH,
    CONTAINER_NESTING_DEEP,
    BINDING_NO_BLOCK,
    BINDING_HEADING,
    BINDING_BLANK_LINE,
    BINDING_NOT_SECTION_TOP,
    TITLE_NOT_ACCEPTED,
    TITLE_DOT_SPACE,
    ID_DUPLICATE,
    INCLUDE_TARGET_MISSING,
    INCLUDE_ID_MISSING,
    INCLUDE_CYCLE,
    VARIANT_NO_ARM_SURVIVES,
    VARIANT_UNKNOWN,
    VARIANT_MIXED_ARMS,
    VARIANT_ARM_KIND,
    VARIANT_NO_SHARED_DIMENSION,
    AVAILABLE_UNKNOWN,
    AVAILABLE_HISTORY_ORDER,
    AVAILABLE_VERSIONLESS,
    AVAILABLE_EXCEEDS_SCOPE,
    STEPS_NOT_ORDERED_LIST,
    DETAILS_TITLE_MISSING,
    WIDGET_SCHEMA,
    LINK_TARGET_MISSING,
    LINK_ID_MISSING,
    LINK_TO_FRAGMENT,
    LINK_ID_IN_FRAGMENT,
    LINK_ID_REMOVED,
    LINK_ROUTE,
    IMAGE_SOURCE_MISSING,
    IMAGE_ALT_MISSING,
    PHRASE_UNDECLARED,
    HEADING_PHRASE_WITHOUT_ID,
    HEADING_DUPLICATE_WITHOUT_ID,
    FRONTMATTER_UNKNOWN_KEY,
    FRONTMATTER_MISSING_FIELD,
    FRONTMATTER_TYPE_MISMATCH,
    FRONTMATTER_RESERVED_IN_FRAGMENT,
    CONTENT_TYPE_UNRESOLVED,
    LIST_ENDED_BY_DIRECTIVE,
    DIRECTIVE_INDENTED_CODE,
    STEPS_NUMBERING_CONTINUED,
    ATTRIBUTE_SYNTAX,
    ATTRIBUTE_DUPLICATE_KEY,
    AVAILABLE_SYNTAX,
    ID_INVALID,
    IMAGE_ATTRIBUTE_MISSING,
    MODEL_TOML_SYNTAX,
    MODEL_UNKNOWN_KEY,
    MODEL_MISSING_KEY,
    MODEL_WRONG_TYPE,
    MODEL_INVALID_VALUE,
    MODEL_SPEC_UNSUPPORTED,
    MODEL_INVALID_NAME,
    MODEL_EMPTY_TEXT,
    MODEL_PATH_ABSOLUTE,
    MODEL_CONTENT_ROOT_MISSING,
    MODEL_OUTPUT_OVERLAPS_CONTENT,
    MODEL_TYPE_MULTIPLE_DEFAULTS,
    MODEL_TYPE_UNREACHABLE,
    MODEL_TYPE_TITLE,
    MODEL_FIELD_RESERVED,
    MODEL_TYPE_SYNTAX,
    MODEL_TYPE_FIELDS,
    MODEL_ENUM_VALUES,
    MODEL_SET_TOKEN,
    MODEL_DEFAULT_TYPE,
    MODEL_PHRASES_FIELD_TYPE,
    MODEL_PATTERN_SYNTAX,
    MODEL_NAME_MULTIPLE_ROLES,
    MODEL_NAME_CASE,
    MODEL_DIMENSION_EMPTY,
    MODEL_DIMENSION_VALUE_DUPLICATE,
    MODEL_DIMENSION_VALUE_SHARED,
    MODEL_LABEL_UNDECLARED,
    MODEL_VERSIONLESS_UNDECLARED,
    MODEL_LIFECYCLE_AVAILABLE_REQUIRED,
    MODEL_LIFECYCLE_GA_UNAVAILABLE,
    MODEL_NOTE_LABEL_REQUIRED,
    MODEL_AVAILABILITY_SYNTAX,
    MODEL_AVAILABILITY_UNKNOWN_NAME,
    MODEL_AVAILABILITY_VERSIONLESS,
    MODEL_AVAILABILITY_HISTORY_ORDER,
    MODEL_FEATURE_NESTED,
    MODEL_PHRASE_VALUE_TYPE,
    MODEL_GLOSSARY_DUPLICATE_TERM,
    MODEL_GLOSSARY_LINK,
    MODEL_WIDGET_RESERVED_NAME,
    MODEL_WIDGET_FORMS,
    MODEL_WIDGET_BINDING,
    MODEL_WIDGET_CONTAINER_PRIMARY,
    MODEL_WIDGET_GROUPABLE_FORM,
    MODEL_WIDGET_PLAIN_CONTENT,
    MODEL_CONSUMER_UNSUPPORTED,
    MODEL_CONSUMER_SITE,
    MODEL_CONSUMER_BASE_PATH,
    MODEL_BUILD_NAME_CASE,
    MODEL_BUILD_VARIANTS,
    MODEL_BUILD_UNKNOWN_DIMENSION,
    MODEL_BUILD_UNKNOWN_VALUE,
    MODEL_BUILD_AVAILABILITY,
    MODEL_BUILD_FILTER_TARGET,
    MODEL_BUILD_FILTER_VERSION,
    MODEL_BUILD_FILTER_EXCLUDED,
    MODEL_EDITOR_BUILD_UNKNOWN,
    MODEL_EDITOR_BUILD_REQUIRED,
    MODEL_ATTRIBUTE_RESERVED,
    DIRECTIVE_EXTRA_TEXT,
    LINK_PAGE_DROPPED,
    FRONTMATTER_SYNTAX,
    SOURCE_UNREADABLE,
    HEADING_EMPTY_SLUG,
    INCLUDE_HEADING_WITHOUT_ID,
    PHRASE_DOUBLE_BRACES,
    SNIPPET_ADDRESS,
    SNIPPET_SOURCE_UNKNOWN,
    SNIPPET_FILE_MISSING,
    SNIPPET_FILE_NOT_TEXT,
    SNIPPET_REGION_MISSING,
    SNIPPET_TAGS,
    MODEL_SOURCE_PATH_MISSING,
    MODEL_SOURCE_OUTSIDE_REPOSITORY,
    MODEL_SOURCE_REMOTE,
    LOCK_INVALID,
    LOCK_SOURCE_UNKNOWN,
    SOURCE_COPY_CHANGED,
    SOURCE_COPY_UNLOCKED,
    SOURCE_COPY_UNUSED,
    MODEL_INLINE_FIELD,
    MODEL_CHECK_NOT_CONFIGURABLE,
    PAGE_ORPHAN,
    FRAGMENT_UNUSED,
    PHRASE_UNUSED,
    FEATURE_UNUSED,
    GLOSSARY_TERM_UNUSED,
    IMAGE_UNUSED,
    IMAGE_LARGE,
    TITLE_DUPLICATE,
];

/// The diagnostics a project may set the level of in `[checks]`: the
/// registry's entries with `configurable = true`, in registry order. A test
/// keeps it equal to the registry.
pub const CONFIGURABLE: &[DiagnosticSlug] = &[
    PAGE_ORPHAN,
    FRAGMENT_UNUSED,
    PHRASE_UNUSED,
    FEATURE_UNUSED,
    GLOSSARY_TERM_UNUSED,
    IMAGE_UNUSED,
    IMAGE_LARGE,
    TITLE_DUPLICATE,
];
