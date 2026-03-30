//! CKM CLI — browse, validate, migrate, and inspect Codebase Knowledge Manifests.
//!
//! A pure Rust binary built on the `ckm` core crate.

use std::fs;
use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand};
use serde_json::Value;

use ckm::{CkmEngine, detect_version, migrate_v1_to_v2, validate_manifest};

// ─── CLI Definition ────────────────────────────────────────────────────

#[derive(Parser)]
#[command(name = "ckm", version, about = "CKM — Codebase Knowledge Manifest CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Browse CKM topics (default view)
    Browse {
        /// Topic name to drill into (omit for index)
        topic: Option<String>,

        /// Output as JSON instead of human-readable text
        #[arg(long)]
        json: bool,

        /// Path to ckm.json (default: auto-resolve)
        #[arg(long, value_name = "PATH")]
        file: Option<PathBuf>,
    },

    /// Validate a ckm.json against the v2 schema
    Validate {
        /// Path to the ckm.json file
        file: PathBuf,
    },

    /// Migrate a v1 manifest to v2 format
    Migrate {
        /// Path to the v1 ckm.json file
        file: PathBuf,

        /// Print migrated output without writing to disk
        #[arg(long)]
        dry_run: bool,

        /// Output file path (default: overwrite input)
        #[arg(long, value_name = "PATH")]
        output: Option<PathBuf>,
    },

    /// Show manifest statistics
    Inspect {
        /// Path to the ckm.json file
        file: PathBuf,
    },
}

// ─── File Resolution ───────────────────────────────────────────────────

/// Resolves a ckm.json file path using the priority:
/// 1. Explicit --file flag
/// 2. ./ckm.json
/// 3. ./docs/ckm.json
/// 4. ./.ckm/ckm.json
fn resolve_manifest_path(explicit: Option<PathBuf>) -> Result<PathBuf, String> {
    if let Some(path) = explicit {
        if path.exists() {
            return Ok(path);
        }
        return Err(format!("File not found: {}", path.display()));
    }

    let candidates = [
        PathBuf::from("./ckm.json"),
        PathBuf::from("./docs/ckm.json"),
        PathBuf::from("./.ckm/ckm.json"),
    ];

    for candidate in &candidates {
        if candidate.exists() {
            return Ok(candidate.clone());
        }
    }

    Err(
        "No ckm.json found. Searched: ./ckm.json, ./docs/ckm.json, ./.ckm/ckm.json\n\
         Use --file <path> to specify a manifest."
            .to_string(),
    )
}

/// Reads and parses a JSON file from disk.
fn read_json(path: &PathBuf) -> Result<Value, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse {} as JSON: {}", path.display(), e))
}

// ─── Command Handlers ──────────────────────────────────────────────────

fn cmd_browse(topic: Option<String>, json: bool, file: Option<PathBuf>) {
    let path = match resolve_manifest_path(file) {
        Ok(p) => p,
        Err(msg) => {
            eprintln!("Error: {}", msg);
            process::exit(1);
        }
    };

    let data = match read_json(&path) {
        Ok(d) => d,
        Err(msg) => {
            eprintln!("Error: {}", msg);
            process::exit(1);
        }
    };

    let engine = CkmEngine::new(data);

    if json {
        let result = engine.topic_json(topic.as_deref());
        match serde_json::to_string_pretty(&result) {
            Ok(output) => println!("{}", output),
            Err(e) => {
                eprintln!("Error: Failed to serialize JSON: {}", e);
                process::exit(1);
            }
        }
    } else {
        match topic {
            None => {
                let project = engine.manifest().meta.project.clone();
                println!("{}", engine.topic_index(&project));
            }
            Some(ref name) => match engine.topic_content(name) {
                Some(content) => println!("{}", content),
                None => {
                    let available: Vec<&str> =
                        engine.topics().iter().map(|t| t.name.as_str()).collect();
                    eprintln!("Error: Unknown topic \"{}\"", name);
                    if available.is_empty() {
                        eprintln!("No topics available in this manifest.");
                    } else {
                        eprintln!("Available topics: {}", available.join(", "));
                    }
                    process::exit(1);
                }
            },
        }
    }
}

fn cmd_validate(file: PathBuf) {
    let data = match read_json(&file) {
        Ok(d) => d,
        Err(msg) => {
            eprintln!("Error: {}", msg);
            process::exit(1);
        }
    };

    let result = validate_manifest(&data);

    if result.valid {
        println!("Valid: {} passes v2 schema validation.", file.display());
    } else {
        eprintln!(
            "Invalid: {} has {} error(s):\n",
            file.display(),
            result.errors.len()
        );
        for err in &result.errors {
            eprintln!("  {} — {}", err.path, err.message);
        }
        process::exit(1);
    }
}

fn cmd_migrate(file: PathBuf, dry_run: bool, output: Option<PathBuf>) {
    let data = match read_json(&file) {
        Ok(d) => d,
        Err(msg) => {
            eprintln!("Error: {}", msg);
            process::exit(1);
        }
    };

    let version = detect_version(&data);
    if version == 2 {
        println!("{} is already v2. No migration needed.", file.display());
        return;
    }

    let migrated = migrate_v1_to_v2(&data);
    let json_output = match serde_json::to_string_pretty(&migrated) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error: Failed to serialize migrated manifest: {}", e);
            process::exit(1);
        }
    };

    if dry_run {
        println!("{}", json_output);
        return;
    }

    let dest = output.unwrap_or(file);
    match fs::write(&dest, format!("{}\n", json_output)) {
        Ok(()) => println!("Migrated v1 -> v2: {}", dest.display()),
        Err(e) => {
            eprintln!("Error: Failed to write {}: {}", dest.display(), e);
            process::exit(1);
        }
    }
}

fn cmd_inspect(file: PathBuf) {
    let data = match read_json(&file) {
        Ok(d) => d,
        Err(msg) => {
            eprintln!("Error: {}", msg);
            process::exit(1);
        }
    };

    let engine = CkmEngine::new(data);
    let info = engine.inspect();

    println!("Manifest: {}", file.display());
    println!("Project:  {}", info.meta.project);
    println!("Language: {}", info.meta.language);
    println!("Generator: {}", info.meta.generator);
    println!("Generated: {}", info.meta.generated);
    println!();
    println!("Counts:");
    println!("  Concepts:    {}", info.counts.concepts);
    println!("  Operations:  {}", info.counts.operations);
    println!("  Constraints: {}", info.counts.constraints);
    println!("  Workflows:   {}", info.counts.workflows);
    println!("  Config keys: {}", info.counts.config_keys);
    println!("  Topics:      {}", info.counts.topics);

    if !info.topic_names.is_empty() {
        println!();
        println!("Topics: {}", info.topic_names.join(", "));
    }
}

// ─── Main ──────────────────────────────────────────────────────────────

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Browse { topic, json, file } => cmd_browse(topic, json, file),
        Commands::Validate { file } => cmd_validate(file),
        Commands::Migrate {
            file,
            dry_run,
            output,
        } => cmd_migrate(file, dry_run, output),
        Commands::Inspect { file } => cmd_inspect(file),
    }
}
