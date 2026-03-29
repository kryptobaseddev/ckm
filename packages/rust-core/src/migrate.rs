//! CKM v1 to v2 migration and version detection.
//!
//! Implements SPEC.md sections 1 and 2: version detection and deterministic
//! migration from v1 format to v2 format.

use serde_json::Value;

use crate::types::{
    CanonicalType, CkmConcept, CkmConfigEntry, CkmConstraint, CkmInput, CkmManifest, CkmMeta,
    CkmOperation, CkmOutput, CkmProperty, CkmTypeRef, CkmWorkflow, CkmWorkflowStep, Severity,
    StepAction,
};

// ─── Version Detection (SPEC.md Section 1) ──────────────────────────────

/// Detects the schema version of a parsed manifest.
///
/// Returns 2 if the data has a `meta` object or a `$schema` URL containing "v2".
/// Returns 1 otherwise (including malformed data).
pub fn detect_version(data: &Value) -> u8 {
    if let Value::Object(obj) = data {
        // Primary indicator: presence of a "meta" object
        if let Some(Value::Object(_)) = obj.get("meta") {
            return 2;
        }
        // Secondary indicator: $schema URL containing "v2"
        if let Some(Value::String(schema)) = obj.get("$schema")
            && schema.contains("v2")
        {
            return 2;
        }
    }
    1
}

// ─── Migration (SPEC.md Section 2) ──────────────────────────────────────

/// Deterministic migration from v1 format to v2 format.
///
/// Accepts a parsed v1 manifest (as `serde_json::Value`) and returns a valid
/// v2 [`CkmManifest`]. The algorithm follows SPEC.md exactly.
pub fn migrate_v1_to_v2(v1: &Value) -> CkmManifest {
    let empty_map = serde_json::Map::new();
    let obj = v1.as_object().unwrap_or(&empty_map);

    // Step 2: Create meta block
    let meta = CkmMeta {
        project: obj
            .get("project")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string(),
        language: "typescript".to_string(),
        generator: "unknown".to_string(),
        generated: obj
            .get("generated")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        source_url: None,
    };

    // Step 3: Migrate concepts
    let concepts: Vec<CkmConcept> = obj
        .get("concepts")
        .and_then(Value::as_array)
        .map(|arr| arr.iter().map(migrate_concept).collect())
        .unwrap_or_default();

    // Step 4: Migrate operations
    let operations: Vec<CkmOperation> = obj
        .get("operations")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .map(|op| migrate_operation(op, &concepts))
                .collect()
        })
        .unwrap_or_default();

    // Step 5: Migrate constraints
    let constraints: Vec<CkmConstraint> = obj
        .get("constraints")
        .and_then(Value::as_array)
        .map(|arr| arr.iter().map(migrate_constraint).collect())
        .unwrap_or_default();

    // Step 6: Migrate workflows
    let workflows: Vec<CkmWorkflow> = obj
        .get("workflows")
        .and_then(Value::as_array)
        .map(|arr| arr.iter().map(migrate_workflow).collect())
        .unwrap_or_default();

    // Step 7: Migrate config schema
    let config_schema: Vec<CkmConfigEntry> = obj
        .get("configSchema")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .map(|e| migrate_config_entry(e, &concepts))
                .collect()
        })
        .unwrap_or_default();

    CkmManifest {
        schema: "https://ckm.dev/schemas/v2.json".to_string(),
        version: "2.0.0".to_string(),
        meta,
        concepts,
        operations,
        constraints,
        workflows,
        config_schema,
    }
}

// ─── Helpers ────────────────────────────────────────────────────────────

/// Derives a slug from a concept name by stripping known suffixes.
pub(crate) fn derive_slug(name: &str) -> String {
    let slug = if let Some(stripped) = name.strip_suffix("Config") {
        stripped
    } else if let Some(stripped) = name.strip_suffix("Result") {
        stripped
    } else if let Some(stripped) = name.strip_suffix("Options") {
        stripped
    } else {
        name
    };
    slug.to_lowercase()
}

/// Infers tags from a concept name based on known suffixes.
fn infer_tags(name: &str) -> Vec<String> {
    let mut tags = Vec::new();
    if name.ends_with("Config") {
        tags.push("config".to_string());
    }
    if name.ends_with("Result") {
        tags.push("result".to_string());
    }
    if name.ends_with("Options") {
        tags.push("options".to_string());
    }
    tags
}

