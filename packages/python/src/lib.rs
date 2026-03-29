//! `PyO3` wrapper for `ckm-core` — thin FFI layer exposing the Rust CKM engine to Python.
//!
//! All logic lives in `ckm`. This module only handles:
//! - JSON string to `serde_json::Value` conversion
//! - `serde_json::Value` to Python dict conversion (via JSON round-trip)
//! - `PyO3` class/function registration

use pyo3::prelude::*;
use pyo3::types::PyAnyMethods;
use serde_json::Value;

// ─── Helpers ───────────────────────────────────────────────────────────────

/// Parse a JSON string, returning a `PyErr` on failure.
fn parse_json(data: &str) -> PyResult<Value> {
    serde_json::from_str(data)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid JSON: {e}")))
}

/// Convert a serializable Rust value into a Python object via JSON round-trip.
fn to_py_object<T: serde::Serialize>(py: Python<'_>, val: &T) -> PyResult<Py<PyAny>> {
    let json_str = serde_json::to_string(val).map_err(|e| {
        pyo3::exceptions::PyRuntimeError::new_err(format!("Serialization error: {e}"))
    })?;
    let json_mod = py.import("json")?;
    let result = json_mod.call_method1("loads", (json_str,))?;
    Ok(result.unbind())
}

// ─── CkmEngine wrapper ────────────────────────────────────────────────────

/// CKM engine — auto-generates topic index from a `ckm.json` manifest.
///
/// Wraps the Rust `ckm::CkmEngine`. Accepts and returns JSON strings
/// or Python dicts for structured data.
#[pyclass]
struct CkmEngine {
    inner: ckm::CkmEngine,
}

#[pymethods]
impl CkmEngine {
    /// Returns formatted topic index for terminal display (Level 0).
    ///
    /// `tool_name`: Optional tool name for the header. Defaults to "ckm".
    ///
    /// Returns a formatted string suitable for terminal output.
    #[pyo3(signature = (tool_name=None))]
    fn get_topic_index(&self, tool_name: Option<&str>) -> String {
        self.inner.topic_index(tool_name.unwrap_or("ckm"))
    }

    /// Returns human-readable content for a specific topic (Level 1).
    ///
    /// `topic_name`: The topic slug to look up.
    ///
    /// Returns a formatted string, or `None` if the topic is not found.
    fn get_topic_content(&self, topic_name: &str) -> Option<String> {
        self.inner.topic_content(topic_name)
    }

    /// Returns structured JSON data for a topic or the full index.
    ///
    /// `topic_name`: Optional topic slug. If `None`, returns the full index.
    ///
    /// Returns a Python dict with topic data, index data, or error information.
    #[pyo3(signature = (topic_name=None))]
    fn get_topic_json(&self, py: Python<'_>, topic_name: Option<&str>) -> PyResult<Py<PyAny>> {
        let result = self.inner.topic_json(topic_name);
        to_py_object(py, &result)
    }

    /// Returns the raw v2 manifest as a Python dict.
    fn get_manifest(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        to_py_object(py, self.inner.manifest())
    }

    /// Returns manifest statistics: metadata, counts, and topic names.
    fn inspect(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        to_py_object(py, &self.inner.inspect())
    }

    /// Returns the number of derived topics.
    fn topics_count(&self) -> usize {
        self.inner.topics().len()
    }
}

// ─── Module-level functions ────────────────────────────────────────────────

/// Create a CKM engine from a JSON manifest string.
///
/// Accepts both v1 and v2 manifests. V1 manifests are auto-migrated to v2.
///
/// `manifest`: JSON string of the `ckm.json` manifest.
///
/// Returns a `CkmEngine` instance.
///
/// # Errors
///
/// Returns `PyValueError` if the input is not valid JSON.
#[pyfunction]
fn create_engine(manifest: &str) -> PyResult<CkmEngine> {
    let data = parse_json(manifest)?;
    Ok(CkmEngine {
        inner: ckm::CkmEngine::new(data),
    })
}

/// Validate a manifest JSON string against the v2 schema.
///
/// `data`: JSON string to validate.
///
/// Returns a Python dict with "valid" (bool) and "errors" (list) keys.
///
/// # Errors
///
/// Returns `PyValueError` if the input is not valid JSON.
#[pyfunction]
fn validate_manifest(py: Python<'_>, data: &str) -> PyResult<Py<PyAny>> {
    let parsed = parse_json(data)?;
    let result = ckm::validate_manifest(&parsed);
    to_py_object(py, &result)
}

/// Migrate a v1 manifest to v2 format.
///
/// `data`: JSON string of a v1 manifest.
///
/// Returns a Python dict of the migrated v2 manifest.
///
/// # Errors
///
/// Returns `PyValueError` if the input is not valid JSON.
#[pyfunction]
fn migrate_v1_to_v2(py: Python<'_>, data: &str) -> PyResult<Py<PyAny>> {
    let parsed = parse_json(data)?;
    let result = ckm::migrate_v1_to_v2(&parsed);
    to_py_object(py, &result)
}

/// Detect the schema version of a manifest.
///
/// `data`: JSON string of a manifest.
///
/// Returns 1 for v1 manifests, 2 for v2 manifests.
///
/// # Errors
///
/// Returns `PyValueError` if the input is not valid JSON.
#[pyfunction]
fn detect_version(data: &str) -> PyResult<u8> {
    let parsed = parse_json(data)?;
    Ok(ckm::detect_version(&parsed))
}

// ─── Module registration ───────────────────────────────────────────────────

/// CKM — Codebase Knowledge Manifest SDK (powered by Rust core).
#[pymodule]
fn ckm(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<CkmEngine>()?;
    m.add_function(wrap_pyfunction!(create_engine, m)?)?;
    m.add_function(wrap_pyfunction!(validate_manifest, m)?)?;
    m.add_function(wrap_pyfunction!(migrate_v1_to_v2, m)?)?;
    m.add_function(wrap_pyfunction!(detect_version, m)?)?;
    Ok(())
}
