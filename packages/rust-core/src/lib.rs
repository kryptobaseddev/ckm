//! CKM Core — the Single Source of Truth for all CKM language SDKs.
//!
//! This crate contains ALL CKM algorithms: types, engine, migration,
//! validation, and formatting. It has zero FFI concerns. Language
//! wrappers (napi-rs, PyO3, CGo) depend on this crate.
//!
//! # Quick Start
//!
//! ```rust
//! use ckm::CkmEngine;
//! use serde_json::json;
//!
//! let data = json!({
//!     "$schema": "https://ckm.dev/schemas/v2.json",
//!     "version": "2.0.0",
//!     "meta": {
//!         "project": "my-tool",
//!         "language": "rust",
//!         "generator": "hand-authored",
//!         "generated": "2026-01-01T00:00:00.000Z"
//!     },
//!     "concepts": [],
//!     "operations": [],
//!     "constraints": [],
//!     "workflows": [],
//!     "configSchema": []
//! });
//!
//! let engine = CkmEngine::new(data);
//! let index = engine.topic_index("my-tool");
//! println!("{}", index);
//! ```

pub mod engine;
pub mod format;
pub mod migrate;
pub mod types;
pub mod validate;

// Re-exports for convenience
pub use engine::CkmEngine;
pub use migrate::{detect_version, migrate_v1_to_v2};
pub use types::{
    CanonicalType, CkmConcept, CkmConfigEntry, CkmConstraint, CkmErrorResult, CkmInput,
    CkmInspectResult, CkmManifest, CkmMeta, CkmOperation, CkmOutput, CkmProperty, CkmTopic,
    CkmTopicIndex, CkmTopicIndexEntry, CkmTypeRef, CkmValidationError, CkmValidationResult,
    CkmWorkflow, CkmWorkflowStep, TopicJsonResult,
};
pub use validate::validate_manifest;
