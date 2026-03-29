/**
 * CKM Citty adapter.
 *
 * @remarks
 * Registers CKM as a Citty subcommand. Citty is a peerDependency.
 *
 * @packageDocumentation
 */

import type { CkmEngine } from '../types.js';
import type { CkmAdapterOptions, CkmCliAdapter } from './types.js';

/**
 * Citty adapter for CKM.
 *
 * @public
 */
const adapter: CkmCliAdapter = {
  name: 'citty',
  framework: 'Citty',

  register(program: unknown, engine: CkmEngine, options?: CkmAdapterOptions): void {
    const citty = program as {
      command: (name: string, def: Record<string, unknown>) => void;
    };
    const cmdName = options?.commandName ?? 'ckm';
    const toolName = options?.toolName ?? 'tool';

    citty.command(cmdName, {
      meta: { description: 'Codebase Knowledge Manifest' },
      args: { topic: { type: 'positional', required: false } },
      flags: {
        json: { type: 'boolean', description: 'Machine-readable output' },
      },
      run: ({ args, flags }: { args: { topic?: string }; flags: { json?: boolean } }) => {
        if (flags.json) {
          console.log(JSON.stringify(engine.getTopicJson(args.topic), null, 2));
        } else if (args.topic) {
          const content = engine.getTopicContent(args.topic);
          if (content === null) {
            console.error(`Unknown topic: ${args.topic}`);
            console.log(engine.getTopicIndex(toolName));
          } else {
            console.log(content);
          }
        } else {
          console.log(engine.getTopicIndex(toolName));
        }
      },
    });
  },
};

export default adapter;
