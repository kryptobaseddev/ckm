//! CKM conformance tests.
//!
//! Loads fixtures from `../../conformance/fixtures/` and verifies that the
//! engine, migration, and validation produce correct results on all fixtures.

use std::fs;
use std::path::PathBuf;

use ckm::{CkmEngine, TopicJsonResult, detect_version, migrate_v1_to_v2, validate_manifest};

/// Returns the path to the conformance fixtures directory.
fn fixtures_dir() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.join("../../conformance/fixtures")
}

/// Loads a fixture file by name.
fn load_fixture(name: &str) -> serde_json::Value {
    let path = fixtures_dir().join(name);
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read fixture {}: {}", path.display(), e));
    serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("Failed to parse fixture {}: {}", path.display(), e))
}

// ─── Version Detection ──────────────────────────────────────────────────

#[test]
fn test_detect_version_v2_minimal() {
    let data = load_fixture("minimal.ckm.json");
    assert_eq!(detect_version(&data), 2);
}

#[test]
fn test_detect_version_v2_multi_topic() {
    let data = load_fixture("multi-topic.ckm.json");
    assert_eq!(detect_version(&data), 2);
}

#[test]
fn test_detect_version_v2_polyglot() {
    let data = load_fixture("polyglot.ckm.json");
    assert_eq!(detect_version(&data), 2);
}

#[test]
fn test_detect_version_v2_edge_cases() {
    let data = load_fixture("edge-cases.ckm.json");
    assert_eq!(detect_version(&data), 2);
}

#[test]
fn test_detect_version_v1_legacy() {
    let data = load_fixture("v1-legacy.ckm.json");
    assert_eq!(detect_version(&data), 1);
}

// ─── Engine: Minimal Fixture ────────────────────────────────────────────

#[test]
fn test_engine_minimal_topic_count() {
    let data = load_fixture("minimal.ckm.json");
    let engine = CkmEngine::new(data);
    assert_eq!(engine.topics().len(), 1);
    assert_eq!(engine.topics()[0].name, "calver");
}

#[test]
fn test_engine_minimal_topic_content() {
    let data = load_fixture("minimal.ckm.json");
    let engine = CkmEngine::new(data);

    let content = engine.topics();
    assert_eq!(content[0].concepts.len(), 1);
    assert_eq!(content[0].operations.len(), 1);
    assert_eq!(content[0].config_schema.len(), 1);
    assert_eq!(content[0].constraints.len(), 1);
}

#[test]
fn test_engine_minimal_topic_index() {
    let data = load_fixture("minimal.ckm.json");
    let engine = CkmEngine::new(data);

    let index = engine.topic_index("test-tool");
    assert!(index.contains("test-tool CKM"));
    assert!(index.contains("calver"));
    assert!(index.contains("Topics:"));
    assert!(index.contains("--json"));
    // Level 0 token budget: max 300 tokens (~1200 chars)
    assert!(
        index.len() <= 1200,
        "Topic index exceeds 1200 char budget: {} chars",
        index.len()
    );
}

#[test]
fn test_engine_minimal_topic_content_formatted() {
    let data = load_fixture("minimal.ckm.json");
    let engine = CkmEngine::new(data);

    let content = engine.topic_content("calver").unwrap();
    assert!(content.contains("## Concepts"));
    assert!(content.contains("CalVerConfig"));
    assert!(content.contains("## Operations"));
    assert!(content.contains("validate()"));
    // Level 1 token budget: max 800 tokens (~3200 chars)
    assert!(
        content.len() <= 3200,
        "Topic content exceeds 3200 char budget: {} chars",
        content.len()
    );
}

#[test]
fn test_engine_minimal_topic_content_not_found() {
    let data = load_fixture("minimal.ckm.json");
    let engine = CkmEngine::new(data);
    assert!(engine.topic_content("nonexistent").is_none());
}

