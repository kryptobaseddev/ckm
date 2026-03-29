//! CKM Node.js wrapper — thin napi-rs bridge to ckm-core.
//!
//! All logic lives in `ckm`. This crate only handles JSON
//! serialization across the FFI boundary.

use napi::bindgen_prelude::*;
use napi_derive::napi;

// ─── Engine Wrapper ────────────────────────────────────────────────────

/// Wraps [`ckm::CkmEngine`] for use from Node.js via napi-rs.
#[napi]
pub struct CkmEngineWrapper {
    inner: ckm::CkmEngine,
}

#[napi]
impl CkmEngineWrapper {
    /// Returns the formatted topic index for terminal display.
    /// If `tool_name` is not provided, defaults to "tool".
    #[napi]
    pub fn get_topic_index(&self, tool_name: Option<String>) -> String {
        let name = tool_name.as_deref().unwrap_or("tool");
        self.inner.topic_index(name)
    }

    /// Returns human-readable content for a specific topic, or null if not found.
    #[napi]
    pub fn get_topic_content(&self, topic_name: String) -> Option<String> {
        self.inner.topic_content(&topic_name)
    }

    /// Returns structured JSON for a topic or the full index.
    ///
    /// Pass `undefined`/`null` to get the full topic index; pass a topic name
    /// to get that topic's detail or an error object.
    ///
    /// # Errors
    ///
    /// Returns an error if JSON serialization of the result fails.
    #[napi]
    pub fn get_topic_json(&self, topic_name: Option<String>) -> Result<String> {
        let result = self.inner.topic_json(topic_name.as_deref());
        serde_json::to_string(&result)
            .map_err(|e| Error::from_reason(format!("JSON serialization failed: {}", e)))
    }

    /// Returns the raw v2 manifest as a JSON string.
    ///
    /// # Errors
    ///
    /// Returns an error if JSON serialization of the manifest fails.
    #[napi]
    pub fn get_manifest(&self) -> Result<String> {
        serde_json::to_string(self.inner.manifest())
            .map_err(|e| Error::from_reason(format!("JSON serialization failed: {}", e)))
    }

    /// Returns manifest statistics (metadata, counts, topic names) as JSON.
    ///
    /// # Errors
    ///
    /// Returns an error if JSON serialization of the inspect result fails.
    #[napi]
    pub fn inspect(&self) -> Result<String> {
        let result = self.inner.inspect();
        serde_json::to_string(&result)
            .map_err(|e| Error::from_reason(format!("JSON serialization failed: {}", e)))
    }

    /// Returns the number of derived topics.
    #[napi]
    pub fn topics_count(&self) -> u32 {
        self.inner.topics().len() as u32
    }
}

// ─── Factory Function ──────────────────────────────────────────────────

/// Creates a new CKM engine from a JSON string (v1 or v2 manifest).
///
/// # Errors
///
/// Returns an error if the input string is not valid JSON.
#[napi]
pub fn create_ckm_engine(manifest_json: String) -> Result<CkmEngineWrapper> {
    let data: serde_json::Value = serde_json::from_str(&manifest_json)
        .map_err(|e| Error::from_reason(format!("Invalid JSON: {}", e)))?;
    Ok(CkmEngineWrapper {
        inner: ckm::CkmEngine::new(data),
    })
}

// ─── Standalone Functions ──────────────────────────────────────────────

/// Validates a manifest JSON string against the v2 schema.
///
/// Returns a JSON string with `{ valid: boolean, errors: [...] }`.
///
/// # Errors
///
/// Returns an error if the input is not valid JSON or serialization fails.
#[napi]
pub fn validate_manifest(json: String) -> Result<String> {
    let data: serde_json::Value = serde_json::from_str(&json)
        .map_err(|e| Error::from_reason(format!("Invalid JSON: {}", e)))?;
    let result = ckm::validate_manifest(&data);
    serde_json::to_string(&result)
        .map_err(|e| Error::from_reason(format!("JSON serialization failed: {}", e)))
}

/// Migrates a v1 manifest to v2 format.
///
/// Returns the v2 manifest as a JSON string.
///
/// # Errors
///
/// Returns an error if the input is not valid JSON or serialization fails.
#[napi]
pub fn migrate_v1_to_v2(json: String) -> Result<String> {
    let data: serde_json::Value = serde_json::from_str(&json)
        .map_err(|e| Error::from_reason(format!("Invalid JSON: {}", e)))?;
    let manifest = ckm::migrate_v1_to_v2(&data);
    serde_json::to_string(&manifest)
        .map_err(|e| Error::from_reason(format!("JSON serialization failed: {}", e)))
}

/// Detects whether a manifest is v1 or v2.
///
/// Returns 1 or 2.
///
/// # Errors
///
/// Returns an error if the input string is not valid JSON.
#[napi]
pub fn detect_version(json: String) -> Result<u32> {
    let data: serde_json::Value = serde_json::from_str(&json)
        .map_err(|e| Error::from_reason(format!("Invalid JSON: {}", e)))?;
    Ok(ckm::detect_version(&data) as u32)
}
