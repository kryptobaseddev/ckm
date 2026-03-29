//! CKM manifest validation against the v2 schema.
//!
//! Provides lightweight structural validation without external JSON Schema
//! library dependencies. Validates required fields, types, and enum values.

use serde_json::Value;

use crate::types::{CkmValidationError, CkmValidationResult};

const CANONICAL_TYPES: &[&str] = &[
    "string", "boolean", "number", "integer", "array", "object", "null", "any",
];

/// Validates a parsed JSON object against the ckm.json v2 schema.
///
/// Returns `{ valid: true, errors: [] }` for valid manifests.
/// Returns `{ valid: false, errors: [...] }` with JSON pointer paths for invalid manifests.
/// v1 manifests will fail validation (they lack required v2 fields).
pub fn validate_manifest(data: &Value) -> CkmValidationResult {
    let mut errors: Vec<CkmValidationError> = Vec::new();

    let obj = match data.as_object() {
        Some(obj) => obj,
        None => {
            errors.push(CkmValidationError {
                path: String::new(),
                message: "Manifest must be an object".to_string(),
            });
            return CkmValidationResult {
                valid: false,
                errors,
            };
        }
    };

    // Required top-level fields
    validate_required_string(obj, "version", "", &mut errors);
    validate_required_object(obj, "meta", "", &mut errors);
    validate_required_array(obj, "concepts", "", &mut errors);
    validate_required_array(obj, "operations", "", &mut errors);
    validate_required_array(obj, "constraints", "", &mut errors);
    validate_required_array(obj, "workflows", "", &mut errors);
    validate_required_array(obj, "configSchema", "", &mut errors);

    // Meta validation
    if let Some(Value::Object(meta)) = obj.get("meta") {
        validate_required_string(meta, "project", "/meta", &mut errors);
        validate_required_string(meta, "language", "/meta", &mut errors);
        validate_required_string(meta, "generator", "/meta", &mut errors);
        validate_required_string(meta, "generated", "/meta", &mut errors);
    }

    // Concepts validation
    if let Some(Value::Array(concepts)) = obj.get("concepts") {
        for (i, concept) in concepts.iter().enumerate() {
            let path = format!("/concepts/{}", i);
            if let Some(c) = concept.as_object() {
                validate_required_string(c, "id", &path, &mut errors);
                validate_required_string(c, "name", &path, &mut errors);
                validate_required_string(c, "slug", &path, &mut errors);
                validate_required_string(c, "what", &path, &mut errors);
                validate_required_array(c, "tags", &path, &mut errors);

                if let Some(Value::Array(props)) = c.get("properties") {
                    for (j, prop) in props.iter().enumerate() {
                        validate_property(prop, &format!("{}/properties/{}", path, j), &mut errors);
                    }
                }
            } else {
                errors.push(CkmValidationError {
                    path,
                    message: "Concept must be an object".to_string(),
                });
            }
        }
    }

    // Operations validation
    if let Some(Value::Array(operations)) = obj.get("operations") {
        for (i, operation) in operations.iter().enumerate() {
            let path = format!("/operations/{}", i);
            if let Some(op) = operation.as_object() {
                validate_required_string(op, "id", &path, &mut errors);
                validate_required_string(op, "name", &path, &mut errors);
                validate_required_string(op, "what", &path, &mut errors);
                validate_required_array(op, "tags", &path, &mut errors);
            } else {
                errors.push(CkmValidationError {
                    path,
                    message: "Operation must be an object".to_string(),
                });
            }
        }
    }

    // Constraints validation
    if let Some(Value::Array(constraints)) = obj.get("constraints") {
        for (i, constraint) in constraints.iter().enumerate() {
            let path = format!("/constraints/{}", i);
            if let Some(c) = constraint.as_object() {
                validate_required_string(c, "id", &path, &mut errors);
                validate_required_string(c, "rule", &path, &mut errors);
                validate_required_string(c, "enforcedBy", &path, &mut errors);
                if let Some(Value::String(severity)) = c.get("severity") {
                    if !["error", "warning", "info"].contains(&severity.as_str()) {
                        errors.push(CkmValidationError {
                            path: format!("{}/severity", path),
                            message: format!(
                                "Invalid severity: \"{}\". Must be \"error\", \"warning\", or \"info\"",
                                severity
                            ),
                        });
                    }
                } else {
                    errors.push(CkmValidationError {
                        path: format!("{}/severity", path),
                        message: "Missing required field: severity".to_string(),
                    });
                }
            } else {
                errors.push(CkmValidationError {
                    path,
                    message: "Constraint must be an object".to_string(),
                });
            }
        }
    }

    // Workflows validation
    if let Some(Value::Array(workflows)) = obj.get("workflows") {
        for (i, workflow) in workflows.iter().enumerate() {
            let path = format!("/workflows/{}", i);
            if let Some(wf) = workflow.as_object() {
                validate_required_string(wf, "id", &path, &mut errors);
                validate_required_string(wf, "goal", &path, &mut errors);
                validate_required_array(wf, "tags", &path, &mut errors);
                match wf.get("steps") {
                    Some(Value::Array(steps)) if !steps.is_empty() => {}
                    _ => {
                        errors.push(CkmValidationError {
                            path: format!("{}/steps", path),
                            message: "Workflow must have at least one step".to_string(),
                        });
                    }
                }
            } else {
                errors.push(CkmValidationError {
                    path,
                    message: "Workflow must be an object".to_string(),
                });
            }
        }
    }

    // ConfigSchema validation
    if let Some(Value::Array(config_schema)) = obj.get("configSchema") {
        for (i, entry) in config_schema.iter().enumerate() {
            let path = format!("/configSchema/{}", i);
            if let Some(e) = entry.as_object() {
                validate_required_string(e, "key", &path, &mut errors);
                validate_required_string(e, "description", &path, &mut errors);
                if e.get("required").and_then(Value::as_bool).is_none() {
                    errors.push(CkmValidationError {
                        path: format!("{}/required", path),
                        message: "Missing required field: required".to_string(),
                    });
                }
                validate_type_ref(e.get("type"), &format!("{}/type", path), &mut errors);
            } else {
                errors.push(CkmValidationError {
                    path,
                    message: "Config entry must be an object".to_string(),
                });
            }
        }
    }

    CkmValidationResult {
        valid: errors.is_empty(),
        errors,
    }
}

