/**
 * CKM oclif adapter.
 *
 * @remarks
 * Registers CKM as an oclif command class. oclif is a peerDependency.
 *
 * @packageDocumentation
 */

import type { CkmEngine } from '../types.js';
import type { CkmAdapterOptions, CkmCliAdapter } from './types.js';

/**
 * oclif adapter for CKM.
 *
 * @remarks
 * Since oclif uses class-based commands, this adapter creates a
 * command class factory that the consumer can register.
 *
 * @public
 */
const adapter: CkmCliAdapter = {
  name: 'oclif',
  framework: 'oclif',

  register(program: unknown, engine: CkmEngine, options?: CkmAdapterOptions): void {
    const toolName = options?.toolName ?? 'tool';
    const registry = program as {
      register: (name: string, handler: Record<string, unknown>) => void;
    };

    registry.register(options?.commandName ?? 'ckm', {
      description: 'Codebase Knowledge Manifest',
      args: [{ name: 'topic', required: false }],
      flags: {
        json: { char: 'j', description: 'Machine-readable output' },
      },
      run: async (ctx: { args: { topic?: string }; flags: { json?: boolean } }) => {
        if (ctx.flags.json) {
          console.log(JSON.stringify(engine.getTopicJson(ctx.args.topic), null, 2));
        } else if (ctx.args.topic) {
          const content = engine.getTopicContent(ctx.args.topic);
          if (content === null) {
            console.error(`Unknown topic: ${ctx.args.topic}`);
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
