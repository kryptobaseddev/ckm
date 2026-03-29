/**
 * CKM CLI Adapter interfaces.
 *
 * @remarks
 * These interfaces define the contract for adapters that wire CKM
 * into different CLI frameworks (Commander.js, Citty, oclif, etc.).
 * Adapters are peer dependencies — `npm install ckm` does NOT
 * install any CLI framework.
 *
 * @packageDocumentation
 */

import type { CkmEngine, CkmTopic } from '../types.js';

/**
 * Adapter interface for wiring CKM into a CLI framework.
 *
 * @typeParam TProgram - The CLI framework's program/command type.
 *
 * @public
 */
export interface CkmCliAdapter<TProgram = unknown> {
  /** Adapter identifier (e.g., "commander", "citty", "oclif"). */
  readonly name: string;

  /** Framework display name (e.g., "Commander.js", "Citty", "oclif"). */
  readonly framework: string;

  /**
   * Registers a `ckm [topic]` subcommand on the host program.
   *
   * @param program - The host CLI program object.
   * @param engine - A configured CKM engine instance.
   * @param options - Optional adapter configuration.
   */
  register(program: TProgram, engine: CkmEngine, options?: CkmAdapterOptions): void;
}

/**
 * Options for configuring a CKM adapter.
 *
 * @public
 */
export interface CkmAdapterOptions {
  /** Subcommand name to register (default: "ckm"). */
  commandName?: string;

  /** Tool name in help output (default: inferred from program). */
  toolName?: string;

  /** Custom output formatter. */
  formatter?: CkmFormatter;
}

/**
 * Custom formatter interface for adapter output.
 *
 * @public
 */
export interface CkmFormatter {
  /** Formats the topic index for terminal display. */
  formatIndex(topics: CkmTopic[], toolName: string): string;

  /** Formats a single topic's content for terminal display. */
  formatTopic(topic: CkmTopic): string;

  /** Formats JSON output. */
  formatJson(data: unknown): string;
}
