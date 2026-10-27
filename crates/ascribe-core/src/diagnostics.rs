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

/// `ASC047`, error, file level: SPEC §8.2, "Frontmatter | Key the file's content type or the fragment schema doesn't declare, other than a reserved key on a page or `intended` on a fragment".
pub const FRONTMATTER_UNKNOWN_KEY: DiagnosticSlug = DiagnosticSlug("frontmatter-unknown-key");

/// `ASC048`, error, file level: SPEC §8.2, "Frontmatter | Required field missing".
pub const FRONTMATTER_MISSING_FIELD: DiagnosticSlug = DiagnosticSlug("frontmatter-missing-field");

/// `ASC049`, error, file level: SPEC §8.2, "Frontmatter | Value doesn't match the field's declared type".
pub const FRONTMATTER_TYPE_MISMATCH: DiagnosticSlug = DiagnosticSlug("frontmatter-type-mismatch");

/// `ASC050`, error, file level: SPEC §8.2, "Frontmatter | Reserved key (`available`, `variant`) on a fragment".
pub const FRONTMATTER_RESERVED_IN_FRAGMENT: DiagnosticSlug =
    DiagnosticSlug("frontmatter-reserved-in-fragment");

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

/// `ASC143`, error, file level: SPEC §8.2, "`@intended` | Names no check, a check that doesn't exist, a check that isn't a review check, or a check reported somewhere else".
pub const INTENDED_CHECK: DiagnosticSlug = DiagnosticSlug("intended-check");

/// `ASC144`, error, file level: SPEC §8.2, "`@intended` | An `intended` frontmatter entry that isn't a mapping with a `check` and a reason".
pub const INTENDED_ENTRY: DiagnosticSlug = DiagnosticSlug("intended-entry");

/// `ASC145`, advice, page level: SPEC §8.2, "`@intended` | An acknowledgement that matches no problem in any build".
pub const INTENDED_UNUSED: DiagnosticSlug = DiagnosticSlug("intended-unused");

/// `ASC146`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_INTENDED_CHECK: DiagnosticSlug = DiagnosticSlug("model-intended-check");

/// `ASC147`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_INTENDED_ENTRY: DiagnosticSlug = DiagnosticSlug("model-intended-entry");

/// `ASC148`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_FIELD_ROLE: DiagnosticSlug = DiagnosticSlug("model-field-role");

/// `ASC149`, advice, page level: a content check (area `pages`).
pub const PAGE_SIZE: DiagnosticSlug = DiagnosticSlug("page-size");

/// `ASC150`, advice, file level: a content check (area `pages`).
pub const PAGE_DESCRIPTION_MISSING: DiagnosticSlug = DiagnosticSlug("page-description-missing");

/// `ASC151`, advice, page level: a content check (area `pages`).
pub const HEADING_LEVEL_SKIPPED: DiagnosticSlug = DiagnosticSlug("heading-level-skipped");

/// `ASC152`, advice, file level: a content check (area `pages`).
pub const CODE_LANGUAGE_MISSING: DiagnosticSlug = DiagnosticSlug("code-language-missing");

/// `ASC153`, advice, file level: a content check (area `pages`).
pub const RECHECK_OVERDUE: DiagnosticSlug = DiagnosticSlug("recheck-overdue");

/// `ASC154`, advice, page level: a content check (area `project`).
pub const PAGE_ORPHAN: DiagnosticSlug = DiagnosticSlug("page-orphan");

/// `ASC155`, advice, file level: a content check (area `project`).
pub const FRAGMENT_UNUSED: DiagnosticSlug = DiagnosticSlug("fragment-unused");

/// `ASC156`, advice, file level: a content check (area `project`).
pub const PHRASE_UNUSED: DiagnosticSlug = DiagnosticSlug("phrase-unused");

/// `ASC157`, advice, file level: a content check (area `project`).
pub const FEATURE_UNUSED: DiagnosticSlug = DiagnosticSlug("feature-unused");

/// `ASC158`, advice, file level: a content check (area `project`).
pub const GLOSSARY_TERM_UNUSED: DiagnosticSlug = DiagnosticSlug("glossary-term-unused");

/// `ASC159`, advice, file level: a content check (area `project`).
pub const IMAGE_UNUSED: DiagnosticSlug = DiagnosticSlug("image-unused");

/// `ASC160`, advice, file level: a content check (area `project`).
pub const IMAGE_LARGE: DiagnosticSlug = DiagnosticSlug("image-large");

/// `ASC161`, advice, page level: a content check (area `project`).
pub const TITLE_DUPLICATE: DiagnosticSlug = DiagnosticSlug("title-duplicate");

/// `ASC162`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_CONSUMER_AGENTS_SITE: DiagnosticSlug = DiagnosticSlug("model-consumer-agents-site");

/// `ASC163`, advice, file level: a content check (area `outputs`).
pub const DESCRIPTION_TOO_LONG: DiagnosticSlug = DiagnosticSlug("description-too-long");

/// `ASC164`, advice, page level: a content check (area `outputs`).
pub const LLMS_SECTION_LARGE: DiagnosticSlug = DiagnosticSlug("llms-section-large");

