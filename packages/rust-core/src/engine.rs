//! CKM engine — auto-generates topic index from a ckm.json manifest.
//!
//! Implements SPEC.md algorithms for topic derivation, JSON output, and
//! terminal formatting. Handles both v1 and v2 manifests transparently.

use serde_json::Value;

use crate::format::{format_topic_content, format_topic_index};
use crate::migrate::{derive_slug, detect_version, migrate_v1_to_v2};
use crate::types::{
    CkmConcept, CkmErrorResult, CkmInspectCounts, CkmInspectResult, CkmManifest, CkmManifestCounts,
    CkmTopic, CkmTopicIndex, CkmTopicIndexEntry, TopicJsonResult,
};

/// The core CKM engine.
///
/// Consumes a manifest (v1 or v2), derives topics at construction time,
/// and exposes methods for progressive disclosure at four levels.
#[derive(Debug, Clone)]
pub struct CkmEngine {
    manifest: CkmManifest,
    derived_topics: Vec<CkmTopic>,
}

impl CkmEngine {
    /// Creates a new CKM engine from a parsed JSON value.
    ///
    /// If the input is a v1 manifest, it is automatically migrated to v2.
    /// Topics are derived at construction time.
    pub fn new(data: Value) -> Self {
        let version = detect_version(&data);
        let manifest = if version == 1 {
            migrate_v1_to_v2(&data)
        } else {
            serde_json::from_value(data).unwrap_or_else(|_| CkmManifest {
                schema: String::new(),
                version: "2.0.0".to_string(),
                meta: crate::types::CkmMeta {
                    project: "unknown".to_string(),
                    language: "unknown".to_string(),
                    generator: "unknown".to_string(),
                    generated: String::new(),
                    source_url: None,
                },
                concepts: Vec::new(),
                operations: Vec::new(),
                constraints: Vec::new(),
                workflows: Vec::new(),
                config_schema: Vec::new(),
            })
        };

        let derived_topics = derive_topics(&manifest);

        CkmEngine {
            manifest,
            derived_topics,
        }
    }

    /// All auto-derived topics, computed at construction time.
    pub fn topics(&self) -> &[CkmTopic] {
        &self.derived_topics
    }

    /// Returns formatted topic index for terminal display (Level 0).
    ///
    /// Output stays within 300 token budget.
    pub fn topic_index(&self, tool_name: &str) -> String {
        format_topic_index(&self.derived_topics, tool_name)
    }

    /// Returns human-readable content for a specific topic (Level 1).
    ///
    /// Returns `None` if the topic is not found.
    /// Output stays within 800 token budget.
    pub fn topic_content(&self, topic_name: &str) -> Option<String> {
        format_topic_content(&self.derived_topics, topic_name)
    }

    /// Returns structured JSON data for a topic or the full index.
    ///
    /// - If `topic_name` is `None`: returns `TopicJsonResult::Index` (Level 2)
    /// - If `topic_name` matches a topic: returns `TopicJsonResult::Topic` (Level 1J)
    /// - If `topic_name` does not match: returns `TopicJsonResult::Error`
    pub fn topic_json(&self, topic_name: Option<&str>) -> TopicJsonResult {
        match topic_name {
            None => TopicJsonResult::Index(self.build_topic_index_json()),
            Some(name) => self.build_topic_json(name),
        }
    }

    /// Returns the raw manifest (v2, possibly migrated from v1).
    pub fn manifest(&self) -> &CkmManifest {
        &self.manifest
    }

    /// Returns manifest statistics: metadata, counts, and topic names.
    pub fn inspect(&self) -> CkmInspectResult {
        CkmInspectResult {
            meta: self.manifest.meta.clone(),
            counts: CkmInspectCounts {
                concepts: self.manifest.concepts.len(),
                operations: self.manifest.operations.len(),
                constraints: self.manifest.constraints.len(),
                workflows: self.manifest.workflows.len(),
                config_keys: self.manifest.config_schema.len(),
                topics: self.derived_topics.len(),
            },
            topic_names: self.derived_topics.iter().map(|t| t.name.clone()).collect(),
        }
    }