#[test]
fn test_engine_minimal_topic_json_index() {
    let data = load_fixture("minimal.ckm.json");
    let engine = CkmEngine::new(data);

    match engine.topic_json(None) {
        TopicJsonResult::Index(idx) => {
            assert_eq!(idx.topics.len(), 1);
            assert_eq!(idx.topics[0].name, "calver");
            assert_eq!(idx.ckm.concepts, 1);
            assert_eq!(idx.ckm.operations, 1);
        }
        other => panic!("Expected Index, got {:?}", other),
    }
}

#[test]
fn test_engine_minimal_topic_json_single() {
    let data = load_fixture("minimal.ckm.json");
    let engine = CkmEngine::new(data);

    match engine.topic_json(Some("calver")) {
        TopicJsonResult::Topic(t) => {
            assert_eq!(t.name, "calver");
            assert!(!t.concepts.is_empty());
        }
        other => panic!("Expected Topic, got {:?}", other),
    }
}

#[test]
fn test_engine_minimal_topic_json_error() {
    let data = load_fixture("minimal.ckm.json");
    let engine = CkmEngine::new(data);

    match engine.topic_json(Some("unknown")) {
        TopicJsonResult::Error(e) => {
            assert!(e.error.contains("Unknown topic: unknown"));
            assert!(e.topics.contains(&"calver".to_string()));
        }
        other => panic!("Expected Error, got {:?}", other),
    }
}

#[test]
fn test_engine_minimal_manifest() {
    let data = load_fixture("minimal.ckm.json");
    let engine = CkmEngine::new(data);
    let manifest = engine.manifest();
    assert_eq!(manifest.meta.project, "minimal-tool");
    assert_eq!(manifest.version, "2.0.0");
}

#[test]
fn test_engine_minimal_inspect() {
    let data = load_fixture("minimal.ckm.json");
    let engine = CkmEngine::new(data);
    let result = engine.inspect();
    assert_eq!(result.meta.project, "minimal-tool");
    assert_eq!(result.counts.concepts, 1);
    assert_eq!(result.counts.operations, 1);
    assert_eq!(result.counts.constraints, 1);
    assert_eq!(result.counts.workflows, 0);
    assert_eq!(result.counts.config_keys, 1);
    assert_eq!(result.counts.topics, 1);
    assert_eq!(result.topic_names, vec!["calver"]);
}

// ─── Engine: Multi-Topic Fixture ────────────────────────────────────────

#[test]
fn test_engine_multi_topic_count() {
    let data = load_fixture("multi-topic.ckm.json");
    let engine = CkmEngine::new(data);
    // 3 config concepts + 1 unclaimed operation topic = 4 topics (all concepts become topics)
    assert!(
        engine.topics().len() >= 3,
        "Expected at least 3 topics, got {}",
        engine.topics().len()
    );
}

#[test]
fn test_engine_multi_topic_names() {
    let data = load_fixture("multi-topic.ckm.json");
    let engine = CkmEngine::new(data);
    let names: Vec<&str> = engine.topics().iter().map(|t| t.name.as_str()).collect();
    assert!(names.contains(&"calver"));
    assert!(names.contains(&"semver"));
    assert!(names.contains(&"git"));
}

#[test]
fn test_engine_multi_topic_calver_ops() {
    let data = load_fixture("multi-topic.ckm.json");
    let engine = CkmEngine::new(data);
    let calver = engine.topics().iter().find(|t| t.name == "calver").unwrap();
    // validateCalVer should match via tag "calver"
    assert!(
        calver
            .operations
            .iter()
            .any(|op| op.name == "validateCalVer")
    );
}

#[test]
fn test_engine_multi_topic_git_ops() {
    let data = load_fixture("multi-topic.ckm.json");
    let engine = CkmEngine::new(data);
    let git = engine.topics().iter().find(|t| t.name == "git").unwrap();
    assert!(git.operations.iter().any(|op| op.name == "createGitTag"));
}