/// `ASC165`, advice, file level: a content check (area `prose`): an alert
/// from Vale about the prose.
pub const PROSE: DiagnosticSlug = DiagnosticSlug("prose");

/// `ASC166`, advice, file level: a content check (area `prose`): Vale
/// couldn't be run, so the prose wasn't checked.
pub const PROSE_NOT_CHECKED: DiagnosticSlug = DiagnosticSlug("prose-not-checked");

/// `ASC167`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_CHECKS_VALE: DiagnosticSlug = DiagnosticSlug("model-checks-vale");
/// `ASC168`, advice, file level: a content check (area `links`).
pub const LINK_EXTERNAL_BROKEN: DiagnosticSlug = DiagnosticSlug("link-external-broken");

/// `ASC169`, advice, file level: a content check (area `links`).
pub const LINK_EXTERNAL_MOVED: DiagnosticSlug = DiagnosticSlug("link-external-moved");

/// `ASC170`, advice, file level: a content check (area `site`).
pub const DELIVERY_HOSTING: DiagnosticSlug = DiagnosticSlug("delivery-hosting");

/// `ASC171`, advice, file level: a content check (area `site`).
pub const DELIVERY_OUTPUT: DiagnosticSlug = DiagnosticSlug("delivery-output");

/// `ASC172`, advice, file level: a content check (area `project`).
pub const AVAILABILITY_LEFT_BEHIND: DiagnosticSlug = DiagnosticSlug("availability-left-behind");

/// `ASC173`, error, page level: SPEC §8.2, "`@id` | Permalink published by one build on more than one page (page level)".
pub const PERMALINK_DUPLICATE: DiagnosticSlug = DiagnosticSlug("permalink-duplicate");

/// `ASC174`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_CONSUMER_PERMALINK_PATH: DiagnosticSlug =
    DiagnosticSlug("model-consumer-permalink-path");

/// `ASC175`, error, file level: SPEC §8.2, "Permalinks list | `ascribe.permalinks` isn't valid TOML, or doesn't have the shape §7.5 gives it".
pub const PERMALINKS_LIST_INVALID: DiagnosticSlug = DiagnosticSlug("permalinks-list-invalid");

/// `ASC176`, warning, file level: SPEC §8.2, "Permalinks list | A permalink the content declares, or a name `ascribe.addresses` moves or retires, that `ascribe.permalinks` doesn't list as it is".
pub const PERMALINKS_LIST_BEHIND: DiagnosticSlug = DiagnosticSlug("permalinks-list-behind");

/// `ASC177`, warning, file level: SPEC §8.2, "Permalinks list | A name `ascribe.permalinks` lists that the project no longer has: not declared, and not moved or retired in `ascribe.addresses`".
pub const PERMALINK_LOST: DiagnosticSlug = DiagnosticSlug("permalink-lost");

/// `ASC182`, error, page level: SPEC §8.2, "`@include` | A page's includes expand it past the processor's limit (page level)".
pub const INCLUDE_LIMIT: DiagnosticSlug = DiagnosticSlug("include-limit");

/// `ASC183`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_CRAWLERS_LINE: DiagnosticSlug = DiagnosticSlug("model-crawlers-line");

/// `ASC184`, advice, file level: a content check (area `outputs`).
pub const LISTING_DISALLOWED: DiagnosticSlug = DiagnosticSlug("listing-disallowed");

/// `ASC185`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_LISTING_FOLDER: DiagnosticSlug = DiagnosticSlug("model-listing-folder");

/// `ASC186`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_LISTING_TYPE: DiagnosticSlug = DiagnosticSlug("model-listing-type");

/// `ASC187`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_LISTING_LOOSER: DiagnosticSlug = DiagnosticSlug("model-listing-looser");

/// `ASC188`, error, file level: SPEC §8.2, "Frontmatter | `listing` looser than a rule of the content model the page is under".
pub const LISTING_LOOSENED: DiagnosticSlug = DiagnosticSlug("listing-loosened");

/// `ASC189`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_CRAWLERS_NAMED_TWICE: DiagnosticSlug = DiagnosticSlug("model-crawlers-named-twice");

/// `ASC190`, advice, file level: a content check (area `outputs`).
pub const CRAWLERS_DISALLOWED: DiagnosticSlug = DiagnosticSlug("crawlers-disallowed");

/// `ASC191`, advice, page level: a content check (area `project`).
pub const DESCRIPTION_DUPLICATE: DiagnosticSlug = DiagnosticSlug("description-duplicate");

/// `ASC192`, advice, page level: a content check (area `pages`).
pub const TITLE_HEADING_DIFFERS: DiagnosticSlug = DiagnosticSlug("title-heading-differs");

/// `ASC193`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_TYPE_FILE: DiagnosticSlug = DiagnosticSlug("model-type-file");

/// `ASC194`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_TYPE_TEMPLATE: DiagnosticSlug = DiagnosticSlug("model-type-template");

/// `ASC195`, advice, file level: a content check (area `pages`).
pub const TEMPLATE_PLACEHOLDER_LEFT: DiagnosticSlug = DiagnosticSlug("template-placeholder-left");

/// `ASC196`, error, file level: SPEC §8.2, "Frontmatter | Page names no content type, in a project that declares types".
pub const CONTENT_TYPE_MISSING: DiagnosticSlug = DiagnosticSlug("content-type-missing");