    fn build_topic_index_json(&self) -> CkmTopicIndex {
        CkmTopicIndex {
            topics: self
                .derived_topics
                .iter()
                .map(|t| CkmTopicIndexEntry {
                    name: t.name.clone(),
                    summary: t.summary.clone(),
                    concepts: t.concepts.len(),
                    operations: t.operations.len(),
                    config_fields: t.config_schema.len(),
                    constraints: t.constraints.len(),
                })
                .collect(),
            ckm: CkmManifestCounts {
                concepts: self.manifest.concepts.len(),
                operations: self.manifest.operations.len(),
                constraints: self.manifest.constraints.len(),
                workflows: self.manifest.workflows.len(),
                config_schema: self.manifest.config_schema.len(),
            },
        }
    }

    fn build_topic_json(&self, topic_name: &str) -> TopicJsonResult {
        match self.derived_topics.iter().find(|t| t.name == topic_name) {
            Some(topic) => TopicJsonResult::Topic(topic.clone()),
            None => TopicJsonResult::Error(CkmErrorResult {
                error: format!("Unknown topic: {}", topic_name),
                topics: self.derived_topics.iter().map(|t| t.name.clone()).collect(),
            }),
        }
    }
}

// ─── Topic Derivation (SPEC.md Section 3 — revised) ────────────────────
//
// Every concept with a non-empty slug becomes a topic.
// Operations/constraints/config are matched by tag overlap or keyword.
// Unmatched operations get their own topics so nothing is invisible.

fn derive_topics(manifest: &CkmManifest) -> Vec<CkmTopic> {
    let mut topics: Vec<CkmTopic> = Vec::new();
    let mut claimed_op_ids: Vec<String> = Vec::new();
    let mut claimed_constraint_ids: Vec<String> = Vec::new();

    // Phase 1: Every concept with a slug becomes a topic
    for concept in &manifest.concepts {
        let slug = &concept.slug;
        if slug.is_empty() {
            continue;
        }

        // Skip if we already have a topic with this slug (dedup)
        if topics.iter().any(|t| t.name == *slug) {
            // Merge this concept into the existing topic
            if let Some(existing) = topics.iter_mut().find(|t| t.name == *slug) {
                existing.concepts.push(concept.clone());
            }
            continue;
        }

        // Collect related concepts (same slug or name contains slug)
        let mut related_concepts: Vec<CkmConcept> = vec![concept.clone()];
        for other in &manifest.concepts {
            if other.id == concept.id {
                continue;
            }
            if other.slug == *slug {
                related_concepts.push(other.clone());
            } else {
                let other_slug = derive_slug(&other.name);
                if other.name.to_lowercase().contains(slug) || slug.contains(&other_slug) {
                    related_concepts.push(other.clone());
                }
            }
        }
        let concept_names: Vec<String> = related_concepts.iter().map(|c| c.name.clone()).collect();

        // Match operations by tag overlap or keyword
        let matched_operations: Vec<_> = manifest
            .operations
            .iter()
            .filter(|op| {
                if op.tags.iter().any(|t| t.to_lowercase() == slug.to_lowercase()) {
                    return true;
                }
                matches_by_keyword(&op.name, &op.what, slug, &concept_names)
            })
            .cloned()
            .collect();
        for op in &matched_operations {
            claimed_op_ids.push(op.id.clone());
        }

        // Match config entries by key prefix
        let matched_config: Vec<_> = manifest
            .config_schema
            .iter()
            .filter(|entry| {
                let key_prefix = entry.key.split('.').next().unwrap_or("");
                key_prefix == slug
            })
            .cloned()
            .collect();

        // Match constraints by enforcedBy referencing concepts or matched operations
        let matched_constraints: Vec<_> = manifest
            .constraints
            .iter()
            .filter(|c| {
                if concept_names.iter().any(|name| c.enforced_by.contains(name.as_str())) {
                    return true;
                }
                matched_operations.iter().any(|op| c.enforced_by.contains(op.name.as_str()))
            })
            .cloned()
            .collect();
        for c in &matched_constraints {
            claimed_constraint_ids.push(c.id.clone());
        }

        topics.push(CkmTopic {
            name: slug.clone(),
            summary: concept.what.clone(),
            concepts: related_concepts,
            operations: matched_operations,
            config_schema: matched_config,
            constraints: matched_constraints,
        });
    }

    // Phase 2: Unclaimed operations get their own topics
    for op in &manifest.operations {
        if claimed_op_ids.contains(&op.id) {
            continue;
        }
        let slug = derive_slug(&op.name);
        if slug.is_empty() {
            continue;
        }
        // If a topic with this slug already exists, add the operation to it
        if let Some(existing) = topics.iter_mut().find(|t| t.name == slug) {
            existing.operations.push(op.clone());
            claimed_op_ids.push(op.id.clone());
            continue;
        }
        // Create a new topic for this operation
        topics.push(CkmTopic {
            name: slug.clone(),
            summary: op.what.clone(),
            concepts: Vec::new(),
            operations: vec![op.clone()],
            config_schema: Vec::new(),
            constraints: Vec::new(),
        });
        claimed_op_ids.push(op.id.clone());
    }

    // Phase 3: Unclaimed constraints get added to matching topics or their own
    for constraint in &manifest.constraints {
        if claimed_constraint_ids.contains(&constraint.id) {
            continue;
        }
        // Try to find a topic whose operations match enforcedBy
        let mut matched = false;
        for topic in &mut topics {
            if topic.operations.iter().any(|op| constraint.enforced_by.contains(op.name.as_str())) {
                topic.constraints.push(constraint.clone());
                matched = true;
                break;
            }
        }
        if !matched {
            // Add to first topic or create an "other" topic
            if let Some(first) = topics.first_mut() {
                first.constraints.push(constraint.clone());
            }
        }
    }

    topics
}

