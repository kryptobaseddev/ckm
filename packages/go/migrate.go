package ckm

import "encoding/json"

// DetectVersion returns the schema version of a parsed manifest.
// Returns 2 for v2 manifests (those with a "meta" object or "$schema" containing "v2").
// Returns 1 otherwise (including malformed data).
//
// All detection logic lives in Rust.
func DetectVersion(data json.RawMessage) int {
	return ffiDetectVersion(data)
}

// MigrateV1ToV2 performs a deterministic migration from v1 format to v2 format.
// All migration logic (slug derivation, tag inference, type mapping, config key
// normalization) lives in Rust.
func MigrateV1ToV2(data json.RawMessage) (*Manifest, error) {
	raw, err := ffiMigrateV1ToV2(data)
	if err != nil {
		return nil, err
	}
	var m Manifest
	if err := json.Unmarshal(raw, &m); err != nil {
		return nil, err
	}
	return &m, nil
}