// ─── Helpers ────────────────────────────────────────────────────────────

fn validate_required_string(
    obj: &serde_json::Map<String, Value>,
    field: &str,
    parent_path: &str,
    errors: &mut Vec<CkmValidationError>,
) {
    if obj.get(field).and_then(Value::as_str).is_none() {
        errors.push(CkmValidationError {
            path: format!("{}/{}", parent_path, field),
            message: format!("Missing required field: {}", field),
        });
    }
}

fn validate_required_object(
    obj: &serde_json::Map<String, Value>,
    field: &str,
    parent_path: &str,
    errors: &mut Vec<CkmValidationError>,
) {
    if obj.get(field).and_then(Value::as_object).is_none() {
        errors.push(CkmValidationError {
            path: format!("{}/{}", parent_path, field),
            message: format!("Missing required field: {}", field),
        });
    }
}

fn validate_required_array(
    obj: &serde_json::Map<String, Value>,
    field: &str,
    parent_path: &str,
    errors: &mut Vec<CkmValidationError>,
) {
    if obj.get(field).and_then(Value::as_array).is_none() {
        errors.push(CkmValidationError {
            path: format!("{}/{}", parent_path, field),
            message: format!("Missing required field: {}", field),
        });
    }
}

fn validate_type_ref(type_ref: Option<&Value>, path: &str, errors: &mut Vec<CkmValidationError>) {
    match type_ref.and_then(Value::as_object) {
        Some(ref_obj) => {
            if let Some(Value::String(canonical)) = ref_obj.get("canonical") {
                if !CANONICAL_TYPES.contains(&canonical.as_str()) {
                    errors.push(CkmValidationError {
                        path: format!("{}/canonical", path),
                        message: format!(
                            "Invalid canonical type. Must be one of: {}",
                            CANONICAL_TYPES.join(", ")
                        ),
                    });
                }
            } else {
                errors.push(CkmValidationError {
                    path: format!("{}/canonical", path),
                    message: format!(
                        "Invalid canonical type. Must be one of: {}",
                        CANONICAL_TYPES.join(", ")
                    ),
                });
            }
        }
        None => {
            errors.push(CkmValidationError {
                path: path.to_string(),
                message: "Type reference must be an object".to_string(),
            });
        }
    }
}

fn validate_property(prop: &Value, path: &str, errors: &mut Vec<CkmValidationError>) {
    match prop.as_object() {
        Some(p) => {
            validate_required_string(p, "name", path, errors);
            validate_required_string(p, "description", path, errors);
            if p.get("required").and_then(Value::as_bool).is_none() {
                errors.push(CkmValidationError {
                    path: format!("{}/required", path),
                    message: "Missing required field: required".to_string(),
                });
            }
            validate_type_ref(p.get("type"), &format!("{}/type", path), errors);
        }
        None => {
            errors.push(CkmValidationError {
                path: path.to_string(),
                message: "Property must be an object".to_string(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_manifest() {
        let data: Value = serde_json::json!({
            "$schema": "https://ckm.dev/schemas/v2.json",
            "version": "2.0.0",
            "meta": {
                "project": "test",
                "language": "typescript",
                "generator": "test",
                "generated": "2026-01-01T00:00:00.000Z"
            },
            "concepts": [],
            "operations": [],
            "constraints": [],
            "workflows": [],
            "configSchema": []
        });
        let result = validate_manifest(&data);
        assert!(result.valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_invalid_manifest_missing_meta() {
        let data: Value = serde_json::json!({
            "version": "2.0.0",
            "concepts": [],
            "operations": [],
            "constraints": [],
            "workflows": [],
            "configSchema": []
        });
        let result = validate_manifest(&data);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.path == "/meta"));
    }

    #[test]
    fn test_invalid_manifest_not_object() {
        let data: Value = serde_json::json!(42);
        let result = validate_manifest(&data);
        assert!(!result.valid);
        assert_eq!(result.errors[0].message, "Manifest must be an object");
    }
}
