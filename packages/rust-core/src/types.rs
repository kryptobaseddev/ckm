//! CKM type definitions — Rust implementation of INTERFACE.md v2.0.0.
//!
//! Every type here has a 1:1 correspondence with the Single Source of Truth interface definition.

use serde::{Deserialize, Serialize};

// ────────────────────────────────────────────────────────────────────────────
// Section 2: Input Types (from ckm.json v2)
// ────────────────────────────────────────────────────────────────────────────

/// The set of portable primitive types, mapped to JSON Schema primitives.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CanonicalType {
    /// JSON string type.
    String,
    /// JSON boolean type.
    Boolean,
    /// JSON number type (floating point).
    Number,
    /// JSON integer type.
    Integer,
    /// JSON array type.
    Array,
    /// JSON object type.
    Object,
    /// JSON null type.
    Null,
    /// Any type (untyped).
    Any,
}

impl std::fmt::Display for CanonicalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            CanonicalType::String => "string",
            CanonicalType::Boolean => "boolean",
            CanonicalType::Number => "number",
            CanonicalType::Integer => "integer",
            CanonicalType::Array => "array",
            CanonicalType::Object => "object",
            CanonicalType::Null => "null",
            CanonicalType::Any => "any",
        };
        write!(f, "{}", s)
    }
}

/// A portable type reference with canonical mapping.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CkmTypeRef {
    /// Language-agnostic canonical type.
    pub canonical: CanonicalType,

    /// Source language type annotation (e.g., `CalVerFormat`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original: Option<String>,

    /// Known enum values for string types.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#enum: Option<Vec<String>>,
}

/// A property within a concept.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CkmProperty {
    /// Property name.
    pub name: String,

    /// Type reference (canonical + original).
    pub r#type: CkmTypeRef,

    /// Description from source documentation.
    pub description: String,

    /// Whether the property is required.
    pub required: bool,

    /// Default value. Null means no default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
}

/// A domain concept extracted from source code.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CkmConcept {
    /// Unique identifier (e.g., "concept-calver-config").
    pub id: String,

    /// Type name (e.g., `CalVerConfig`).
    pub name: String,

    /// Topic slug (e.g., "calver") — used for topic derivation.
    pub slug: String,

    /// One-line description.
    pub what: String,

    /// Semantic tags (e.g., ["config"]).
    pub tags: Vec<String>,

    /// Properties of the type, if applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub properties: Option<Vec<CkmProperty>>,
}

/// A function parameter within an operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CkmInput {
    /// Parameter name.
    pub name: String,

    /// Type reference.
    pub r#type: CkmTypeRef,

    /// Whether the parameter is required.
    pub required: bool,

    /// Description from source documentation.
    pub description: String,
}

/// A return value from an operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CkmOutput {
    /// Type reference.
    pub r#type: CkmTypeRef,

    /// Description of the return value.
    pub description: String,
}

/// A user-facing operation extracted from source code.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CkmOperation {
    /// Unique identifier (e.g., "op-validate").
    pub id: String,

    /// Function name (e.g., "validate").
    pub name: String,

    /// One-line description.
    pub what: String,

    /// Semantic tags for topic linkage.
    pub tags: Vec<String>,

    /// Function parameters.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inputs: Option<Vec<CkmInput>>,

    /// Return value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outputs: Option<CkmOutput>,
}

/// A rule enforced by the tool.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CkmConstraint {
    /// Unique identifier (e.g., "constraint-future-date").
    pub id: String,

    /// Human-readable rule description.
    pub rule: String,

    /// Function or module that enforces the constraint.
    pub enforced_by: String,

    /// Severity level.
    pub severity: Severity,
}

/// Severity levels for constraints.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Validation error — must be fixed.
    Error,
    /// Validation warning — should be addressed.
    Warning,
    /// Informational notice.
    Info,
}

/// A single step within a workflow.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CkmWorkflowStep {
    /// Discriminant: CLI command or manual action.
    pub action: StepAction,

    /// The command or instruction.
    pub value: String,

    /// Optional explanatory note.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// Discriminant for workflow step types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StepAction {
    /// A CLI command to execute.
    Command,
    /// A manual action for the user to perform.
    Manual,
}

/// A multi-step workflow for achieving a common goal.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CkmWorkflow {
    /// Unique identifier.
    pub id: String,

    /// What the workflow achieves.
    pub goal: String,

    /// Semantic tags.
    pub tags: Vec<String>,

    /// Ordered steps (minimum 1).
    pub steps: Vec<CkmWorkflowStep>,
}

/// A configuration schema entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CkmConfigEntry {
    /// Dotted key path (e.g., "calver.format").
    pub key: String,

    /// Type reference.
    pub r#type: CkmTypeRef,

    /// Description.
    pub description: String,

    /// Default value. Null means no default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,

    /// Whether the config entry is required.
    pub required: bool,
}

