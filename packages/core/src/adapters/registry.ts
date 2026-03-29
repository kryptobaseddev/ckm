/**
 * CKM Adapter Registry — lazy-loaded adapter table.
 *
 * @packageDocumentation
 */

import type { CkmCliAdapter } from './types.js';

/**
 * Lazy-loaded adapter table.
 *
 * @remarks
 * Each entry maps a framework identifier to a factory function
 * that dynamically imports the adapter module. This ensures that
 * `npm install ckm` does NOT force a dependency on any CLI framework.
 *
 * @public
 */
export const ADAPTER_TABLE: Record<string, () => Promise<CkmCliAdapter>> = {
  commander: () => import('./commander.js').then((m) => m.default),
  citty: () => import('./citty.js').then((m) => m.default),
  oclif: () => import('./oclif.js').then((m) => m.default),
  clipanion: () => import('./clipanion.js').then((m) => m.default),
};

/**
 * Loads an adapter by framework identifier.
 *
 * @param framework - Framework identifier (e.g., "commander").
 * @returns The loaded adapter instance.
 * @throws Error if the framework is not registered.
 *
 * @public
 */
export async function loadAdapter(framework: string): Promise<CkmCliAdapter> {
  const factory = ADAPTER_TABLE[framework];
  if (!factory) {
    const available = Object.keys(ADAPTER_TABLE).join(', ');
    throw new Error(`Unknown CKM adapter: "${framework}". Available: ${available}`);
  }
  return factory();
}
