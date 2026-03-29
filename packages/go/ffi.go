package ckm

// #cgo LDFLAGS: -L${SRCDIR}/lib -lckm_core
//
// // FFI contract: every function takes a NUL-terminated JSON C string and
// // returns a heap-allocated NUL-terminated JSON C string. The caller must
// // free the returned string with ckm_free_string.
// //
// // The Rust side exposes these via #[no_mangle] extern "C" functions in a
// // thin FFI shim crate (or a cdylib target on ckm-core). Example Rust:
// //
// //   #[no_mangle]
// //   pub extern "C" fn ckm_create_engine(json_ptr: *const c_char) -> *mut c_char { ... }
// //
// // Build the shared library:
// //   cd packages/rust-core
// //   cargo build --release          # produces libckm_core.so / .dylib / .dll
// //   cp target/release/libckm_core.* ../go/lib/
// //
// // NOTE: The Rust Cargo.toml must include:
// //   [lib]
// //   crate-type = ["lib", "cdylib"]
//
// #include <stdlib.h>
//
// // Engine lifecycle
// extern char* ckm_create_engine(const char* manifest_json);
// extern void  ckm_destroy_engine(unsigned long long handle);
//
// // Engine queries — each takes the engine handle + optional args as JSON,
// // returns a JSON string that must be freed with ckm_free_string.
// extern char* ckm_topic_index(unsigned long long handle, const char* tool_name);
// extern char* ckm_topic_content(unsigned long long handle, const char* topic_name);
// extern char* ckm_topic_json(unsigned long long handle, const char* topic_name);
// extern char* ckm_manifest(unsigned long long handle);
// extern char* ckm_inspect(unsigned long long handle);
//
// // Standalone functions (no engine handle needed)
// extern char* ckm_validate_manifest(const char* manifest_json);
// extern int   ckm_detect_version(const char* manifest_json);
// extern char* ckm_migrate_v1_to_v2(const char* manifest_json);
//
// // Memory management
// extern void  ckm_free_string(char* ptr);
import "C"

import (
	"encoding/json"
	"fmt"
	"unsafe"
)

// engineHandle is an opaque handle returned by the Rust FFI layer.
// It represents a pointer to a heap-allocated CkmEngine on the Rust side.
type engineHandle uint64

// ffiCreateEngine passes raw manifest JSON to Rust, which constructs a
// CkmEngine (auto-migrating v1 if needed) and returns an opaque handle.
// The handle must be destroyed with ffiDestroyEngine when no longer needed.
func ffiCreateEngine(data json.RawMessage) (engineHandle, error) {
	cJSON := C.CString(string(data))
	defer C.free(unsafe.Pointer(cJSON))

	result := C.ckm_create_engine(cJSON)
	if result == nil {
		return 0, fmt.Errorf("ckm_create_engine returned nil")
	}
	defer C.ckm_free_string(result)

	// The result is a JSON object: {"handle": <uint64>} on success,
	// or {"error": "message"} on failure.
	goResult := C.GoString(result)
	var resp struct {
		Handle uint64 `json:"handle"`
		Error  string `json:"error"`
	}
	if err := json.Unmarshal([]byte(goResult), &resp); err != nil {
		return 0, fmt.Errorf("failed to parse FFI response: %w", err)
	}
	if resp.Error != "" {
		return 0, fmt.Errorf("engine creation failed: %s", resp.Error)
	}
	return engineHandle(resp.Handle), nil
}

// ffiDestroyEngine releases the Rust-side CkmEngine associated with the handle.
func ffiDestroyEngine(h engineHandle) {
	C.ckm_destroy_engine(C.ulonglong(h))
}

// ffiTopicIndex calls the Rust engine's topic_index method.
func ffiTopicIndex(h engineHandle, toolName string) string {
	cToolName := C.CString(toolName)
	defer C.free(unsafe.Pointer(cToolName))

	result := C.ckm_topic_index(C.ulonglong(h), cToolName)
	if result == nil {
		return ""
	}
	defer C.ckm_free_string(result)
	return C.GoString(result)
}