/// `ASC197`, error, file level: SPEC §8.2, "Frontmatter | Page's `type` names no declared content type".
pub const CONTENT_TYPE_UNKNOWN: DiagnosticSlug = DiagnosticSlug("content-type-unknown");

/// `ASC198`, advice, file level: a content check (area `pages`).
pub const DESCRIPTION_LINK: DiagnosticSlug = DiagnosticSlug("description-link");

/// `ASC199`, warning, file level: a content check (area `examples`).
pub const EXAMPLE_STALE: DiagnosticSlug = DiagnosticSlug("example-stale");

/// `ASC200`, advice, file level: a content check (area `examples`).
pub const EXAMPLE_UNOBSERVED: DiagnosticSlug = DiagnosticSlug("example-unobserved");

/// `ASC201`, warning, file level: a content check (area `examples`).
pub const EXAMPLES_CONFIG_INVALID: DiagnosticSlug = DiagnosticSlug("examples-config-invalid");

/// `ASC202`, advice, file level: a content check (area `examples`).
pub const OBSERVATION_FILE_MISSING: DiagnosticSlug = DiagnosticSlug("observation-file-missing");

/// `ASC204`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_RELEASES_TARGET: DiagnosticSlug = DiagnosticSlug("model-releases-target");

/// `ASC205`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_RELEASES_LINE: DiagnosticSlug = DiagnosticSlug("model-releases-line");

/// `ASC206`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_BUILD_VERSIONS: DiagnosticSlug = DiagnosticSlug("model-build-versions");

/// `ASC207`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_RELEASES_ORDER: DiagnosticSlug = DiagnosticSlug("model-releases-order");

/// `ASC208`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_RELEASES_ENDS: DiagnosticSlug = DiagnosticSlug("model-releases-ends");

/// `ASC209`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_VERSIONS_CURRENT: DiagnosticSlug = DiagnosticSlug("model-versions-current");

/// `ASC210`, error, file level: SPEC §8.2, "`@available` | Version newer than every release line its target declares".
pub const AVAILABLE_VERSION_AHEAD: DiagnosticSlug = DiagnosticSlug("available-version-ahead");

/// `ASC211`, advice, file level: a content check (area `project`).
pub const AVAILABLE_LINE_UNDECLARED: DiagnosticSlug = DiagnosticSlug("available-line-undeclared");

/// `ASC212`, advice, file level: a content check (area `project`).
pub const AVAILABLE_SAME_IN_EVERY_LINE: DiagnosticSlug =
    DiagnosticSlug("available-same-in-every-line");

/// `ASC213`, warning, file level: a content check (area `project`).
pub const AVAILABLE_IN_NO_LINE: DiagnosticSlug = DiagnosticSlug("available-in-no-line");

/// `ASC214`, advice, file level: a content check (area `project`).
pub const RELEASE_LINE_ENDED: DiagnosticSlug = DiagnosticSlug("release-line-ended");

/// `ASC215`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_PHRASE_LINE: DiagnosticSlug = DiagnosticSlug("model-phrase-line");

/// `ASC216`, error, file level: SPEC §8.2, "Frontmatter | `replaces` that doesn't name a page".
pub const REPLACES_NOT_A_PAGE: DiagnosticSlug = DiagnosticSlug("replaces-not-a-page");

/// `ASC217`, error, page level: SPEC §8.2, "Frontmatter | A page that `replaces` itself, or a chain of `replaces` that comes back to where it started (page level)".
pub const REPLACES_CYCLE: DiagnosticSlug = DiagnosticSlug("replaces-cycle");

/// `ASC218`, error, page level: SPEC §8.2, "Frontmatter | A page and the page it `replaces`, both published by one line of a versioned build (page level, per build)".
pub const REPLACES_OVERLAP: DiagnosticSlug = DiagnosticSlug("replaces-overlap");

/// `ASC219`, error, page level: SPEC §8.2, "Frontmatter | Two pages that `replaces` the same page, both published by one line of a versioned build (page level, per build)".
pub const REPLACES_SAME_PAGE: DiagnosticSlug = DiagnosticSlug("replaces-same-page");

/// `ASC220`, advice, page level: a content check (area `pages`).
pub const LINK_INTO_OLDER_LINE: DiagnosticSlug = DiagnosticSlug("link-into-older-line");

/// `ASC221`, advice, page level: a content check (area `pages`).
pub const LINK_FRAGMENT_DROPPED: DiagnosticSlug = DiagnosticSlug("link-fragment-dropped");

/// `ASC222`, error, page level: SPEC §8.2, "Frontmatter | `listing` looser than `[listing] older-lines`, on a page an older line of a versioned build publishes (page level, per build)".
pub const LISTING_LOOSENED_IN_LINE: DiagnosticSlug = DiagnosticSlug("listing-loosened-in-line");

/// `ASC223`, advice, file level: a content check (area `pages`).
pub const HTML_COMMENT: DiagnosticSlug = DiagnosticSlug("html-comment");

/// `ASC224`, advice, file level: a content check (area `pages`).
pub const MEMO_EMPTY: DiagnosticSlug = DiagnosticSlug("memo-empty");