/// Infers canonical type from a type string.
fn infer_canonical(type_str: &str) -> CanonicalType {
    let lower = type_str.to_lowercase();
    match lower.as_str() {
        "string" => CanonicalType::String,
        "boolean" => CanonicalType::Boolean,
        "number" => CanonicalType::Number,
        "integer" => CanonicalType::Integer,
        "null" | "undefined" | "void" => CanonicalType::Null,
        "object" | "record" => CanonicalType::Object,
        "unknown" | "any" => CanonicalType::Any,
        _ => {
            if lower.contains("[]") || lower.starts_with("array") {
                CanonicalType::Array
            } else if lower.contains('|') {
                CanonicalType::String
            } else {
                CanonicalType::Object
            }
        }
    }
}

/// Migrates a v1 type string to a v2 [`CkmTypeRef`].
fn migrate_type_string(type_str: Option<&str>) -> CkmTypeRef {
    match type_str {
        None | Some("") => CkmTypeRef {
            canonical: CanonicalType::Any,
            original: None,
            r#enum: None,
        },
        Some(s) => CkmTypeRef {
            canonical: infer_canonical(s),
            original: Some(s.to_string()),
            r#enum: None,
        },
    }
}

/// Infers operation tags by matching operation name/what against concept slugs.
fn infer_operation_tags(op: &Value, concepts: &[CkmConcept]) -> Vec<String> {
    let name = op.get("name").and_then(Value::as_str).unwrap_or("");
    let what = op.get("what").and_then(Value::as_str).unwrap_or("");
    let haystack = format!("{} {}", name, what).to_lowercase();

    let mut tags: Vec<String> = Vec::new();
    for concept in concepts {
        if !concept.slug.is_empty()
            && haystack.contains(&concept.slug)
            && !tags.contains(&concept.slug)
        {
            tags.push(concept.slug.clone());
        }
    }
    tags
}

fn migrate_concept(v: &Value) -> CkmConcept {
    let name = v
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let slug = derive_slug(&name);
    let tags = infer_tags(&name);

    let properties = v.get("properties").and_then(Value::as_array).map(|arr| {
        arr.iter()
            .map(|p| {
                let type_str = p.get("type").and_then(Value::as_str);
                CkmProperty {
                    name: p
                        .get("name")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string(),
                    r#type: migrate_type_string(type_str),
                    description: p
                        .get("description")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string(),
                    required: true,
                    default: None,
                }
            })
            .collect()
    });

    CkmConcept {
        id: v
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        name,
        slug,
        what: v
            .get("what")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        tags,
        properties,
    }
}

fn migrate_operation(v: &Value, concepts: &[CkmConcept]) -> CkmOperation {
    let tags = infer_operation_tags(v, concepts);

    let inputs = v.get("inputs").and_then(Value::as_array).map(|arr| {
        arr.iter()
            .map(|i| {
                let type_str = i.get("type").and_then(Value::as_str);
                CkmInput {
                    name: i
                        .get("name")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string(),
                    r#type: migrate_type_string(type_str),
                    required: i.get("required").and_then(Value::as_bool).unwrap_or(true),
                    description: i
                        .get("description")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string(),
                }
            })
            .collect()
    });

    let outputs = v.get("outputs").and_then(|o| {
        if o.is_null() {
            return None;
        }
        let text = o.get("text").and_then(Value::as_str).unwrap_or("unknown");
        Some(CkmOutput {
            r#type: migrate_type_string(Some(text)),
            description: text.to_string(),
        })
    });

    CkmOperation {
        id: v
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        name: v
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        what: v
            .get("what")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        tags,
        inputs,
        outputs,
    }
}

fn migrate_constraint(v: &Value) -> CkmConstraint {
    CkmConstraint {
        id: v
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        rule: v
            .get("rule")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        enforced_by: v
            .get("enforcedBy")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        severity: Severity::Error,
    }
}

fn migrate_workflow(v: &Value) -> CkmWorkflow {
    let steps = v
        .get("steps")
        .and_then(Value::as_array)
        .map(|arr| arr.iter().map(migrate_workflow_step).collect())
        .unwrap_or_default();

    CkmWorkflow {
        id: v
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        goal: v
            .get("goal")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        tags: Vec::new(),
        steps,
    }
}

