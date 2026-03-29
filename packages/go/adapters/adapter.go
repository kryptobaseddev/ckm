// Package adapters provides CLI framework adapters for the CKM engine.
//
// Each adapter bridges the CKM engine to a specific CLI framework (e.g., Cobra,
// urfave/cli). Adapters are optional and live in a separate subpackage so the
// core ckm library has zero external dependencies.
package adapters

import (
	"encoding/json"

	ckm "github.com/kryptobaseddev/ckm/go"
)

// Adapter is the interface that CLI framework adapters must implement.
// The generic concept is represented by accepting interface{} for the program
// parameter, with concrete adapters performing type assertions internally.
type Adapter interface {
	// Name returns the adapter identifier (e.g., "cobra", "urfave").
	Name() string

	// Framework returns the framework display name (e.g., "Cobra", "urfave/cli").
	Framework() string

	// Register wires a "ckm [topic]" subcommand onto the host CLI program.
	// The program parameter is framework-specific and must be type-asserted
	// by the concrete adapter implementation.
	Register(program interface{}, engine *ckm.Engine, options *Options)
}

// Options configures the CKM adapter registration.
type Options struct {
	// CommandName is the subcommand name to register (default: "ckm").
	CommandName string

	// ToolName is the tool name in help output (default: inferred from program).
	ToolName string

	// Formatter provides custom output formatting. If nil, built-in formatters are used.
	Formatter Formatter
}

// Formatter allows consumers to override the default plain-text formatting.
type Formatter interface {
	// FormatIndex formats the topic index for terminal display.
	FormatIndex(topics []ckm.Topic, toolName string) string

	// FormatTopic formats a single topic's content for terminal display.
	FormatTopic(topic ckm.Topic) string

	// FormatJSON formats JSON output (default: json.MarshalIndent with 2-space indent).
	FormatJSON(data json.RawMessage) string
}

// DefaultCommandName returns the command name from options, falling back to "ckm".
func DefaultCommandName(opts *Options) string {
	if opts != nil && opts.CommandName != "" {
		return opts.CommandName
	}
	return "ckm"
}

// DefaultToolName returns the tool name from options, falling back to the provided default.
func DefaultToolName(opts *Options, fallback string) string {
	if opts != nil && opts.ToolName != "" {
		return opts.ToolName
	}
	return fallback
}
