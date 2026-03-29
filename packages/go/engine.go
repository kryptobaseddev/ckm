package ckm

import (
	"encoding/json"
	"fmt"
	"runtime"
)

// Engine is the core CKM engine. It holds an opaque handle to the Rust-side
// CkmEngine, which owns the parsed manifest and derived topics. All computation
// happens in Rust; Go is a thin deserialization layer.
type Engine struct {
	handle engineHandle
}

// NewEngine creates a CKM engine from raw JSON data. If the data is a v1
// manifest, the Rust core automatically migrates it to v2. Topics are derived
// at construction time and the engine is immutable thereafter.
//
// The returned Engine must not be copied. It is garbage-collected via a
// runtime finalizer that calls into Rust to free the underlying memory.
func NewEngine(data json.RawMessage) (*Engine, error) {
	h, err := ffiCreateEngine(data)
	if err != nil {
		return nil, fmt.Errorf("creating engine: %w", err)
	}

	e := &Engine{handle: h}
	runtime.SetFinalizer(e, func(e *Engine) {
		ffiDestroyEngine(e.handle)
	})
	return e, nil
}

// Close explicitly releases the Rust-side engine. After Close, all other
// methods will return zero values. Close is idempotent.
// If Close is not called, the finalizer will clean up eventually.
func (e *Engine) Close() {
	if e.handle != 0 {
		ffiDestroyEngine(e.handle)
		e.handle = 0
		runtime.SetFinalizer(e, nil)
	}
}

// TopicIndex returns the formatted topic index for terminal display (Level 0).
// The toolName parameter is used in the output header.
func (e *Engine) TopicIndex(toolName string) string {
	if e.handle == 0 {
		return ""
	}
	return ffiTopicIndex(e.handle, toolName)
}

// TopicContent returns human-readable content for a specific topic (Level 1).
// Returns nil if the topic is not found.
func (e *Engine) TopicContent(topicName string) *string {
	if e.handle == 0 {
		return nil
	}
	return ffiTopicContent(e.handle, topicName)
}

// TopicJSON returns structured JSON data. If topicName is nil, it returns
// the full TopicIndex (Level 2). If topicName matches a topic, it returns
// that Topic (Level 1J). If topicName does not match, it returns an ErrorResult.
//
// The caller receives raw JSON and can unmarshal into the appropriate Go type
// (TopicIndex, Topic, or ErrorResult) based on context.
func (e *Engine) TopicJSON(topicName *string) json.RawMessage {
	if e.handle == 0 {
		return json.RawMessage("null")
	}
	return ffiTopicJSON(e.handle, topicName)
}

// Manifest returns the v2 manifest (possibly migrated from v1).
func (e *Engine) Manifest() *Manifest {
	if e.handle == 0 {
		return nil
	}
	raw := ffiManifest(e.handle)
	var m Manifest
	if err := json.Unmarshal(raw, &m); err != nil {
		return nil
	}
	return &m
}

// Inspect returns manifest statistics: metadata, counts, and topic names.
func (e *Engine) Inspect() *InspectResult {
	if e.handle == 0 {
		return nil
	}
	raw := ffiInspect(e.handle)
	var r InspectResult
	if err := json.Unmarshal(raw, &r); err != nil {
		return nil
	}
	return &r
}