fn migrate_workflow_step(v: &Value) -> CkmWorkflowStep {
    let note = v.get("note").and_then(Value::as_str).map(|s| s.to_string());

    if let Some(cmd) = v.get("command").and_then(Value::as_str) {
        return CkmWorkflowStep {
            action: StepAction::Command,
            value: cmd.to_string(),
            note,
        };
    }

    if let Some(manual) = v.get("manual").and_then(Value::as_str) {
        return CkmWorkflowStep {
            action: StepAction::Manual,
            value: manual.to_string(),
            note,
        };
    }

    CkmWorkflowStep {
        action: StepAction::Manual,
        value: String::new(),
        note,
    }
}

fn migrate_config_entry(v: &Value, concepts: &[CkmConcept]) -> CkmConfigEntry {
    let key = v
        .get("key")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let type_str = v.get("type").and_then(Value::as_str);
    let default = v
        .get("default")
        .and_then(Value::as_str)
        .map(|s| s.to_string());

    CkmConfigEntry {
        key: migrate_config_key(&key, concepts),
        r#type: migrate_type_string(type_str),
        description: v
            .get("description")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        default,
        required: true,
    }
}

/// Migrates a v1 config key from "ConceptName.prop" to "slug.prop" format.
fn migrate_config_key(key: &str, concepts: &[CkmConcept]) -> String {
    let parts: Vec<&str> = key.splitn(2, '.').collect();
    if parts.len() >= 2 {
        let concept_part = parts[0];
        let rest = parts[1];
        // Find matching concept to get slug
        for concept in concepts {
            if concept.name == concept_part {
                return format!("{}.{}", concept.slug, rest);
            }
        }
        // If no concept match, lowercase the first segment
        return format!("{}.{}", concept_part.to_lowercase(), rest);
    }
    key.to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_version_v2_with_meta() {
        let data: Value = serde_json::json!({
            "meta": { "project": "test" },
            "version": "2.0.0"
        });
        assert_eq!(detect_version(&data), 2);
    }

    #[test]
    fn test_detect_version_v2_with_schema() {
        let data: Value = serde_json::json!({
            "$schema": "https://ckm.dev/schemas/v2.json"
        });
        assert_eq!(detect_version(&data), 2);
    }

    #[test]
    fn test_detect_version_v1() {
        let data: Value = serde_json::json!({
            "project": "test",
            "generated": "2025-01-01"
        });
        assert_eq!(detect_version(&data), 1);
    }

    #[test]
    fn test_detect_version_malformed() {
        let data: Value = serde_json::json!(42);
        assert_eq!(detect_version(&data), 1);
    }

    #[test]
    fn test_derive_slug() {
        assert_eq!(derive_slug("CalVerConfig"), "calver");
        assert_eq!(derive_slug("SemVerConfig"), "semver");
        assert_eq!(derive_slug("ValidationResult"), "validation");
        assert_eq!(derive_slug("GitHooksConfig"), "githooks");
        assert_eq!(derive_slug("VersionGuardConfig"), "versionguard");
        assert_eq!(derive_slug("PlainName"), "plainname");
    }

    #[test]
    fn test_infer_tags() {
        assert_eq!(infer_tags("CalVerConfig"), vec!["config"]);
        assert_eq!(infer_tags("ValidationResult"), vec!["result"]);
        assert_eq!(infer_tags("RenderOptions"), vec!["options"]);
        assert!(infer_tags("PlainName").is_empty());
    }

    #[test]
    fn test_infer_canonical() {
        assert_eq!(infer_canonical("string"), CanonicalType::String);
        assert_eq!(infer_canonical("boolean"), CanonicalType::Boolean);
        assert_eq!(infer_canonical("number"), CanonicalType::Number);
        assert_eq!(infer_canonical("integer"), CanonicalType::Integer);
        assert_eq!(infer_canonical("null"), CanonicalType::Null);
        assert_eq!(infer_canonical("undefined"), CanonicalType::Null);
        assert_eq!(infer_canonical("void"), CanonicalType::Null);
        assert_eq!(infer_canonical("string[]"), CanonicalType::Array);
        assert_eq!(infer_canonical("Array<number>"), CanonicalType::Array);
        assert_eq!(infer_canonical("object"), CanonicalType::Object);
        assert_eq!(infer_canonical("Record"), CanonicalType::Object);
        assert_eq!(infer_canonical("unknown"), CanonicalType::Any);
        assert_eq!(infer_canonical("any"), CanonicalType::Any);
        assert_eq!(infer_canonical("string | number"), CanonicalType::String);
        assert_eq!(infer_canonical("CalVerFormat"), CanonicalType::Object);
    }
}
