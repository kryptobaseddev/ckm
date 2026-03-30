//! Utility to regenerate conformance expected outputs from actual engine behavior.
//! Run with: FERROUS_FORGE_ENABLED=0 cargo test --test regenerate_expected -- --ignored --nocapture

use std::fs;
use std::path::PathBuf;

use ckm::{CkmEngine, detect_version, validate_manifest};

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../conformance/fixtures")
}

fn expected_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../conformance/expected")
}

fn load_fixture(name: &str) -> serde_json::Value {
    let path = fixtures_dir().join(name);
    let content =
        fs::read_to_string(&path).unwrap_or_else(|_| panic!("Failed to read {}", path.display()));
    serde_json::from_str(&content).unwrap_or_else(|_| panic!("Failed to parse {}", path.display()))
}

#[test]
#[ignore] // Only run manually to regenerate
fn regenerate_all_expected_outputs() {
    let fixtures = [
        "minimal",
        "multi-topic",
        "polyglot",
        "v1-legacy",
        "edge-cases",
    ];

    for name in &fixtures {
        let fixture_file = format!("{}.ckm.json", name);
        let data = load_fixture(&fixture_file);
        let out_dir = expected_dir().join(name);
        fs::create_dir_all(&out_dir).unwrap();

        // detectVersion
        let version = detect_version(&data);
        let detect_json = serde_json::json!({ "version": version });
        fs::write(
            out_dir.join("detectVersion.json"),
            serde_json::to_string_pretty(&detect_json).unwrap(),
        )
        .unwrap();

        // validateManifest
        let validation = validate_manifest(&data);
        fs::write(
            out_dir.join("validate.json"),
            serde_json::to_string_pretty(&validation).unwrap(),
        )
        .unwrap();

        // Engine
        let engine = CkmEngine::new(data);

        // topics
        let topics_json: Vec<serde_json::Value> = engine
            .topics()
            .iter()
            .map(|t| {
                serde_json::json!({
                    "name": t.name,
                    "summary": t.summary,
                    "conceptCount": t.concepts.len(),
                    "operationCount": t.operations.len(),
                    "configCount": t.config_schema.len(),
                    "constraintCount": t.constraints.len(),
                })
            })
            .collect();
        fs::write(
            out_dir.join("topics.json"),
            serde_json::to_string_pretty(&topics_json).unwrap(),
        )
        .unwrap();

        // topicIndex (getTopicJson with no arg)
        let topic_index = engine.topic_json(None);
        fs::write(
            out_dir.join("topicIndex.json"),
            serde_json::to_string_pretty(&topic_index).unwrap(),
        )
        .unwrap();

        // inspect
        let inspect = engine.inspect();
        fs::write(
            out_dir.join("inspect.json"),
            serde_json::to_string_pretty(&inspect).unwrap(),
        )
        .unwrap();

        // Per-topic content and JSON
        for topic in engine.topics() {
            // topicContent
            if let Some(content) = engine.topic_content(&topic.name) {
                let filename = format!("topicContent-{}.txt", topic.name);
                fs::write(out_dir.join(&filename), content).unwrap();
            }

            // topicJson
            let topic_json = engine.topic_json(Some(&topic.name));
            let filename = format!("topicJson-{}.json", topic.name);
            fs::write(
                out_dir.join(&filename),
                serde_json::to_string_pretty(&topic_json).unwrap(),
            )
            .unwrap();
        }

        println!("Regenerated: {}", name);
    }

    println!("All expected outputs regenerated.");
}