/// `ASC225`, error, file level: SPEC §8.2, "Memo files | A memo file that can't be read, has no frontmatter or frontmatter that isn't valid YAML, has no `about`, has an `about` that isn't written as a target, has a key other than `about` and `quote`, or has a `quote` with an `about` that names a content model entry; or a `memos/` folder inside the content root".
pub const MEMO_INVALID: DiagnosticSlug = DiagnosticSlug("memo-invalid");

/// `ASC226`, warning, file level: SPEC §8.2, "Memo files | A memo file's `about` that names nothing in the project".
pub const MEMO_DETACHED: DiagnosticSlug = DiagnosticSlug("memo-detached");

/// `ASC227`, advice, file level: a content check (area `project`).
pub const MEMO_OUTDATED: DiagnosticSlug = DiagnosticSlug("memo-outdated");

/// `ASC228`, advice, file level: a content check (area `project`).
pub const MEMO_LONG: DiagnosticSlug = DiagnosticSlug("memo-long");

/// `ASC229`, advice, file level: a content check (area `pages`).
pub const TEMPLATE_SECTION_EMPTY: DiagnosticSlug = DiagnosticSlug("template-section-empty");

/// `ASC230`, error, file level: SPEC §8.2, "Navigation | `ascribe.nav` isn't valid TOML, or doesn't have the shape §7.8 gives it".
pub const NAV_INVALID: DiagnosticSlug = DiagnosticSlug("nav-invalid");

/// `ASC231`, error, file level: SPEC §8.2, "Navigation | A `pages` entry in `ascribe.nav` that isn't a page in its section's folder, or that names the section's own `index.md`".
pub const NAV_PAGE_UNKNOWN: DiagnosticSlug = DiagnosticSlug("nav-page-unknown");

/// `ASC232`, error, file level: SPEC §8.2, "Navigation | A `folder` in `ascribe.nav` that isn't a section".
pub const NAV_FOLDER_UNKNOWN: DiagnosticSlug = DiagnosticSlug("nav-folder-unknown");

/// `ASC233`, error, file level: SPEC §8.2, "Navigation | A folder `ascribe.nav` lists twice, or a page a section's `pages` names twice".
pub const NAV_DUPLICATE: DiagnosticSlug = DiagnosticSlug("nav-duplicate");

/// `ASC234`, error, file level: SPEC §8.2, "Navigation | A folder with an `index.md` and a page beside it with the folder's name".
pub const SECTION_PAGE_BOTH: DiagnosticSlug = DiagnosticSlug("section-page-both");

/// `ASC235`, advice, file level: a content check (area `project`).
pub const NAV_PAGE_UNPLACED: DiagnosticSlug = DiagnosticSlug("nav-page-unplaced");

/// `ASC236`, advice, file level: a content check (area `project`).
pub const NAV_SECTION_UNPLACED: DiagnosticSlug = DiagnosticSlug("nav-section-unplaced");

/// `ASC237`, advice, file level: a content check (area `pages`).
pub const NAV_LABEL_LONG: DiagnosticSlug = DiagnosticSlug("nav-label-long");

/// `ASC238`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_GLOSSARY_PAGE: DiagnosticSlug = DiagnosticSlug("model-glossary-page");

/// `ASC239`, error, file level: SPEC §8.2, "Glossary | Two terms, or a term and an alias, with the same text, or two terms with the same id".
pub const GLOSSARY_TERM_DUPLICATE: DiagnosticSlug = DiagnosticSlug("glossary-term-duplicate");

/// `ASC240`, error, file level: SPEC §8.2, "Glossary | A term whose section doesn't start with a paragraph".
pub const GLOSSARY_DEFINITION_MISSING: DiagnosticSlug =
    DiagnosticSlug("glossary-definition-missing");

/// `ASC241`, advice, page level: a content check (area `pages`).
pub const GLOSSARY_DEFINITION_LONG: DiagnosticSlug = DiagnosticSlug("glossary-definition-long");

/// `ASC242`, error, file level: SPEC §8.2, "Glossary | `@term` on a heading that no page the glossary lists shows".
pub const TERM_OFF_GLOSSARY_PAGE: DiagnosticSlug = DiagnosticSlug("term-off-glossary-page");

/// `ASC243`, advice, file level: a content check (area `project`).
pub const BASELINE_ENTRY_UNMATCHED: DiagnosticSlug = DiagnosticSlug("baseline-entry-unmatched");

/// `ASC244`, advice, file level: a content check (area `project`).
pub const FRONTMATTER_FIELD_REPEATED: DiagnosticSlug = DiagnosticSlug("frontmatter-field-repeated");

/// `ASC245`, error, file level: SPEC §8.2, "`@available` | `planned` for a target with no planned line, or more than one".
pub const AVAILABLE_PLANNED_LINE: DiagnosticSlug = DiagnosticSlug("available-planned-line");

/// `ASC246`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_FROM_FILE: DiagnosticSlug = DiagnosticSlug("model-from-file");

/// `ASC247`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_FROM_TAKE: DiagnosticSlug = DiagnosticSlug("model-from-take");

/// `ASC248`, error, file level: a rule for loading `ascribe.toml`.
pub const MODEL_FROM_WRITTEN: DiagnosticSlug = DiagnosticSlug("model-from-written");

