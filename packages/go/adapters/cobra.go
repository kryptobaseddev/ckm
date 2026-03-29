package adapters

import (
	"encoding/json"
	"fmt"
	"os"

	ckm "github.com/kryptobaseddev/ckm/go"
	"github.com/spf13/cobra"
)

// CobraAdapter wires CKM into a Cobra CLI program.
type CobraAdapter struct{}

// NewCobraAdapter creates a new Cobra adapter.
func NewCobraAdapter() *CobraAdapter {
	return &CobraAdapter{}
}

// Name returns the adapter identifier.
func (a *CobraAdapter) Name() string {
	return "cobra"
}

// Framework returns the framework display name.
func (a *CobraAdapter) Framework() string {
	return "Cobra"
}

// Register adds a "ckm [topic]" subcommand to the given Cobra root command.
// The program parameter must be a *cobra.Command.
func (a *CobraAdapter) Register(program interface{}, engine *ckm.Engine, options *Options) {
	rootCmd, ok := program.(*cobra.Command)
	if !ok {
		panic("CobraAdapter.Register: program must be *cobra.Command")
	}

	cmdName := DefaultCommandName(options)
	toolName := DefaultToolName(options, rootCmd.Name())

	var jsonFlag bool

	ckmCmd := &cobra.Command{
		Use:   fmt.Sprintf("%s [topic]", cmdName),
		Short: "Codebase Knowledge Manifest — auto-generated docs and help",
		Args:  cobra.MaximumNArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			if jsonFlag {
				var topic *string
				if len(args) > 0 {
					t := args[0]
					topic = &t
				}
				data := engine.TopicJSON(topic)
				if options != nil && options.Formatter != nil {
					fmt.Fprintln(os.Stdout, options.Formatter.FormatJSON(data))
				} else {
					// data is already JSON; indent it for display
					var parsed json.RawMessage
					if err := json.Unmarshal(data, &parsed); err == nil {
						out, _ := json.MarshalIndent(parsed, "", "  ")
						fmt.Fprintln(os.Stdout, string(out))
					} else {
						fmt.Fprintln(os.Stdout, string(data))
					}
				}
			} else if len(args) > 0 {
				content := engine.TopicContent(args[0])
				if content == nil {
					fmt.Fprintf(os.Stderr, "Unknown topic: %s\n", args[0])
					fmt.Fprintln(os.Stdout, engine.TopicIndex(toolName))
					os.Exit(1)
				} else {
					fmt.Fprintln(os.Stdout, *content)
				}
			} else {
				fmt.Fprintln(os.Stdout, engine.TopicIndex(toolName))
			}
		},
	}

	ckmCmd.Flags().BoolVar(&jsonFlag, "json", false, "Machine-readable CKM output for LLM agents")

	rootCmd.AddCommand(ckmCmd)
}
