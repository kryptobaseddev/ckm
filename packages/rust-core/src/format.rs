//! CKM terminal formatter — plain text output with no color dependencies.
//!
//! Implements SPEC.md section 4: output formatting for progressive disclosure.

use crate::types::{CkmConcept, CkmConfigEntry, CkmProperty, CkmTopic, CkmTypeRef};

/// Formats the topic index for terminal display (Level 0).
///
/// Includes tool name, usage line, topic list with summaries, and flag descriptions.
/// Output stays within 300 token budget.
pub fn format_topic_index(topics: &[CkmTopic], tool_name: &str) -> String {
    let mut lines = vec![
        format!("{} CKM — Codebase Knowledge Manifest", tool_name),
        String::new(),
        format!("Usage: {} ckm [topic] [--json] [--llm]", tool_name),
        String::new(),
        "Topics:".to_string(),
    ];

    let max_name = topics.iter().map(|t| t.name.len()).max().unwrap_or(0);
    for topic in topics {
        let padded = format!("{:width$}", topic.name, width = max_name + 2);
        lines.push(format!("  {}{}", padded, topic.summary));
    }

    lines.push(String::new());
    lines.push("Flags:".to_string());
    lines.push(
        "  --json    Machine-readable CKM output (concepts, operations, config schema)".to_string(),
    );
    lines.push("  --llm     Full API context for LLM agents (forge-ts llms.txt)".to_string());

    lines.join("\n")
}