/// `ASC249`, error, file level: SPEC §8.2, "`@summary` | Following form whose
/// block isn't a paragraph".
pub const SUMMARY_NOT_PARAGRAPH: DiagnosticSlug = DiagnosticSlug("summary-not-paragraph");

/// `ASC250`, error, page level: SPEC §8.2, "`@summary` | A second summary in
/// one section, including summaries from included content (page level)".
pub const SUMMARY_REPEATED: DiagnosticSlug = DiagnosticSlug("summary-repeated");

/// `ASC251`, error, file level: SPEC §8.2, "`@summary` | Summary inside a
/// `@variant` arm".
pub const SUMMARY_IN_VARIANT: DiagnosticSlug = DiagnosticSlug("summary-in-variant");

/// `ASC252`, advice, page level: a content check (area `pages`).
pub const SUMMARY_MISSING: DiagnosticSlug = DiagnosticSlug("summary-missing");

/// `ASC253`, advice, page level: a content check (area `pages`).
pub const SUMMARY_LONG: DiagnosticSlug = DiagnosticSlug("summary-long");

/// `ASC254`, error, file level: SPEC §8.2, "Addresses file | `ascribe.addresses`
/// isn't valid TOML, or doesn't have the shape §7.10 gives it".
pub const ADDRESSES_FILE_INVALID: DiagnosticSlug = DiagnosticSlug("addresses-file-invalid");

/// `ASC255`, warning, file level: SPEC §8.2, "Addresses file | A page's
/// address, or its Markdown version's, that the base published and nothing
/// accounts for".
pub const ADDRESS_LOST: DiagnosticSlug = DiagnosticSlug("address-lost");

/// `ASC256`, advice, file level: SPEC §8.2, "Addresses file | A heading's
/// explicit id that the base published on a page that's still published, and
/// nothing accounts for".
pub const ANCHOR_LOST: DiagnosticSlug = DiagnosticSlug("anchor-lost");

/// `ASC257`, warning, file level: a content check (area `project`).
pub const ADDRESSES_BASE_UNREADABLE: DiagnosticSlug = DiagnosticSlug("addresses-base-unreadable");

/// `ASC258`, advice, file level: SPEC §8.2, "Addresses file | A build the base
/// has and the working tree doesn't, other than a line archived now".
pub const ADDRESSES_BUILD_GONE: DiagnosticSlug = DiagnosticSlug("addresses-build-gone");

/// `ASC259`, error, file level: a content model loader rule.
pub const MODEL_API_OUTSIDE_REPOSITORY: DiagnosticSlug =
    DiagnosticSlug("model-api-outside-repository");

/// `ASC260`, error, file level: a content model loader rule.
pub const MODEL_API_REFERENCE: DiagnosticSlug = DiagnosticSlug("model-api-reference");

/// `ASC261`, error, file level: SPEC §8.2, "APIs | A declared spec that
/// doesn't exist, can't be read, or isn't YAML or JSON".
pub const API_SPEC_UNREADABLE: DiagnosticSlug = DiagnosticSlug("api-spec-unreadable");

/// `ASC262`, advice, file level: SPEC §8.2, "APIs | A declared spec with no
/// operations".
pub const API_SPEC_EMPTY: DiagnosticSlug = DiagnosticSlug("api-spec-empty");

/// `ASC263`, error, file level: SPEC §8.2, "Links | Target is an operation
/// the declared spec doesn't have".
pub const LINK_OPERATION_MISSING: DiagnosticSlug = DiagnosticSlug("link-operation-missing");

/// `ASC264`, error, file level: SPEC §8.2, "Links | Target is a declared spec
/// outside the project's folder, with no operation".
pub const LINK_SPEC_OUTSIDE_PROJECT: DiagnosticSlug = DiagnosticSlug("link-spec-outside-project");
/// `ASC265`, error, file level: SPEC §8.2, "Addresses file | A `[moved]` value that isn't a page, or a heading on one, in every build that published its key, or, for a permalink, a permalink the content declares".
pub const ADDRESSES_ENTRY_NOT_A_PAGE: DiagnosticSlug = DiagnosticSlug("addresses-entry-not-a-page");

/// `ASC266`, advice, file level: SPEC §8.2, "Addresses file | A key of `[moved]` or `[retired]` whose source is back: a page, a heading's `@id`, or a permalink the content declares".
pub const ADDRESSES_ENTRY_LIVE: DiagnosticSlug = DiagnosticSlug("addresses-entry-live");

/// `ASC267`, error, file level: SPEC §8.2, "Addresses file | A key of `[moved]` or `[retired]` whose address another source publishes now".
pub const ADDRESSES_ENTRY_TAKEN: DiagnosticSlug = DiagnosticSlug("addresses-entry-taken");

/// `ASC268`, advice, file level: SPEC §8.2, "Addresses file | An entry the base's `ascribe.addresses` doesn't have, whose key the base never published".
pub const ADDRESSES_ENTRY_UNPUBLISHED: DiagnosticSlug =
    DiagnosticSlug("addresses-entry-unpublished");

