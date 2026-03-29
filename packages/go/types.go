// Package ckm provides a Go SDK for consuming CKM (Codebase Knowledge Manifest) files.
//
// This is a thin wrapper around the Rust ckm-core library, accessed via C FFI.
// All algorithms (topic derivation, migration, validation, formatting) live in
// Rust. Go types mirror the Rust types and are populated by deserializing JSON
// returned from the FFI boundary.
package ckm

// CanonicalType represents a portable primitive type mapped to JSON Schema primitives.
type CanonicalType string

const (
	CanonicalString  CanonicalType = "string"
	CanonicalBoolean CanonicalType = "boolean"
	CanonicalNumber  CanonicalType = "number"
	CanonicalInteger CanonicalType = "integer"
	CanonicalArray   CanonicalType = "array"
	CanonicalObject  CanonicalType = "object"
	CanonicalNull    CanonicalType = "null"
	CanonicalAny     CanonicalType = "any"
)

// TypeRef is a portable type reference with canonical mapping.
type TypeRef struct {
	Canonical CanonicalType `json:"canonical"`
	Original  *string       `json:"original,omitempty"`
	Enum      []string      `json:"enum,omitempty"`
}

// Property represents a property within a Concept.
type Property struct {
	Name        string  `json:"name"`
	Type        TypeRef `json:"type"`
	Description string  `json:"description"`
	Required    bool    `json:"required"`
	Default     *string `json:"default"`
}

// Concept represents a domain concept extracted from source code.
type Concept struct {
	ID         string     `json:"id"`
	Name       string     `json:"name"`
	Slug       string     `json:"slug"`
	What       string     `json:"what"`
	Tags       []string   `json:"tags"`
	Properties []Property `json:"properties,omitempty"`
}

// Input represents a function parameter within an Operation.
type Input struct {
	Name        string  `json:"name"`
	Type        TypeRef `json:"type"`
	Required    bool    `json:"required"`
	Description string  `json:"description"`
}

// Output represents a return value from an Operation.
type Output struct {
	Type        TypeRef `json:"type"`
	Description string  `json:"description"`
}

// Operation represents a user-facing operation extracted from source code.
type Operation struct {
	ID      string   `json:"id"`
	Name    string   `json:"name"`
	What    string   `json:"what"`
	Tags    []string `json:"tags"`
	Inputs  []Input  `json:"inputs,omitempty"`
	Outputs *Output  `json:"outputs,omitempty"`
}

// Constraint represents a rule enforced by the tool.
type Constraint struct {
	ID         string `json:"id"`
	Rule       string `json:"rule"`
	EnforcedBy string `json:"enforcedBy"`
	Severity   string `json:"severity"`
}

// WorkflowStep represents a single step within a Workflow.
type WorkflowStep struct {
	Action string  `json:"action"`
	Value  string  `json:"value"`
	Note   *string `json:"note,omitempty"`
}

// Workflow represents a multi-step workflow for achieving a common goal.
type Workflow struct {
	ID    string         `json:"id"`
	Goal  string         `json:"goal"`
	Tags  []string       `json:"tags"`
	Steps []WorkflowStep `json:"steps"`
}

// ConfigEntry represents a configuration schema entry.
type ConfigEntry struct {
	Key         string  `json:"key"`
	Type        TypeRef `json:"type"`
	Description string  `json:"description"`
	Default     *string `json:"default"`
	Required    bool    `json:"required"`
}

// Meta contains provenance metadata about the manifest source.
type Meta struct {
	Project   string  `json:"project"`
	Language  string  `json:"language"`
	Generator string  `json:"generator"`
	Generated string  `json:"generated"`
	SourceURL *string `json:"sourceUrl,omitempty"`
}

// Manifest is the top-level CKM manifest object (v2).
type Manifest struct {
	Schema       string        `json:"$schema"`
	Version      string        `json:"version"`
	Meta         Meta          `json:"meta"`
	Concepts     []Concept     `json:"concepts"`
	Operations   []Operation   `json:"operations"`
	Constraints  []Constraint  `json:"constraints"`
	Workflows    []Workflow    `json:"workflows"`
	ConfigSchema []ConfigEntry `json:"configSchema"`
}

// Topic is an auto-derived topic grouping related concepts, operations, config, and constraints.
type Topic struct {
	Name         string        `json:"name"`
	Summary      string        `json:"summary"`
	Concepts     []Concept     `json:"concepts"`
	Operations   []Operation   `json:"operations"`
	ConfigSchema []ConfigEntry `json:"configSchema"`
	Constraints  []Constraint  `json:"constraints"`
}

// TopicIndexEntry is a summary entry for the topic index.
type TopicIndexEntry struct {
	Name         string `json:"name"`
	Summary      string `json:"summary"`
	Concepts     int    `json:"concepts"`
	Operations   int    `json:"operations"`
	ConfigFields int    `json:"configFields"`
	Constraints  int    `json:"constraints"`
}

// TopicIndexCounts holds aggregate manifest item counts.
type TopicIndexCounts struct {
	Concepts     int `json:"concepts"`
	Operations   int `json:"operations"`
	Constraints  int `json:"constraints"`
	Workflows    int `json:"workflows"`
	ConfigSchema int `json:"configSchema"`
}

// TopicIndex is the full topic index returned by TopicJSON with no argument.
type TopicIndex struct {
	Topics []TopicIndexEntry `json:"topics"`
	CKM    TopicIndexCounts  `json:"ckm"`
}

// InspectCounts holds per-section counts for the inspect result.
type InspectCounts struct {
	Concepts    int `json:"concepts"`
	Operations  int `json:"operations"`
	Constraints int `json:"constraints"`
	Workflows   int `json:"workflows"`
	ConfigKeys  int `json:"configKeys"`
	Topics      int `json:"topics"`
}

// InspectResult contains manifest statistics returned by Inspect.
type InspectResult struct {
	Meta       Meta          `json:"meta"`
	Counts     InspectCounts `json:"counts"`
	TopicNames []string      `json:"topicNames"`
}

// ValidationError represents a single validation error.
type ValidationError struct {
	Path    string `json:"path"`
	Message string `json:"message"`
}

// ValidationResult is the result of manifest validation.
type ValidationResult struct {
	Valid  bool              `json:"valid"`
	Errors []ValidationError `json:"errors"`
}

// ErrorResult is returned when a topic is not found.
type ErrorResult struct {
	Error  string   `json:"error"`
	Topics []string `json:"topics"`
}
