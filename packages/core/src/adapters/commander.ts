/**
 * CKM Commander.js adapter.
 *
 * @remarks
 * Registers a `ckm [topic]` subcommand on a Commander program.
 * Commander is a peerDependency — this module is only loaded
 * when the consumer explicitly imports it.
 *
 * @packageDocumentation
 */

import type { Command } from 'commander';
import type { CkmEngine } from '../types.js';
import type { CkmAdapterOptions, CkmCliAdapter } from './types.js';

/**
 * Commander.js adapter for CKM.
 *
 * @public
 */
const adapter: CkmCliAdapter<Command> = {
  name: 'commander',
  framework: 'Commander.js',

  register(program: Command, engine: CkmEngine, options?: CkmAdapterOptions): void {
    const cmdName = options?.commandName ?? 'ckm';
    const toolName = options?.toolName ?? program.name();

    program
      .command(`${cmdName} [topic]`)
      .description('Codebase Knowledge Manifest — auto-generated docs and help')
      .option('--json', 'Machine-readable CKM output for LLM agents')
      .option('--llm', 'Full API context')
      .action((topic: string | undefined, flags: { json?: boolean; llm?: boolean }) => {
        if (flags.json) {
          const formatter = options?.formatter;
          const data = engine.getTopicJson(topic);
          if (formatter) {
            process.stdout.write(`${formatter.formatJson(data)}\n`);
          } else {
            process.stdout.write(`${JSON.stringify(data, null, 2)}\n`);
          }
        } else if (topic) {
          const content = engine.getTopicContent(topic);
          if (content === null) {
            process.stderr.write(`Unknown topic: ${topic}\n`);
            process.stdout.write(`${engine.getTopicIndex(toolName)}\n`);
            process.exitCode = 1;
          } else {
            process.stdout.write(`${content}\n`);
          }
        } else {
          process.stdout.write(`${engine.getTopicIndex(toolName)}\n`);
        }
      });
  },
};

export default adapter;