/// Checks if a name+description matches a slug or concept names by keyword.
fn matches_by_keyword(name: &str, what: &str, slug: &str, concept_names: &[String]) -> bool {
    let haystack = format!("{} {}", name, what).to_lowercase();
    if haystack.contains(slug) {
        return true;
    }
    concept_names
        .iter()
        .any(|n| haystack.contains(&n.to_lowercase()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_v2() -> Value {
        serde_json::json!({
            "$schema": "https://ckm.dev/schemas/v2.json",
            "version": "2.0.0",
            "meta": {
                "project": "test",
                "language": "typescript",
                "generator": "hand-authored",
                "generated": "2026-01-01T00:00:00.000Z"
            },
            "concepts": [
                {
                    "id": "concept-calver-config",
                    "name": "CalVerConfig",
                    "slug": "calver",
                    "what": "Configures CalVer validation rules.",
                    "tags": ["config"],
                    "properties": [
                        {
                            "name": "format",
                            "type": { "canonical": "string", "original": "CalVerFormat" },
                            "description": "Calendar format.",
                            "required": true,
                            "default": "YYYY.MM.DD"
                        }
                    ]
                }
            ],
            "operations": [
                {
                    "id": "op-validate",
                    "name": "validate",
                    "what": "Validates a calver version string.",
                    "tags": ["calver"],
                    "inputs": [
                        {
                            "name": "version",
                            "type": { "canonical": "string" },
                            "required": true,
                            "description": "The version string."
                        }
                    ]
                }
            ],
            "constraints": [
                {
                    "id": "constraint-no-future",
                    "rule": "No future dates.",
                    "enforcedBy": "validate",
                    "severity": "error"
                }
            ],
            "workflows": [],
            "configSchema": [
                {
                    "key": "calver.format",
                    "type": { "canonical": "string", "original": "CalVerFormat" },
                    "description": "Calendar format.",
                    "default": "YYYY.MM.DD",
                    "required": true
                }
            ]
        })
    }

    #[test]
    fn test_engine_new() {
        let engine = CkmEngine::new(minimal_v2());
        assert_eq!(engine.topics().len(), 1);
        assert_eq!(engine.topics()[0].name, "calver");
    }

    #[test]
    fn test_engine_topic_index() {
        let engine = CkmEngine::new(minimal_v2());
        let output = engine.topic_index("my-tool");
        assert!(output.contains("my-tool CKM"));
        assert!(output.contains("calver"));
    }

    #[test]
    fn test_engine_topic_content() {
        let engine = CkmEngine::new(minimal_v2());
        let output = engine.topic_content("calver").unwrap();
        assert!(output.contains("CalVerConfig"));
        assert!(engine.topic_content("nonexistent").is_none());
    }

    #[test]
    fn test_engine_topic_json_index() {
        let engine = CkmEngine::new(minimal_v2());
        match engine.topic_json(None) {
            TopicJsonResult::Index(idx) => {
                assert_eq!(idx.topics.len(), 1);
                assert_eq!(idx.topics[0].name, "calver");
            }
            _ => panic!("Expected TopicJsonResult::Index"),
        }
    }

    #[test]
    fn test_engine_topic_json_single() {
        let engine = CkmEngine::new(minimal_v2());
        match engine.topic_json(Some("calver")) {
            TopicJsonResult::Topic(t) => {
                assert_eq!(t.name, "calver");
            }
            _ => panic!("Expected TopicJsonResult::Topic"),
        }
    }

    #[test]
    fn test_engine_topic_json_error() {
        let engine = CkmEngine::new(minimal_v2());
        match engine.topic_json(Some("nonexistent")) {
            TopicJsonResult::Error(e) => {
                assert!(e.error.contains("Unknown topic"));
                assert!(e.topics.contains(&"calver".to_string()));
            }
            _ => panic!("Expected TopicJsonResult::Error"),
        }
    }

    #[test]
    fn test_engine_manifest() {
        let engine = CkmEngine::new(minimal_v2());
        assert_eq!(engine.manifest().meta.project, "test");
    }

    #[test]
    fn test_engine_inspect() {
        let engine = CkmEngine::new(minimal_v2());
        let result = engine.inspect();
        assert_eq!(result.meta.project, "test");
        assert_eq!(result.counts.concepts, 1);
        assert_eq!(result.counts.operations, 1);
        assert_eq!(result.counts.topics, 1);
        assert_eq!(result.topic_names, vec!["calver"]);
    }

    #[test]
    fn test_engine_v1_auto_migration() {
        let v1 = serde_json::json!({
            "project": "legacy",
            "generated": "2025-01-01T00:00:00.000Z",
            "concepts": [
                {
                    "id": "concept-CalVerConfig",
                    "name": "CalVerConfig",
                    "what": "Configures CalVer.",
                    "properties": [
                        {
                            "name": "format",
                            "type": "CalVerFormat",
                            "description": "The format."
                        }
                    ]
                }
            ],
            "operations": [],
            "constraints": [],
            "workflows": [],
            "configSchema": []
        });
        let engine = CkmEngine::new(v1);
        assert_eq!(engine.manifest().version, "2.0.0");
        assert_eq!(engine.manifest().meta.project, "legacy");
        assert_eq!(engine.topics().len(), 1);
        assert_eq!(engine.topics()[0].name, "calver");
    }

    #[test]
    fn test_engine_empty_manifest() {
        let data = serde_json::json!({
            "$schema": "https://ckm.dev/schemas/v2.json",
            "version": "2.0.0",
            "meta": {
                "project": "empty",
                "language": "rust",
                "generator": "hand-authored",
                "generated": "2026-01-01T00:00:00.000Z"
            },
            "concepts": [],
            "operations": [],
            "constraints": [],
            "workflows": [],
            "configSchema": []
        });
        let engine = CkmEngine::new(data);
        assert!(engine.topics().is_empty());
        assert_eq!(engine.inspect().counts.topics, 0);
    }
}
