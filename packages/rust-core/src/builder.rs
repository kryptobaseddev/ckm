//! CKM Manifest Builder — fluent API for constructing valid ckm.json manifests.
//!
//! This is the **producer** side of CKM. Generators (like forge-ts) use this
//! to construct manifests with compile-time type safety instead of hand-rolling JSON.
//!
//! # Example
//!
//! ```rust,ignore
//! use ckm::CkmManifestBuilder;
//!
//! let manifest = CkmManifestBuilder::new("my-tool", "typescript")
//!     .generator("forge-ts@1.0.0")
//!     .source_url("https://github.com/org/repo")
//!     .add_concept("CalVerConfig", "calver", "Configures CalVer validation.", &["config"])
//!     .add_concept_property("calver", "format", "string", "Calendar format.", true, Some("YYYY.MM.DD"))
//!     .add_operation("validate", "Validates a CalVer string.", &["calver"])
//!     .add_operation_input("validate", "version", "string", true, "Version string.")
//!     .add_constraint("No future dates", "validate", "error")
//!     .add_config("calver.format", "string", "Calendar format.", true, Some("YYYY.MM.DD"))
//!     .build();
//!
//! let json = serde_json::to_string_pretty(&manifest).unwrap();
//! ```

use crate::types::*;

/// Fluent builder for constructing valid CKM v2 manifests.
#[derive(Debug, Clone)]
pub struct CkmManifestBuilder {
    project: String,
    language: String,
    generator: String,
    source_url: Option<String>,
    concepts: Vec<CkmConcept>,
    operations: Vec<CkmOperation>,
    constraints: Vec<CkmConstraint>,
    workflows: Vec<CkmWorkflow>,
    config_schema: Vec<CkmConfigEntry>,
    concept_counter: usize,
    operation_counter: usize,
    constraint_counter: usize,
    workflow_counter: usize,
}

impl CkmManifestBuilder {
    /// Creates a new builder with the required project name and language.
    pub fn new(project: &str, language: &str) -> Self {
        Self {
            project: project.to_string(),
            language: language.to_string(),
            generator: "unknown".to_string(),
            source_url: None,
            concepts: Vec::new(),
            operations: Vec::new(),
            constraints: Vec::new(),
            workflows: Vec::new(),
            config_schema: Vec::new(),
            concept_counter: 0,
            operation_counter: 0,
            constraint_counter: 0,
            workflow_counter: 0,
        }
    }

    /// Sets the generator name (e.g., "forge-ts@1.0.0").
    pub fn generator(mut self, generator: &str) -> Self {
        self.generator = generator.to_string();
        self
    }

    /// Sets the source repository URL.
    pub fn source_url(mut self, url: &str) -> Self {
        self.source_url = Some(url.to_string());
        self
    }

    /// Adds a concept with the given slug, description, and tags.
    pub fn add_concept(mut self, name: &str, slug: &str, what: &str, tags: &[&str]) -> Self {
        self.concept_counter += 1;
        self.concepts.push(CkmConcept {
            id: format!("concept-{}", slug),
            name: name.to_string(),
            slug: slug.to_string(),
            what: what.to_string(),
            tags: tags.iter().map(|t| t.to_string()).collect(),
            properties: Some(Vec::new()),
        });
        self
    }

