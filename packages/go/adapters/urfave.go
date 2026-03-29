package adapters

import (
	"encoding/json"
	"fmt"
	"os"

	ckm "github.com/kryptobaseddev/ckm/go"
	"github.com/urfave/cli/v2"
)

// UrfaveAdapter wires CKM into a urfave/cli v2 application.
type UrfaveAdapter struct{}

// NewUrfaveAdapter creates a new urfave/cli adapter.
func NewUrfaveAdapter() *UrfaveAdapter {
	return &UrfaveAdapter{}
}

// Name returns the adapter identifier.
func (a *UrfaveAdapter) Name() string {
	return "urfave"
}

// Framework returns the framework display name.
func (a *UrfaveAdapter) Framework() string {
	return "urfave/cli"
}

// Register adds a "ckm [topic]" subcommand to the given urfave/cli app.
// The program parameter must be a *cli.App.
func (a *UrfaveAdapter) Register(program interface{}, engine *ckm.Engine, options *Options) {
	app, ok := program.(*cli.App)
	if !ok {
		panic("UrfaveAdapter.Register: program must be *cli.App")
	}

	cmdName := DefaultCommandName(options)
	toolName := DefaultToolName(options, app.Name)

	ckmCommand := &cli.Command{
		Name:      cmdName,
		Usage:     "Codebase Knowledge Manifest — auto-generated docs and help",
		ArgsUsage: "[topic]",
		Flags: []cli.Flag{
			&cli.BoolFlag{
				Name:  "json",
				Usage: "Machine-readable CKM output for LLM agents",
			},
		},
		Action: func(c *cli.Context) error {
			jsonFlag := c.Bool("json")
			topic := c.Args().First()

			if jsonFlag {
				var topicPtr *string
				if topic != "" {
					topicPtr = &topic
				}
				data := engine.TopicJSON(topicPtr)
				if options != nil && options.Formatter != nil {
					fmt.Fprintln(os.Stdout, options.Formatter.FormatJSON(data))
				} else {
					var parsed json.RawMessage
					if err := json.Unmarshal(data, &parsed); err == nil {
						out, _ := json.MarshalIndent(parsed, "", "  ")
						fmt.Fprintln(os.Stdout, string(out))
					} else {
						fmt.Fprintln(os.Stdout, string(data))
					}
				}
			} else if topic != "" {
				content := engine.TopicContent(topic)
				if content == nil {
					fmt.Fprintf(os.Stderr, "Unknown topic: %s\n", topic)
					fmt.Fprintln(os.Stdout, engine.TopicIndex(toolName))
					return cli.Exit("", 1)
				}
				fmt.Fprintln(os.Stdout, *content)
			} else {
				fmt.Fprintln(os.Stdout, engine.TopicIndex(toolName))
			}
			return nil
		},
	}

	app.Commands = append(app.Commands, ckmCommand)
}