/// `ASC269`, error, file level: SPEC §8.2, "Addresses file | A key written twice in a table, or in both `[moved]` and `[retired]`".
pub const ADDRESSES_KEY_TWICE: DiagnosticSlug = DiagnosticSlug("addresses-key-twice");

/// `ASC270`, advice, file level: SPEC §8.2, "Addresses file | A `[moved]` value that's another entry's key".
pub const ADDRESSES_CHAIN: DiagnosticSlug = DiagnosticSlug("addresses-chain");

/// `ASC271`, error, file level: SPEC §8.2, "Addresses file | An `#id` entry from a page that's gone whose id is live on the page its page's addresses lead to".
pub const ADDRESSES_ANCHOR_COLLISION: DiagnosticSlug = DiagnosticSlug("addresses-anchor-collision");

/// `ASC272`, error, file level: SPEC §8.2, "Addresses file | `ascribe.addresses` deleted while the base has it".
pub const ADDRESSES_FILE_DELETED: DiagnosticSlug = DiagnosticSlug("addresses-file-deleted");

/// `ASC273`, advice, file level: SPEC §8.2, "Addresses file | Many addresses forwarded to one index page or the home page".
pub const ADDRESSES_SOFT_404: DiagnosticSlug = DiagnosticSlug("addresses-soft-404");

/// `ASC274`, error, file level: SPEC §8.2, "Links | Target is an operation
/// with no id, by its method and path, and the API's reference is a
/// template".
pub const LINK_OPERATION_FORM_UNSUPPORTED: DiagnosticSlug =
    DiagnosticSlug("link-operation-form-unsupported");

/// `ASC275`, advice, file level: SPEC §8.2, "Links | Target is an operation
/// that has an id, by its method and path".
pub const LINK_OPERATION_BY_PATH: DiagnosticSlug = DiagnosticSlug("link-operation-by-path");

/// `ASC276`, advice, file level: SPEC §8.2, "Links | Target is an operation
/// the spec marks deprecated".
pub const LINK_OPERATION_DEPRECATED: DiagnosticSlug = DiagnosticSlug("link-operation-deprecated");

/// `ASC277`, error, file level: SPEC §8.2, "Examples | A marked example
/// that disagrees with its body's schema".
pub const API_EXAMPLE_SCHEMA_MISMATCH: DiagnosticSlug =
    DiagnosticSlug("api-example-schema-mismatch");

/// `ASC278`, error, file level: SPEC §8.2, "Examples | A marked example
/// that isn't JSON, or isn't a `json` block".
pub const API_EXAMPLE_NOT_JSON: DiagnosticSlug = DiagnosticSlug("api-example-not-json");

/// `ASC279`, error, file level: SPEC §8.2, "Examples | A marked example
/// whose operation has no body it names".
pub const API_EXAMPLE_BODY_MISSING: DiagnosticSlug = DiagnosticSlug("api-example-body-missing");

/// `ASC280`, advice, file level: SPEC §8.2, "Examples | A marked example
/// whose schema can't all be checked".
pub const API_EXAMPLE_SCHEMA_UNSUPPORTED: DiagnosticSlug =
    DiagnosticSlug("api-example-schema-unsupported");

/// `ASC281`, error, file level: SPEC §8.2, "Links | An `@operation`'s link
/// isn't to an operation of a declared API".
pub const OPERATION_LINK_NOT_OPERATION: DiagnosticSlug =
    DiagnosticSlug("operation-link-not-operation");

/// `ASC282`, advice, file level: a content check (area `project`).
pub const API_OPERATION_UNREFERENCED: DiagnosticSlug = DiagnosticSlug("api-operation-unreferenced");

/// `ASC283`, advice, file level: a content check (area `project`).
pub const BUILD_FILTER_LINE_UNDECLARED: DiagnosticSlug =
    DiagnosticSlug("build-filter-line-undeclared");

/// `ASC284`, warning, file level: SPEC §8.2, "Attributes | A word after a
/// fenced code block's language that starts like a pair a processor reads
/// (its key in any case, then optional spaces and `=`) but isn't one".
pub const INFO_PAIR_MALFORMED: DiagnosticSlug = DiagnosticSlug("info-pair-malformed");

/// `ASC285`, warning, file level: a content check (area `examples`).
pub const EXAMPLES_CONFIG_UNREAD: DiagnosticSlug = DiagnosticSlug("examples-config-unread");

/// `ASC286`, error, file level: a content check (area `pages`).
pub const TEMPLATE_PLACEHOLDER_PUBLISHED: DiagnosticSlug =
    DiagnosticSlug("template-placeholder-published");

/// `ASC287`, warning, file level: a content check (area `pages`).
pub const GITHUB_ALERT: DiagnosticSlug = DiagnosticSlug("github-alert");