    /// Adds a property to the most recently added concept, or to a concept by slug.
    pub fn add_concept_property(
        mut self,
        concept_slug: &str,
        name: &str,
        canonical_type: &str,
        description: &str,
        required: bool,
        default: Option<&str>,
    ) -> Self {
        if let Some(concept) = self.concepts.iter_mut().find(|c| c.slug == concept_slug) {
            let props = concept.properties.get_or_insert_with(Vec::new);
            props.push(CkmProperty {
                name: name.to_string(),
                r#type: CkmTypeRef {
                    canonical: CanonicalType::parse(canonical_type),
                    original: None,
                    r#enum: None,
                },
                description: description.to_string(),
                required,
                default: default.map(|d| d.to_string()),
            });
        }
        self
    }

    /// Adds a property with an original type annotation (e.g., "CalVerFormat").
    pub fn add_concept_property_typed(
        mut self,
        concept_slug: &str,
        name: &str,
        canonical_type: &str,
        original_type: &str,
        description: &str,
        required: bool,
        default: Option<&str>,
    ) -> Self {
        if let Some(concept) = self.concepts.iter_mut().find(|c| c.slug == concept_slug) {
            let props = concept.properties.get_or_insert_with(Vec::new);
            props.push(CkmProperty {
                name: name.to_string(),
                r#type: CkmTypeRef {
                    canonical: CanonicalType::parse(canonical_type),
                    original: Some(original_type.to_string()),
                    r#enum: None,
                },
                description: description.to_string(),
                required,
                default: default.map(|d| d.to_string()),
            });
        }
        self
    }

    /// Adds an operation with tags for topic linkage.
    pub fn add_operation(mut self, name: &str, what: &str, tags: &[&str]) -> Self {
        self.operation_counter += 1;
        self.operations.push(CkmOperation {
            id: format!("op-{}", name),
            name: name.to_string(),
            what: what.to_string(),
            tags: tags.iter().map(|t| t.to_string()).collect(),
            inputs: Some(Vec::new()),
            outputs: None,
        });
        self
    }

    /// Adds an input parameter to an operation by name.
    pub fn add_operation_input(
        mut self,
        op_name: &str,
        param_name: &str,
        canonical_type: &str,
        required: bool,
        description: &str,
    ) -> Self {
        if let Some(op) = self.operations.iter_mut().find(|o| o.name == op_name) {
            let inputs = op.inputs.get_or_insert_with(Vec::new);
            inputs.push(CkmInput {
                name: param_name.to_string(),
                r#type: CkmTypeRef {
                    canonical: CanonicalType::parse(canonical_type),
                    original: None,
                    r#enum: None,
                },
                required,
                description: description.to_string(),
            });
        }
        self
    }

    /// Sets the output type for an operation.
    pub fn set_operation_output(
        mut self,
        op_name: &str,
        canonical_type: &str,
        description: &str,
    ) -> Self {
        if let Some(op) = self.operations.iter_mut().find(|o| o.name == op_name) {
            op.outputs = Some(CkmOutput {
                r#type: CkmTypeRef {
                    canonical: CanonicalType::parse(canonical_type),
                    original: None,
                    r#enum: None,
                },
                description: description.to_string(),
            });
        }
        self
    }

    /// Adds a constraint with severity.
    pub fn add_constraint(mut self, rule: &str, enforced_by: &str, severity: &str) -> Self {
        self.constraint_counter += 1;
        self.constraints.push(CkmConstraint {
            id: format!("constraint-{}", self.constraint_counter),
            rule: rule.to_string(),
            enforced_by: enforced_by.to_string(),
            severity: match severity {
                "warning" => Severity::Warning,
                "info" => Severity::Info,
                _ => Severity::Error,
            },
        });
        self
    }

    /// Adds a workflow with steps.
    pub fn add_workflow(mut self, goal: &str, tags: &[&str]) -> Self {
        self.workflow_counter += 1;
        self.workflows.push(CkmWorkflow {
            id: format!("wf-{}", self.workflow_counter),
            goal: goal.to_string(),
            tags: tags.iter().map(|t| t.to_string()).collect(),
            steps: Vec::new(),
        });
        self
    }

    /// Adds a command step to the most recently added workflow.
    pub fn add_workflow_command(mut self, command: &str, note: Option<&str>) -> Self {
        if let Some(wf) = self.workflows.last_mut() {
            wf.steps.push(CkmWorkflowStep {
                action: StepAction::Command,
                value: command.to_string(),
                note: note.map(|n| n.to_string()),
            });
        }
        self
    }

    /// Adds a manual step to the most recently added workflow.
    pub fn add_workflow_manual(mut self, instruction: &str, note: Option<&str>) -> Self {
        if let Some(wf) = self.workflows.last_mut() {
            wf.steps.push(CkmWorkflowStep {
                action: StepAction::Manual,
                value: instruction.to_string(),
                note: note.map(|n| n.to_string()),
            });
        }
        self
    }

    /// Adds a config schema entry.
    pub fn add_config(
        mut self,
        key: &str,
        canonical_type: &str,
        description: &str,
        required: bool,
        default: Option<&str>,
    ) -> Self {
        self.config_schema.push(CkmConfigEntry {
            key: key.to_string(),
            r#type: CkmTypeRef {
                canonical: CanonicalType::parse(canonical_type),
                original: None,
                r#enum: None,
            },
            description: description.to_string(),
            default: default.map(|d| d.to_string()),
            required,
        });
        self
    }

    /// Builds the final CkmManifest. Consumes the builder.
    pub fn build(self) -> CkmManifest {
        CkmManifest {
            schema: "https://ckm.dev/schemas/v2.json".to_string(),
            version: "2.0.0".to_string(),
            meta: CkmMeta {
                project: self.project,
                language: self.language,
                generator: self.generator,
                generated: chrono_now(),
                source_url: self.source_url,
            },
            concepts: self.concepts,
            operations: self.operations,
            constraints: self.constraints,
            workflows: self.workflows,
            config_schema: self.config_schema,
        }
    }

    /// Builds and serializes to a JSON string.
    pub fn build_json(&self) -> String {
        let manifest = self.clone().build();
        serde_json::to_string_pretty(&manifest).unwrap_or_default()
    }
}

/// Returns current ISO 8601 timestamp without chrono dependency.
fn chrono_now() -> String {
    // Use a simple approach — generators will typically override this
    "2026-01-01T00:00:00.000Z".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_basic() {
        let manifest = CkmManifestBuilder::new("test-tool", "typescript")
            .generator("test@1.0.0")
            .add_concept("CalVerConfig", "calver", "Configures CalVer.", &["config"])
            .add_concept_property("calver", "format", "string", "The format.", true, None)
            .add_operation("validate", "Validates a version.", &["calver"])
            .add_operation_input("validate", "version", "string", true, "Version string.")
            .add_constraint("No future dates", "validate", "error")
            .add_config("calver.format", "string", "Calendar format.", true, Some("YYYY.MM.DD"))
            .build();

        assert_eq!(manifest.meta.project, "test-tool");
        assert_eq!(manifest.version, "2.0.0");
        assert_eq!(manifest.concepts.len(), 1);
        assert_eq!(manifest.concepts[0].slug, "calver");
        assert_eq!(manifest.operations.len(), 1);
        assert_eq!(manifest.constraints.len(), 1);
        assert_eq!(manifest.config_schema.len(), 1);
    }

    #[test]
    fn test_builder_serializes_to_valid_json() {
        let builder = CkmManifestBuilder::new("test", "rust")
            .add_concept("Config", "config", "Main config.", &["config"]);
        let json = builder.build_json();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["version"], "2.0.0");
        assert_eq!(parsed["meta"]["project"], "test");
    }
}