#[test]
fn test_engine_multi_topic_config_matching() {
    let data = load_fixture("multi-topic.ckm.json");
    let engine = CkmEngine::new(data);

    let calver = engine.topics().iter().find(|t| t.name == "calver").unwrap();
    assert!(
        calver
            .config_schema
            .iter()
            .any(|c| c.key == "calver.format")
    );

    let semver = engine.topics().iter().find(|t| t.name == "semver").unwrap();
    assert!(
        semver
            .config_schema
            .iter()
            .any(|c| c.key == "semver.allowVPrefix")
    );

    let git = engine.topics().iter().find(|t| t.name == "git").unwrap();
    assert!(git.config_schema.iter().any(|c| c.key == "git.tagPrefix"));
}

#[test]
fn test_engine_multi_topic_inspect() {
    let data = load_fixture("multi-topic.ckm.json");
    let engine = CkmEngine::new(data);
    let result = engine.inspect();
    assert_eq!(result.counts.concepts, 4);
    assert_eq!(result.counts.operations, 3);
    assert_eq!(result.counts.constraints, 3);
    assert!(
        result.counts.topics >= 3,
        "Expected at least 3 topics, got {}",
        result.counts.topics
    );
}

// ─── Engine: Edge Cases Fixture ─────────────────────────────────────────

#[test]
fn test_engine_edge_cases_empty() {
    let data = load_fixture("edge-cases.ckm.json");
    let engine = CkmEngine::new(data);
    assert!(engine.topics().is_empty());
    assert_eq!(engine.inspect().counts.topics, 0);
}

#[test]
fn test_engine_edge_cases_topic_index() {
    let data = load_fixture("edge-cases.ckm.json");
    let engine = CkmEngine::new(data);
    let index = engine.topic_index("edge-tool");
    assert!(index.contains("edge-tool CKM"));
    assert!(index.contains("Topics:"));
}

// ─── Engine: Polyglot Fixture ───────────────────────────────────────────

#[test]
fn test_engine_polyglot_topic_count() {
    let data = load_fixture("polyglot.ckm.json");
    let engine = CkmEngine::new(data);
    assert_eq!(engine.topics().len(), 1);
    assert_eq!(engine.topics()[0].name, "format");
}