/// The checks renamed, each by the name it had before, which still names
/// it in `[checks]`, in an acknowledgement, and to `ascribe explain`, for a
/// release (decision #953).
pub const OLD_NAMES: &[(&str, DiagnosticSlug)] = &[("review-overdue", RECHECK_OVERDUE)];

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
    INTENDED_CHECK,
    INTENDED_ENTRY,
    INTENDED_UNUSED,
    MODEL_INTENDED_CHECK,
    MODEL_INTENDED_ENTRY,
    MODEL_FIELD_ROLE,
    PAGE_SIZE,
    PAGE_DESCRIPTION_MISSING,
    HEADING_LEVEL_SKIPPED,
    CODE_LANGUAGE_MISSING,
    RECHECK_OVERDUE,
    PAGE_ORPHAN,
    FRAGMENT_UNUSED,
    PHRASE_UNUSED,
    FEATURE_UNUSED,
    GLOSSARY_TERM_UNUSED,
    IMAGE_UNUSED,
    IMAGE_LARGE,
    TITLE_DUPLICATE,
    MODEL_CONSUMER_AGENTS_SITE,
    DESCRIPTION_TOO_LONG,
    LLMS_SECTION_LARGE,
    PROSE,
    PROSE_NOT_CHECKED,
    MODEL_CHECKS_VALE,
    LINK_EXTERNAL_BROKEN,
    LINK_EXTERNAL_MOVED,
    DELIVERY_HOSTING,
    DELIVERY_OUTPUT,
    AVAILABILITY_LEFT_BEHIND,
    PERMALINK_DUPLICATE,
    MODEL_CONSUMER_PERMALINK_PATH,
    PERMALINKS_LIST_INVALID,
    PERMALINKS_LIST_BEHIND,
    PERMALINK_LOST,
    INCLUDE_LIMIT,
    MODEL_CRAWLERS_LINE,
    LISTING_DISALLOWED,
    MODEL_LISTING_FOLDER,
    MODEL_LISTING_TYPE,
    MODEL_LISTING_LOOSER,
    LISTING_LOOSENED,
    MODEL_CRAWLERS_NAMED_TWICE,
    CRAWLERS_DISALLOWED,
    DESCRIPTION_DUPLICATE,
    TITLE_HEADING_DIFFERS,
    MODEL_TYPE_FILE,
    MODEL_TYPE_TEMPLATE,
    TEMPLATE_PLACEHOLDER_LEFT,
    CONTENT_TYPE_MISSING,
    CONTENT_TYPE_UNKNOWN,
    DESCRIPTION_LINK,
    EXAMPLE_STALE,
    EXAMPLE_UNOBSERVED,
    EXAMPLES_CONFIG_INVALID,
    OBSERVATION_FILE_MISSING,
    MODEL_RELEASES_TARGET,
    MODEL_RELEASES_LINE,
    MODEL_BUILD_VERSIONS,
    MODEL_RELEASES_ORDER,
    MODEL_RELEASES_ENDS,
    MODEL_VERSIONS_CURRENT,
    AVAILABLE_VERSION_AHEAD,
    AVAILABLE_LINE_UNDECLARED,
    AVAILABLE_SAME_IN_EVERY_LINE,
    AVAILABLE_IN_NO_LINE,
    RELEASE_LINE_ENDED,
    MODEL_PHRASE_LINE,
    REPLACES_NOT_A_PAGE,
    REPLACES_CYCLE,
    REPLACES_OVERLAP,
    REPLACES_SAME_PAGE,
    LINK_INTO_OLDER_LINE,
    LINK_FRAGMENT_DROPPED,
    LISTING_LOOSENED_IN_LINE,
    HTML_COMMENT,
    MEMO_EMPTY,
    MEMO_INVALID,
    MEMO_DETACHED,
    MEMO_OUTDATED,
    MEMO_LONG,
    TEMPLATE_SECTION_EMPTY,
    NAV_INVALID,
    NAV_PAGE_UNKNOWN,
    NAV_FOLDER_UNKNOWN,
    NAV_DUPLICATE,
    SECTION_PAGE_BOTH,
    NAV_PAGE_UNPLACED,
    NAV_SECTION_UNPLACED,
    NAV_LABEL_LONG,
    MODEL_GLOSSARY_PAGE,
    GLOSSARY_TERM_DUPLICATE,
    GLOSSARY_DEFINITION_MISSING,
    GLOSSARY_DEFINITION_LONG,
    TERM_OFF_GLOSSARY_PAGE,
    BASELINE_ENTRY_UNMATCHED,
    FRONTMATTER_FIELD_REPEATED,
    AVAILABLE_PLANNED_LINE,
    MODEL_FROM_FILE,
    MODEL_FROM_TAKE,
    MODEL_FROM_WRITTEN,
    SUMMARY_NOT_PARAGRAPH,
    SUMMARY_REPEATED,
    SUMMARY_IN_VARIANT,
    SUMMARY_MISSING,
    SUMMARY_LONG,
    ADDRESSES_FILE_INVALID,
    ADDRESS_LOST,
    ANCHOR_LOST,
    ADDRESSES_BASE_UNREADABLE,
    ADDRESSES_BUILD_GONE,
    MODEL_API_OUTSIDE_REPOSITORY,
    MODEL_API_REFERENCE,
    API_SPEC_UNREADABLE,
    API_SPEC_EMPTY,
    LINK_OPERATION_MISSING,
    LINK_SPEC_OUTSIDE_PROJECT,
    ADDRESSES_ENTRY_NOT_A_PAGE,
    ADDRESSES_ENTRY_LIVE,
    ADDRESSES_ENTRY_TAKEN,
    ADDRESSES_ENTRY_UNPUBLISHED,
    ADDRESSES_KEY_TWICE,
    ADDRESSES_CHAIN,
    ADDRESSES_ANCHOR_COLLISION,
    ADDRESSES_FILE_DELETED,
    ADDRESSES_SOFT_404,
    LINK_OPERATION_FORM_UNSUPPORTED,
    LINK_OPERATION_BY_PATH,
    LINK_OPERATION_DEPRECATED,
    API_EXAMPLE_SCHEMA_MISMATCH,
    API_EXAMPLE_NOT_JSON,
    API_EXAMPLE_BODY_MISSING,
    API_EXAMPLE_SCHEMA_UNSUPPORTED,
    OPERATION_LINK_NOT_OPERATION,
    API_OPERATION_UNREFERENCED,
    BUILD_FILTER_LINE_UNDECLARED,
    INFO_PAIR_MALFORMED,
    EXAMPLES_CONFIG_UNREAD,
    TEMPLATE_PLACEHOLDER_PUBLISHED,
    GITHUB_ALERT,
];