/// Formats a topic's content for human-readable terminal display (Level 1).
///
/// Returns `None` if the topic is not found. Output stays within 800 token budget.
pub fn format_topic_content(topics: &[CkmTopic], topic_name: &str) -> Option<String> {
    let topic = topics.iter().find(|t| t.name == topic_name)?;

    let mut lines: Vec<String> = vec![format!("# {}", topic.summary), String::new()];

    // Concepts
    if !topic.concepts.is_empty() {
        lines.push("## Concepts".to_string());
        lines.push(String::new());
        for c in &topic.concepts {
            lines.push(format!("  {} — {}", c.name, c.what));
            if let Some(ref props) = c.properties {
                for p in props {
                    let def = find_default(&topic.config_schema, c, p);
                    let type_str = format_type_ref(&p.r#type);
                    lines.push(format!("    {}: {}{}", p.name, type_str, def));
                    if !p.description.is_empty() {
                        lines.push(format!("      {}", p.description));
                    }
                }
            }
            lines.push(String::new());
        }
    }

    // Operations
    if !topic.operations.is_empty() {
        lines.push("## Operations".to_string());
        lines.push(String::new());
        for o in &topic.operations {
            lines.push(format!("  {}() — {}", o.name, o.what));
            if let Some(ref inputs) = o.inputs {
                for i in inputs {
                    lines.push(format!("    @param {}: {}", i.name, i.description));
                }
            }
            lines.push(String::new());
        }
    }

    // Config schema
    if !topic.config_schema.is_empty() {
        lines.push("## Config Fields".to_string());
        lines.push(String::new());
        for c in &topic.config_schema {
            let type_str = format_type_ref(&c.r#type);
            let default_str = c
                .default
                .as_ref()
                .map(|d| format!(" = {}", d))
                .unwrap_or_default();
            lines.push(format!("  {}: {}{}", c.key, type_str, default_str));
            if !c.description.is_empty() {
                lines.push(format!("    {}", c.description));
            }
        }
        lines.push(String::new());
    }

    // Constraints
    if !topic.constraints.is_empty() {
        lines.push("## Constraints".to_string());
        lines.push(String::new());
        for c in &topic.constraints {
            lines.push(format!("  [{}] {}", c.id, c.rule));
            lines.push(format!("    Enforced by: {}", c.enforced_by));
        }
        lines.push(String::new());
    }

    Some(lines.join("\n"))
}

/// Formats a [`CkmTypeRef`] for display.
fn format_type_ref(type_ref: &CkmTypeRef) -> String {
    if let Some(ref original) = type_ref.original
        && !original.is_empty()
    {
        return original.clone();
    }
    type_ref.canonical.to_string()
}

/// Finds the default value for a property from config schema entries.
fn find_default(
    config_schema: &[CkmConfigEntry],
    _concept: &CkmConcept,
    property: &CkmProperty,
) -> String {
    for entry in config_schema {
        if entry.key.ends_with(&format!(".{}", property.name))
            && let Some(ref default) = entry.default
        {
            return format!(" = {}", default);
        }
    }
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    fn make_topic() -> CkmTopic {
        CkmTopic {
            name: "calver".to_string(),
            summary: "Configures CalVer validation rules.".to_string(),
            concepts: vec![CkmConcept {
                id: "concept-calver-config".to_string(),
                name: "CalVerConfig".to_string(),
                slug: "calver".to_string(),
                what: "Configures CalVer validation rules.".to_string(),
                tags: vec!["config".to_string()],
                properties: Some(vec![CkmProperty {
                    name: "format".to_string(),
                    r#type: CkmTypeRef {
                        canonical: CanonicalType::String,
                        original: Some("CalVerFormat".to_string()),
                        r#enum: None,
                    },
                    description: "Calendar format used for version strings.".to_string(),
                    required: true,
                    default: Some("YYYY.MM.DD".to_string()),
                }]),
                rules: None,
                related_to: None,
                extensions: None,
            }],
            operations: vec![CkmOperation {
                id: "op-validate".to_string(),
                name: "validate".to_string(),
                what: "Validates a calver version string.".to_string(),
                tags: vec!["calver".to_string()],
                preconditions: None,
                inputs: Some(vec![CkmInput {
                    name: "version".to_string(),
                    r#type: CkmTypeRef {
                        canonical: CanonicalType::String,
                        original: None,
                        r#enum: None,
                    },
                    required: true,
                    description: "The version string to validate.".to_string(),
                }]),
                outputs: None,
                exit_codes: None,
                checks_performed: None,
                extensions: None,
            }],
            config_schema: vec![CkmConfigEntry {
                key: "calver.format".to_string(),
                r#type: CkmTypeRef {
                    canonical: CanonicalType::String,
                    original: Some("CalVerFormat".to_string()),
                    r#enum: None,
                },
                description: "Calendar format used for version strings.".to_string(),
                default: Some("YYYY.MM.DD".to_string()),
                required: true,
                effect: None,
                extensions: None,
            }],
            constraints: vec![CkmConstraint {
                id: "constraint-no-future-dates".to_string(),
                rule: "CalVer versions must not reference future dates.".to_string(),
                enforced_by: "validate".to_string(),
                severity: Severity::Error,
                config_key: None,
                default: None,
                security: None,
                extensions: None,
            }],
        }
    }

    #[test]
    fn test_format_topic_index() {
        let topics = vec![make_topic()];
        let output = format_topic_index(&topics, "my-tool");
        assert!(output.contains("my-tool CKM"));
        assert!(output.contains("calver"));
        assert!(output.contains("Topics:"));
        assert!(output.contains("--json"));
    }

    #[test]
    fn test_format_topic_content() {
        let topics = vec![make_topic()];
        let output = format_topic_content(&topics, "calver").unwrap();
        assert!(output.contains("# Configures CalVer"));
        assert!(output.contains("## Concepts"));
        assert!(output.contains("CalVerConfig"));
        assert!(output.contains("## Operations"));
        assert!(output.contains("validate()"));
        assert!(output.contains("## Config Fields"));
        assert!(output.contains("## Constraints"));
    }

    #[test]
    fn test_format_topic_content_not_found() {
        let topics = vec![make_topic()];
        assert!(format_topic_content(&topics, "nonexistent").is_none());
    }
}
