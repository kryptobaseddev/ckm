package ckm

import "encoding/json"

// ValidateManifest validates a parsed JSON object against the ckm.json v2 schema.
// v1 manifests will fail validation because they lack required v2 fields.
//
// All validation logic lives in Rust. This function marshals the input to JSON,
// calls the Rust FFI, and deserializes the result.
func ValidateManifest(data json.RawMessage) *ValidationResult {
	raw := ffiValidateManifest(data)
	var result ValidationResult
	if err := json.Unmarshal(raw, &result); err != nil {
		return &ValidationResult{
			Valid: false,
			Errors: []ValidationError{
				{Path: "", Message: "failed to parse validation result from FFI"},
			},
		}
	}
	return &result
}