#[test]
fn test_engine_polyglot_all_canonical_types() {
    let data = load_fixture("polyglot.ckm.json");
    let engine = CkmEngine::new(data);
    let format_topic = &engine.topics()[0];
    let concept = &format_topic.concepts[0];
    let props = concept.properties.as_ref().unwrap();

    // Verify all 8 canonical types are present
    let canonical_types: Vec<String> = props
        .iter()
        .map(|p| p.r#type.canonical.to_string())
        .collect();
    assert!(canonical_types.contains(&"string".to_string()));
    assert!(canonical_types.contains(&"boolean".to_string()));
    assert!(canonical_types.contains(&"number".to_string()));
    assert!(canonical_types.contains(&"integer".to_string()));
    assert!(canonical_types.contains(&"array".to_string()));
    assert!(canonical_types.contains(&"object".to_string()));
    assert!(canonical_types.contains(&"null".to_string()));
    assert!(canonical_types.contains(&"any".to_string()));
}

// ─── Migration: v1 Legacy Fixture ───────────────────────────────────────

#[test]
fn test_migration_v1_legacy() {
    let data = load_fixture("v1-legacy.ckm.json");
    let migrated = migrate_v1_to_v2(&data);

    assert_eq!(migrated.version, "2.0.0");
    assert_eq!(migrated.schema, "https://ckm.dev/schemas/v2.json");
    assert_eq!(migrated.meta.project, "legacy-tool");
    assert_eq!(migrated.meta.language, "typescript");
    assert_eq!(migrated.meta.generator, "unknown");
}

#[test]
fn test_migration_v1_concepts() {
    let data = load_fixture("v1-legacy.ckm.json");
    let migrated = migrate_v1_to_v2(&data);

    assert_eq!(migrated.concepts.len(), 3);

    let calver = migrated
        .concepts
        .iter()
        .find(|c| c.name == "CalVerConfig")
        .unwrap();
    assert_eq!(calver.slug, "calver");
    assert!(calver.tags.contains(&"config".to_string()));

    let validation = migrated
        .concepts
        .iter()
        .find(|c| c.name == "ValidationResult")
        .unwrap();
    assert_eq!(validation.slug, "validation");
    assert!(validation.tags.contains(&"result".to_string()));
}

#[test]
fn test_migration_v1_operations() {
    let data = load_fixture("v1-legacy.ckm.json");
    let migrated = migrate_v1_to_v2(&data);

    assert_eq!(migrated.operations.len(), 2);
    // validate should get calver and validation tags (from concept slug matching)
    let validate = migrated
        .operations
        .iter()
        .find(|op| op.name == "validate")
        .unwrap();
    assert!(!validate.tags.is_empty());
}

#[test]
fn test_migration_v1_constraints() {
    let data = load_fixture("v1-legacy.ckm.json");
    let migrated = migrate_v1_to_v2(&data);

    assert_eq!(migrated.constraints.len(), 2);
    for c in &migrated.constraints {
        assert_eq!(
            c.severity,
            ckm::types::Severity::Error,
            "All migrated constraints should have severity 'error'"
        );
    }
}

#[test]
fn test_migration_v1_workflows() {
    let data = load_fixture("v1-legacy.ckm.json");
    let migrated = migrate_v1_to_v2(&data);

    assert_eq!(migrated.workflows.len(), 1);
    let wf = &migrated.workflows[0];
    assert_eq!(wf.steps.len(), 3);
    assert_eq!(wf.steps[0].action, ckm::types::StepAction::Command);
    assert_eq!(wf.steps[0].value, "npm run validate");
    assert_eq!(wf.steps[1].action, ckm::types::StepAction::Manual);
    assert_eq!(wf.steps[1].value, "Review the changelog entries.");
    assert_eq!(wf.steps[2].action, ckm::types::StepAction::Command);
    assert_eq!(wf.steps[2].value, "npm run release");
}

#[test]
fn test_migration_v1_config_key_rewrite() {
    let data = load_fixture("v1-legacy.ckm.json");
    let migrated = migrate_v1_to_v2(&data);

    // v1: "CalVerConfig.format" -> v2: "calver.format"
    assert!(
        migrated
            .config_schema
            .iter()
            .any(|e| e.key == "calver.format")
    );
    // v1: "CalVerConfig.preventFutureDates" -> v2: "calver.preventFutureDates"
    assert!(
        migrated
            .config_schema
            .iter()
            .any(|e| e.key == "calver.preventFutureDates")
    );
    // v1: "SemVerConfig.allowVPrefix" -> v2: "semver.allowVPrefix"
    assert!(
        migrated
            .config_schema
            .iter()
            .any(|e| e.key == "semver.allowVPrefix")
    );
}

#[test]
fn test_migration_v1_engine_integration() {
    let data = load_fixture("v1-legacy.ckm.json");
    let engine = CkmEngine::new(data);

    // Engine should auto-detect v1 and migrate
    assert_eq!(engine.manifest().version, "2.0.0");
    // CalVerConfig and SemVerConfig are tagged "config", unclaimed ops may add more
    assert!(
        engine.topics().len() >= 2,
        "Expected at least 2 topics, got {}",
        engine.topics().len()
    );

    let names: Vec<&str> = engine.topics().iter().map(|t| t.name.as_str()).collect();
    assert!(names.contains(&"calver"));
    assert!(names.contains(&"semver"));
}

// ─── Validation ─────────────────────────────────────────────────────────

#[test]
fn test_validate_v2_minimal() {
    let data = load_fixture("minimal.ckm.json");
    let result = validate_manifest(&data);
    assert!(
        result.valid,
        "Minimal v2 fixture should be valid: {:?}",
        result.errors
    );
}

#[test]
fn test_validate_v2_multi_topic() {
    let data = load_fixture("multi-topic.ckm.json");
    let result = validate_manifest(&data);
    assert!(
        result.valid,
        "Multi-topic v2 fixture should be valid: {:?}",
        result.errors
    );
}

#[test]
fn test_validate_v2_polyglot() {
    let data = load_fixture("polyglot.ckm.json");
    let result = validate_manifest(&data);
    assert!(
        result.valid,
        "Polyglot v2 fixture should be valid: {:?}",
        result.errors
    );
}

#[test]
fn test_validate_v2_edge_cases() {
    let data = load_fixture("edge-cases.ckm.json");
    let result = validate_manifest(&data);
    assert!(
        result.valid,
        "Edge cases v2 fixture should be valid: {:?}",
        result.errors
    );
}

#[test]
fn test_validate_v1_fails() {
    let data = load_fixture("v1-legacy.ckm.json");
    let result = validate_manifest(&data);
    assert!(!result.valid, "v1 manifest should fail v2 validation");
    assert!(!result.errors.is_empty());
}

#[test]
fn test_validate_invalid_not_object() {
    let data = serde_json::json!(42);
    let result = validate_manifest(&data);
    assert!(!result.valid);
}

#[test]
fn test_validate_invalid_missing_fields() {
    let data = serde_json::json!({});
    let result = validate_manifest(&data);
    assert!(!result.valid);
    // Should have errors for version, meta, concepts, operations, constraints, workflows, configSchema
    assert!(result.errors.len() >= 7);
}

// ─── Token Budget Enforcement ───────────────────────────────────────────

#[test]
fn test_token_budget_level0_all_fixtures() {
    for fixture_name in &[
        "minimal.ckm.json",
        "multi-topic.ckm.json",
        "polyglot.ckm.json",
        "edge-cases.ckm.json",
    ] {
        let data = load_fixture(fixture_name);
        let engine = CkmEngine::new(data);
        let output = engine.topic_index("tool");
        assert!(
            output.len() <= 1200,
            "Level 0 output for {} exceeds 1200 char budget: {} chars",
            fixture_name,
            output.len()
        );
    }
}

#[test]
fn test_token_budget_level1_all_fixtures() {
    for fixture_name in &[
        "minimal.ckm.json",
        "multi-topic.ckm.json",
        "polyglot.ckm.json",
    ] {
        let data = load_fixture(fixture_name);
        let engine = CkmEngine::new(data);
        for topic in engine.topics() {
            if let Some(content) = engine.topic_content(&topic.name) {
                assert!(
                    content.len() <= 3200,
                    "Level 1 output for {} topic '{}' exceeds 3200 char budget: {} chars",
                    fixture_name,
                    topic.name,
                    content.len()
                );
            }
        }
    }
}

// ─── Serialization Round-Trip ───────────────────────────────────────────

#[test]
fn test_manifest_serialization_roundtrip() {
    let data = load_fixture("minimal.ckm.json");
    let engine = CkmEngine::new(data.clone());
    let manifest = engine.manifest();

    // Serialize to JSON and back
    let json = serde_json::to_value(manifest).expect("Failed to serialize manifest");
    let roundtripped: ckm::types::CkmManifest =
        serde_json::from_value(json).expect("Failed to deserialize manifest");

    assert_eq!(manifest.meta.project, roundtripped.meta.project);
    assert_eq!(manifest.concepts.len(), roundtripped.concepts.len());
    assert_eq!(manifest.operations.len(), roundtripped.operations.len());
}

#[test]
fn test_topic_json_serialization() {
    let data = load_fixture("minimal.ckm.json");
    let engine = CkmEngine::new(data);

    // Index JSON should be valid JSON
    match engine.topic_json(None) {
        TopicJsonResult::Index(idx) => {
            let json_str = serde_json::to_string(&idx).expect("Index should serialize");
            assert!(!json_str.is_empty());
        }
        _ => panic!("Expected Index"),
    }

    // Topic JSON should be valid JSON
    match engine.topic_json(Some("calver")) {
        TopicJsonResult::Topic(t) => {
            let json_str = serde_json::to_string(&t).expect("Topic should serialize");
            assert!(!json_str.is_empty());
        }
        _ => panic!("Expected Topic"),
    }

    // Error JSON should be valid JSON
    match engine.topic_json(Some("unknown")) {
        TopicJsonResult::Error(e) => {
            let json_str = serde_json::to_string(&e).expect("Error should serialize");
            assert!(!json_str.is_empty());
        }
        _ => panic!("Expected Error"),
    }
}