/// The diagnostics a project may set the level of in `[checks]`: the
/// registry's entries with `configurable = true`, in registry order. A test
/// keeps it equal to the registry.
pub const CONFIGURABLE: &[DiagnosticSlug] = &[
    PAGE_SIZE,
    PAGE_DESCRIPTION_MISSING,
    HEADING_LEVEL_SKIPPED,
    CODE_LANGUAGE_MISSING,
    RECHECK_OVERDUE,
    PAGE_ORPHAN,
    FRAGMENT_UNUSED,
    PHRASE_UNUSED,
    FEATURE_UNUSED,
    GLOSSARY_TERM_UNUSED,
    IMAGE_UNUSED,
    IMAGE_LARGE,
    TITLE_DUPLICATE,
    DESCRIPTION_TOO_LONG,
    LLMS_SECTION_LARGE,
    PROSE,
    PROSE_NOT_CHECKED,
    LINK_EXTERNAL_BROKEN,
    LINK_EXTERNAL_MOVED,
    DELIVERY_HOSTING,
    DELIVERY_OUTPUT,
    AVAILABILITY_LEFT_BEHIND,
    LISTING_DISALLOWED,
    CRAWLERS_DISALLOWED,
    DESCRIPTION_DUPLICATE,
    TITLE_HEADING_DIFFERS,
    TEMPLATE_PLACEHOLDER_LEFT,
    DESCRIPTION_LINK,
    EXAMPLE_STALE,
    EXAMPLE_UNOBSERVED,
    EXAMPLES_CONFIG_INVALID,
    OBSERVATION_FILE_MISSING,
    AVAILABLE_LINE_UNDECLARED,
    AVAILABLE_SAME_IN_EVERY_LINE,
    AVAILABLE_IN_NO_LINE,
    RELEASE_LINE_ENDED,
    LINK_INTO_OLDER_LINE,
    LINK_FRAGMENT_DROPPED,
    HTML_COMMENT,
    MEMO_EMPTY,
    MEMO_OUTDATED,
    MEMO_LONG,
    TEMPLATE_SECTION_EMPTY,
    NAV_PAGE_UNPLACED,
    NAV_SECTION_UNPLACED,
    NAV_LABEL_LONG,
    GLOSSARY_DEFINITION_LONG,
    BASELINE_ENTRY_UNMATCHED,
    FRONTMATTER_FIELD_REPEATED,
    SUMMARY_MISSING,
    SUMMARY_LONG,
    ADDRESSES_BASE_UNREADABLE,
    API_OPERATION_UNREFERENCED,
    BUILD_FILTER_LINE_UNDECLARED,
    EXAMPLES_CONFIG_UNREAD,
    TEMPLATE_PLACEHOLDER_PUBLISHED,
    GITHUB_ALERT,
];

/// The configurable checks that are off unless a project names them in
/// `[checks]`: the registry's entries with `off-by-default = true`, in
/// registry order. A test keeps it equal to the registry.
pub const OFF_BY_DEFAULT: &[DiagnosticSlug] = &[SUMMARY_MISSING, SUMMARY_LONG];

/// The checks an author can acknowledge (SPEC §4.9): the registry's entries
/// whose next step is `review`, each with the place its problems are reported
/// at, in registry order. A test keeps it equal to the registry.
pub const ACKNOWLEDGEABLE: &[(DiagnosticSlug, crate::Place)] = &[
    (RECHECK_OVERDUE, crate::Place::Page),
    (PAGE_ORPHAN, crate::Place::Page),
    (FRAGMENT_UNUSED, crate::Place::Page),
    (PHRASE_UNUSED, crate::Place::Entry),
    (FEATURE_UNUSED, crate::Place::Entry),
    (GLOSSARY_TERM_UNUSED, crate::Place::Block),
    (IMAGE_UNUSED, crate::Place::Entry),
    (IMAGE_LARGE, crate::Place::Entry),
    (LINK_EXTERNAL_BROKEN, crate::Place::Block),
    (AVAILABILITY_LEFT_BEHIND, crate::Place::Entry),
    (TITLE_HEADING_DIFFERS, crate::Place::Page),
    (HTML_COMMENT, crate::Place::Block),
];