/// Provenance metadata about the manifest source.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CkmMeta {
    /// Project name (e.g., "my-tool").
    pub project: String,

    /// Source language (e.g., "typescript", "python", "rust").
    pub language: String,

    /// Tool that generated the manifest (e.g., "forge-ts@0.21.1").
    pub generator: String,

    /// ISO 8601 timestamp of generation.
    pub generated: String,

    /// Optional URL to source repository.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
}

/// The top-level CKM manifest object (v2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CkmManifest {
    /// Schema URL (e.g., `https://ckm.dev/schemas/v2.json`).
    #[serde(rename = "$schema")]
    pub schema: String,

    /// Schema version (e.g., "2.0.0").
    pub version: String,

    /// Project metadata and provenance.
    pub meta: CkmMeta,

    /// Domain concepts (interfaces, types).
    pub concepts: Vec<CkmConcept>,

    /// User-facing operations (functions).
    pub operations: Vec<CkmOperation>,

    /// Enforced rules.
    pub constraints: Vec<CkmConstraint>,

    /// Multi-step workflows.
    pub workflows: Vec<CkmWorkflow>,

    /// Configuration schema entries.
    pub config_schema: Vec<CkmConfigEntry>,
}

// ────────────────────────────────────────────────────────────────────────────
// Section 3: Derived Types (computed by the engine)
// ────────────────────────────────────────────────────────────────────────────

/// An auto-derived topic grouping related concepts, operations, config, and constraints.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CkmTopic {
    /// Slug used as CLI argument (e.g., "calver").
    pub name: String,

    /// One-line description (from the primary concept).
    pub summary: String,

    /// Related concepts.
    pub concepts: Vec<CkmConcept>,

    /// Related operations.
    pub operations: Vec<CkmOperation>,

    /// Related config entries.
    pub config_schema: Vec<CkmConfigEntry>,

    /// Related constraints.
    pub constraints: Vec<CkmConstraint>,
}

/// A summary entry for the topic index.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CkmTopicIndexEntry {
    /// Topic slug.
    pub name: String,

    /// One-line description.
    pub summary: String,

    /// Count of related concepts.
    pub concepts: usize,

    /// Count of related operations.
    pub operations: usize,

    /// Count of related config entries.
    pub config_fields: usize,

    /// Count of related constraints.
    pub constraints: usize,
}

/// Aggregate manifest counts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CkmManifestCounts {
    /// Total concepts in manifest.
    pub concepts: usize,

    /// Total operations.
    pub operations: usize,

    /// Total constraints.
    pub constraints: usize,

    /// Total workflows.
    pub workflows: usize,

    /// Total config entries.
    pub config_schema: usize,
}

/// The full topic index returned by `topic_json()` with no argument.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CkmTopicIndex {
    /// All topic summaries.
    pub topics: Vec<CkmTopicIndexEntry>,

    /// Aggregate manifest counts.
    pub ckm: CkmManifestCounts,
}

/// Manifest statistics returned by `inspect()`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CkmInspectResult {
    /// Manifest metadata.
    pub meta: CkmMeta,

    /// Counts of each manifest section.
    pub counts: CkmInspectCounts,

    /// List of derived topic slugs.
    pub topic_names: Vec<String>,
}

/// Counts for the inspect result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CkmInspectCounts {
    /// Number of concepts.
    pub concepts: usize,
    /// Number of operations.
    pub operations: usize,
    /// Number of constraints.
    pub constraints: usize,
    /// Number of workflows.
    pub workflows: usize,
    /// Number of config keys.
    pub config_keys: usize,
    /// Number of derived topics.
    pub topics: usize,
}

/// A single validation error with a JSON pointer path.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CkmValidationError {
    /// JSON pointer path (e.g., "/concepts/0/slug").
    pub path: String,

    /// Human-readable error message.
    pub message: String,
}

/// Result of manifest validation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CkmValidationResult {
    /// Whether the manifest is valid.
    pub valid: bool,

    /// Validation errors (empty if valid).
    pub errors: Vec<CkmValidationError>,
}

/// Error returned when a topic is not found.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CkmErrorResult {
    /// Error message (e.g., "Unknown topic: foo").
    pub error: String,

    /// Available topic names for suggestion.
    pub topics: Vec<String>,
}

/// The result of `topic_json()` — either a topic index, a single topic, or an error.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TopicJsonResult {
    /// Full topic index (when no topic name is given).
    Index(CkmTopicIndex),

    /// Single topic detail (when topic name matches).
    Topic(CkmTopic),

    /// Error result (when topic name does not match).
    Error(CkmErrorResult),
}
