// Package main provides a standalone CLI binary for CKM.
//
// Usage:
//
//	ckm [topic] [--json] [--llm]
//	ckm inspect <path>
//	ckm validate <path>
//
// The CLI reads a ckm.json manifest and provides progressive disclosure
// of codebase knowledge at four levels.
package main

import (
	"encoding/json"
	"fmt"
	"os"

	ckm "github.com/kryptobaseddev/ckm/go"
)

func main() {
	if len(os.Args) < 2 {
		fmt.Fprintln(os.Stderr, "Usage: ckm <command> [args]")
		fmt.Fprintln(os.Stderr, "")
		fmt.Fprintln(os.Stderr, "Commands:")
		fmt.Fprintln(os.Stderr, "  topics [topic] [--json]   Show topic index or topic detail")
		fmt.Fprintln(os.Stderr, "  inspect <path>            Inspect a ckm.json manifest")
		fmt.Fprintln(os.Stderr, "  validate <path>           Validate a ckm.json manifest")
		os.Exit(1)
	}

	switch os.Args[1] {
	case "topics":
		cmdTopics(os.Args[2:])
	case "inspect":
		cmdInspect(os.Args[2:])
	case "validate":
		cmdValidate(os.Args[2:])
	case "--help", "-h":
		fmt.Fprintln(os.Stdout, "Usage: ckm <command> [args]")
		fmt.Fprintln(os.Stdout, "")
		fmt.Fprintln(os.Stdout, "Commands:")
		fmt.Fprintln(os.Stdout, "  topics [topic] [--json]   Show topic index or topic detail")
		fmt.Fprintln(os.Stdout, "  inspect <path>            Inspect a ckm.json manifest")
		fmt.Fprintln(os.Stdout, "  validate <path>           Validate a ckm.json manifest")
	default:
		// If the first arg is a file path, treat as: ckm topics --file <path>
		fmt.Fprintf(os.Stderr, "Unknown command: %s\n", os.Args[1])
		os.Exit(1)
	}
}

func cmdTopics(args []string) {
	var filePath string
	var topicName string
	var jsonFlag bool

	for i := 0; i < len(args); i++ {
		switch args[i] {
		case "--file", "-f":
			if i+1 < len(args) {
				i++
				filePath = args[i]
			}
		case "--json":
			jsonFlag = true
		default:
			if topicName == "" {
				topicName = args[i]
			}
		}
	}

	if filePath == "" {
		filePath = "ckm.json"
	}

	engine, err := loadEngine(filePath)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error: %s\n", err)
		os.Exit(1)
	}

	if jsonFlag {
		var topicPtr *string
		if topicName != "" {
			topicPtr = &topicName
		}
		data := engine.TopicJSON(topicPtr)
		// data is already JSON from FFI; indent for display
		var parsed json.RawMessage
		if err := json.Unmarshal(data, &parsed); err == nil {
			out, _ := json.MarshalIndent(parsed, "", "  ")
			fmt.Fprintln(os.Stdout, string(out))
		} else {
			fmt.Fprintln(os.Stdout, string(data))
		}
	} else if topicName != "" {
		content := engine.TopicContent(topicName)
		if content == nil {
			fmt.Fprintf(os.Stderr, "Unknown topic: %s\n", topicName)
			fmt.Fprintln(os.Stdout, engine.TopicIndex("ckm"))
			os.Exit(1)
		}
		fmt.Fprintln(os.Stdout, *content)
	} else {
		fmt.Fprintln(os.Stdout, engine.TopicIndex("ckm"))
	}
}

func cmdInspect(args []string) {
	filePath := "ckm.json"
	if len(args) > 0 {
		filePath = args[0]
	}

	engine, err := loadEngine(filePath)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error: %s\n", err)
		os.Exit(1)
	}

	result := engine.Inspect()
	out, _ := json.MarshalIndent(result, "", "  ")
	fmt.Fprintln(os.Stdout, string(out))
}

func cmdValidate(args []string) {
	filePath := "ckm.json"
	if len(args) > 0 {
		filePath = args[0]
	}

	data, err := os.ReadFile(filePath)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error reading file: %s\n", err)
		os.Exit(1)
	}

	result := ckm.ValidateManifest(json.RawMessage(data))
	out, _ := json.MarshalIndent(result, "", "  ")
	fmt.Fprintln(os.Stdout, string(out))

	if !result.Valid {
		os.Exit(1)
	}
}

func loadEngine(filePath string) (*ckm.Engine, error) {
	data, err := os.ReadFile(filePath)
	if err != nil {
		return nil, fmt.Errorf("reading file: %w", err)
	}
	return ckm.NewEngine(json.RawMessage(data))
}
