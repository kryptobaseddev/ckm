/**
 * CKM Clipanion adapter.
 *
 * @remarks
 * Registers CKM as a Clipanion command. Clipanion is a peerDependency.
 *
 * @packageDocumentation
 */

import type { CkmEngine } from '../types.js';
import type { CkmAdapterOptions, CkmCliAdapter } from './types.js';

/**
 * Clipanion adapter for CKM.
 *
 * @public
 */
const adapter: CkmCliAdapter = {
  name: 'clipanion',
  framework: 'Clipanion',

  register(program: unknown, engine: CkmEngine, options?: CkmAdapterOptions): void {
    const toolName = options?.toolName ?? 'tool';
    const cli = program as {
      register: (command: Record<string, unknown>) => void;
    };

    cli.register({
      paths: [[options?.commandName ?? 'ckm']],
      usage: 'Browse Codebase Knowledge Manifest topics',
      execute: async (args: { topic?: string; json?: boolean }) => {
        if (args.json) {
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