// ffiTopicContent calls the Rust engine's topic_content method.
// Returns nil if the topic is not found (Rust returns null JSON).
func ffiTopicContent(h engineHandle, topicName string) *string {
	cTopicName := C.CString(topicName)
	defer C.free(unsafe.Pointer(cTopicName))

	result := C.ckm_topic_content(C.ulonglong(h), cTopicName)
	if result == nil {
		return nil
	}
	defer C.ckm_free_string(result)

	goResult := C.GoString(result)
	if goResult == "null" || goResult == "" {
		return nil
	}
	// Result is a JSON string (quoted). Unmarshal to get the raw value.
	var s string
	if err := json.Unmarshal([]byte(goResult), &s); err != nil {
		// If it's not JSON-quoted, return as-is
		return &goResult
	}
	return &s
}

// ffiTopicJSON calls the Rust engine's topic_json method.
// topicName is nil for the index, or a topic name for detail.
func ffiTopicJSON(h engineHandle, topicName *string) json.RawMessage {
	var cTopicName *C.char
	if topicName != nil {
		cTopicName = C.CString(*topicName)
		defer C.free(unsafe.Pointer(cTopicName))
	}

	result := C.ckm_topic_json(C.ulonglong(h), cTopicName)
	if result == nil {
		return json.RawMessage("null")
	}
	defer C.ckm_free_string(result)
	return json.RawMessage(C.GoString(result))
}

// ffiManifest calls the Rust engine's manifest method, returning the
// full v2 manifest as JSON.
func ffiManifest(h engineHandle) json.RawMessage {
	result := C.ckm_manifest(C.ulonglong(h))
	if result == nil {
		return json.RawMessage("null")
	}
	defer C.ckm_free_string(result)
	return json.RawMessage(C.GoString(result))
}

// ffiInspect calls the Rust engine's inspect method.
func ffiInspect(h engineHandle) json.RawMessage {
	result := C.ckm_inspect(C.ulonglong(h))
	if result == nil {
		return json.RawMessage("null")
	}
	defer C.ckm_free_string(result)
	return json.RawMessage(C.GoString(result))
}

// ffiValidateManifest calls the standalone Rust validation function.
func ffiValidateManifest(data json.RawMessage) json.RawMessage {
	cJSON := C.CString(string(data))
	defer C.free(unsafe.Pointer(cJSON))

	result := C.ckm_validate_manifest(cJSON)
	if result == nil {
		return json.RawMessage(`{"valid":false,"errors":[{"path":"","message":"FFI call returned nil"}]}`)
	}
	defer C.ckm_free_string(result)
	return json.RawMessage(C.GoString(result))
}

// ffiDetectVersion calls the standalone Rust version detection function.
func ffiDetectVersion(data json.RawMessage) int {
	cJSON := C.CString(string(data))
	defer C.free(unsafe.Pointer(cJSON))

	return int(C.ckm_detect_version(cJSON))
}

// ffiMigrateV1ToV2 calls the standalone Rust migration function.
func ffiMigrateV1ToV2(data json.RawMessage) (json.RawMessage, error) {
	cJSON := C.CString(string(data))
	defer C.free(unsafe.Pointer(cJSON))

	result := C.ckm_migrate_v1_to_v2(cJSON)
	if result == nil {
		return nil, fmt.Errorf("ckm_migrate_v1_to_v2 returned nil")
	}
	defer C.ckm_free_string(result)

	goResult := C.GoString(result)

	// Check for error response
	var errResp struct {
		Error string `json:"error"`
	}
	if err := json.Unmarshal([]byte(goResult), &errResp); err == nil && errResp.Error != "" {
		return nil, fmt.Errorf("migration failed: %s", errResp.Error)
	}

	return json.RawMessage(goResult), nil
}
