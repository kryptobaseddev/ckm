/**
 * CKM (Codebase Knowledge Manifest) type definitions.
 *
 * @remarks
 * These types are the TypeScript implementation of INTERFACE.md v2.0.0.
 * Every type here has a 1:1 correspondence with the SSoT interface definition.
 * All input types use `readonly` to enforce immutability of manifest data.
 *
 * @packageDocumentation
 */

// ────────────────────────────────────────────────────────────────────────────
// Section 2: Input Types (from ckm.json v2)
// ────────────────────────────────────────────────────────────────────────────

/**
 * The set of portable primitive types, mapped to JSON Schema primitives.
 *
 * Used within {@link CkmTypeRef} to provide a language-agnostic type classification.
 *
 * @public
 */
export type CanonicalType =
  | 'string'
  | 'boolean'
  | 'number'
  | 'integer'
  | 'array'
  | 'object'
  | 'null'
  | 'any';

/**
 * A portable type reference with canonical mapping.
 *
 * Bridges language-specific types to a universal canonical representation,
 * enabling cross-language SDKs to interpret types consistently.
 *
 * @public
 */
export interface CkmTypeRef {
  /** Language-agnostic canonical type. */
  readonly canonical: CanonicalType;

  /** Source language type annotation (e.g., "CalVerFormat"). Null when not available. */
  readonly original?: string | null;

  /** Known enum values for string types (e.g., ["YYYY.MM.DD", "YYYY.MM"]). */
  readonly enum?: string[] | null;
}

/**
 * A property within a {@link CkmConcept}.
 *
 * Represents a single field on a domain type extracted from source code.
 *
 * @public
 */
export interface CkmProperty {
  /** Property name. */
  readonly name: string;

  /** Type reference (canonical + original). */
  readonly type: CkmTypeRef;

  /** Description from source documentation. */
  readonly description: string;

  /** Whether the property is required. */
  readonly required: boolean;

  /** Default value. Null means no default. */
  readonly default?: string | null;
}

/**
 * A domain concept extracted from source code (e.g., an interface, struct, or class).
 *
 * Concepts are the primary building blocks of a CKM manifest. Each concept
 * has a slug used for topic derivation and tags for semantic grouping.
 *
 * @public
 */
export interface CkmConcept {
  /** Unique identifier (e.g., "concept-calver-config"). */
  readonly id: string;

  /** Type name (e.g., "CalVerConfig"). */
  readonly name: string;

  /** Topic slug (e.g., "calver") -- used for topic derivation. */
  readonly slug: string;

  /** One-line description. */
  readonly what: string;

  /** Semantic tags (e.g., ["config"]). */
  readonly tags: readonly string[];

  /** Properties of the type, if applicable. */
  readonly properties?: CkmProperty[] | null;
}

/**
 * A function parameter within a {@link CkmOperation}.
 *
 * @public
 */
export interface CkmInput {
  /** Parameter name. */
  readonly name: string;

  /** Type reference. */
  readonly type: CkmTypeRef;

  /** Whether the parameter is required. */
  readonly required: boolean;

  /** Description from source documentation. */
  readonly description: string;
}

/**
 * A return value from a {@link CkmOperation}.
 *
 * @public
 */
export interface CkmOutput {
  /** Type reference. */
  readonly type: CkmTypeRef;

  /** Description of the return value. */
  readonly description: string;
}

/**
 * A user-facing operation extracted from source code (e.g., an exported function).
 *
 * Operations represent callable functionality exposed by the tool.
 * Tags link operations to topics for progressive disclosure.
 *
 * @public
 */
export interface CkmOperation {
  /** Unique identifier (e.g., "op-validate"). */
  readonly id: string;

  /** Function name (e.g., "validate"). */
  readonly name: string;

  /** One-line description. */
  readonly what: string;

  /** Semantic tags for topic linkage (e.g., ["calver", "validation"]). */
  readonly tags: readonly string[];

  /** Function parameters. */
  readonly inputs?: CkmInput[] | null;

  /** Return value. */
  readonly outputs?: CkmOutput | null;
}

/**
 * A rule enforced by the tool.
 *
 * Constraints describe validation rules, invariants, or business logic
 * enforced by the tool at runtime.
 *
 * @public
 */
export interface CkmConstraint {
  /** Unique identifier (e.g., "constraint-future-date"). */
  readonly id: string;

  /** Human-readable rule description. */
  readonly rule: string;

  /** Function or module that enforces the constraint. */
  readonly enforcedBy: string;

  /** Severity level. */
  readonly severity: 'error' | 'warning' | 'info';
}

/**
 * A single step within a {@link CkmWorkflow}.
 *
 * Uses a discriminated union on the `action` field:
 * - `"command"`: a CLI command to execute
 * - `"manual"`: a manual action for the user to perform
 *
 * @public
 */
export interface CkmWorkflowStep {
  /** Discriminant: CLI command or manual action. */
  readonly action: 'command' | 'manual';

  /** The command or instruction. */
  readonly value: string;

  /** Optional explanatory note. */
  readonly note?: string | null;
}

/**
 * A multi-step workflow for achieving a common goal.
 *
 * Workflows group ordered steps (commands and manual actions) that
 * guide users through a process.
 *
 * @public
 */
export interface CkmWorkflow {
  /** Unique identifier. */
  readonly id: string;

  /** What the workflow achieves. */
  readonly goal: string;

  /** Semantic tags. */
  readonly tags: readonly string[];

  /** Ordered steps (minimum 1). */
  readonly steps: readonly CkmWorkflowStep[];
}

/**
 * A configuration schema entry.
 *
 * Describes a single configuration key with its type, description,
 * default value, and whether it is required.
 *
 * @public
 */
export interface CkmConfigEntry {
  /** Dotted key path (e.g., "calver.format"). */
  readonly key: string;

  /** Type reference. */
  readonly type: CkmTypeRef;

  /** Description. */
  readonly description: string;

  /** Default value. Null means no default. */
  readonly default?: string | null;

  /** Whether the config entry is required. */
  readonly required: boolean;
}

/**
 * Provenance metadata about the manifest source.
 *
 * Contains information about the project, language, and generator
 * that produced the manifest.
 *
 * @public
 */
export interface CkmMeta {
  /** Project name (e.g., "my-tool"). */
  readonly project: string;

  /** Source language (e.g., "typescript", "python", "rust"). */
  readonly language: string;

  /** Tool that generated the manifest (e.g., "forge-ts@0.21.1"). */
  readonly generator: string;

  /** ISO 8601 timestamp of generation. */
  readonly generated: string;

  /** Optional URL to source repository. */
  readonly sourceUrl?: string | null;
}

/**
 * The top-level CKM manifest object (v2).
 *
 * This is the universal contract: any generator produces it, any SDK
 * consumes it, any adapter wires it into any CLI framework.
 *
 * @public
 */
export interface CkmManifest {
  /** Schema URL (e.g., "https://ckm.dev/schemas/v2.json"). */
  readonly $schema: string;

  /** Schema version (e.g., "2.0.0"). */
  readonly version: string;

  /** Project metadata and provenance. */
  readonly meta: CkmMeta;

  /** Domain concepts (interfaces, types). */
  readonly concepts: readonly CkmConcept[];

  /** User-facing operations (functions). */
  readonly operations: readonly CkmOperation[];

  /** Enforced rules. */
  readonly constraints: readonly CkmConstraint[];

  /** Multi-step workflows. */
  readonly workflows: readonly CkmWorkflow[];

  /** Configuration schema entries. */
  readonly configSchema: readonly CkmConfigEntry[];
}

// ────────────────────────────────────────────────────────────────────────────
// Section 3: Derived Types (computed by the engine)
// ────────────────────────────────────────────────────────────────────────────

/**
 * An auto-derived topic grouping related concepts, operations, config, and constraints.
 *
 * Topics are computed at engine construction time from manifest data using the
 * algorithm defined in SPEC.md. They provide the progressive disclosure structure
 * for CLI help output.
 *
 * @public
 */
export interface CkmTopic {
  /** Slug used as CLI argument (e.g., "calver"). */
  readonly name: string;

  /** One-line description (from the primary concept). */
  readonly summary: string;

  /** Related concepts. */
  readonly concepts: readonly CkmConcept[];

  /** Related operations. */
  readonly operations: readonly CkmOperation[];

  /** Related config entries. */
  readonly configSchema: readonly CkmConfigEntry[];

  /** Related constraints. */
  readonly constraints: readonly CkmConstraint[];
}

/**
 * A summary entry for the topic index.
 *
 * Provides counts of related items without full content,
 * used in the {@link CkmTopicIndex} for Level 2 disclosure.
 *
 * @public
 */
export interface CkmTopicIndexEntry {
  /** Topic slug. */
  readonly name: string;

  /** One-line description. */
  readonly summary: string;

  /** Count of related concepts. */
  readonly concepts: number;

  /** Count of related operations. */
  readonly operations: number;

  /** Count of related config entries. */
  readonly configFields: number;

  /** Count of related constraints. */
  readonly constraints: number;
}

/**
 * The full topic index returned by `getTopicJson()` with no argument.
 *
 * Contains all topic summaries plus aggregate counts of manifest items.
 * This is the Level 2 (structured) disclosure for agent consumption.
 *
 * @public
 */
export interface CkmTopicIndex {
  /** All topic summaries. */
  readonly topics: readonly CkmTopicIndexEntry[];

  /** Aggregate manifest counts. */
  readonly ckm: {
    /** Total concepts in manifest. */
    readonly concepts: number;

    /** Total operations. */
    readonly operations: number;

    /** Total constraints. */
    readonly constraints: number;

    /** Total workflows. */
    readonly workflows: number;

    /** Total config entries. */
    readonly configSchema: number;
  };
}

/**
 * Manifest statistics returned by `inspect()`.
 *
 * Provides a high-level overview of the manifest contents
 * including metadata, item counts, and derived topic names.
 *
 * @public
 */
export interface CkmInspectResult {
  /** Manifest metadata. */
  readonly meta: CkmMeta;

  /** Counts of each manifest section. */
  readonly counts: {
    readonly concepts: number;
    readonly operations: number;
    readonly constraints: number;
    readonly workflows: number;
    readonly configKeys: number;
    readonly topics: number;
  };

  /** List of derived topic slugs. */
  readonly topicNames: readonly string[];
}

/**
 * A single validation error with a JSON pointer path.
 *
 * @public
 */
export interface CkmValidationError {
  /** JSON pointer path (e.g., "/concepts/0/slug"). */
  readonly path: string;

  /** Human-readable error message. */
  readonly message: string;
}

/**
 * Result of manifest validation.
 *
 * Returned by `validateManifest()`. When `valid` is true, `errors` is empty.
 * When `valid` is false, `errors` contains one or more {@link CkmValidationError} entries.
 *
 * @public
 */
export interface CkmValidationResult {
  /** Whether the manifest is valid. */
  readonly valid: boolean;

  /** Validation errors (empty if valid). */
  readonly errors: readonly CkmValidationError[];
}

/**
 * Error returned when a topic is not found.
 *
 * Includes the list of available topic names so callers can
 * suggest corrections or display alternatives.
 *
 * @public
 */
export interface CkmErrorResult {
  /** Error message (e.g., "Unknown topic: foo"). */
  readonly error: string;

  /** Available topic names for suggestion. */
  readonly topics: readonly string[];
}

// ────────────────────────────────────────────────────────────────────────────
// Section 4: Engine Interface
// ────────────────────────────────────────────────────────────────────────────

/**
 * The core CKM engine interface.
 *
 * Every conformant SDK must implement this interface. The engine consumes
 * a {@link CkmManifest}, derives topics at construction time, and exposes
 * methods for progressive disclosure at four levels:
 *
 * - **Level 0**: `getTopicIndex()` -- topic list (max 300 tokens)
 * - **Level 1**: `getTopicContent(topic)` -- topic detail (max 800 tokens)
 * - **Level 1J**: `getTopicJson(topic)` -- structured topic (max 1200 tokens)
 * - **Level 2**: `getTopicJson()` -- full index (max 3000 tokens)
 *
 * @remarks
 * Instantiate via `createCkmEngine()`. All topic data is derived
 * from the CKM manifest at construction time with zero configuration.
 *
 * @public
 */
export interface CkmEngine {
  /**
   * All auto-derived topics, computed at construction time.
   *
   * This is a read-only array populated by the topic derivation algorithm
   * defined in SPEC.md.
   */
  readonly topics: readonly CkmTopic[];

  /**
   * Returns formatted topic index for terminal display.
   *
   * Includes tool name, usage line, topic list with summaries, and flag descriptions.
   * Output MUST stay within 300 tokens (Level 0 disclosure).
   *
   * @param toolName - Tool name to display in the output header (default: "tool")
   * @returns Formatted topic index string
   */
  getTopicIndex(toolName?: string): string;

  /**
   * Returns human-readable content for a specific topic.
   *
   * Includes: concepts with properties, operations with params, config fields, constraints.
   * Output MUST stay within 800 tokens (Level 1 disclosure).
   *
   * @param topicName - The topic slug to look up
   * @returns Formatted topic content string, or `null` if topic not found
   */
  getTopicContent(topicName: string): string | null;

  /**
   * Returns structured JSON data for a topic or the full index.
   *
   * Behavior depends on the `topicName` argument:
   * - If `topicName` is `undefined`: returns {@link CkmTopicIndex} (max 3000 tokens)
   * - If `topicName` matches a topic: returns the full {@link CkmTopic} (max 1200 tokens)
   * - If `topicName` does not match: returns {@link CkmErrorResult} with available topics
   *
   * @param topicName - Optional topic slug to look up
   * @returns Topic index, topic detail, or error result
   */
  getTopicJson(topicName?: string): CkmTopicIndex | CkmTopic | CkmErrorResult;

  /**
   * Returns the raw manifest (v2, possibly migrated from v1).
   *
   * @returns The complete CKM manifest object
   */
  getManifest(): CkmManifest;

  /**
   * Returns manifest statistics: metadata, counts, and topic names.
   *
   * @returns Inspection result with aggregate manifest data
   */
  inspect(): CkmInspectResult;
}

// ────────────────────────────────────────────────────────────────────────────
// Section 7: Adapter Interface
// ────────────────────────────────────────────────────────────────────────────

/**
 * Custom output formatter for CLI adapters.
 *
 * Allows consumers to override the default plain-text formatting
 * of topic index, topic content, and JSON output.
 *
 * @public
 */
export interface CkmFormatter {
  /**
   * Formats the topic index for terminal display.
   *
   * @param topics - All derived topics
   * @param toolName - Tool name to include in the output
   * @returns Formatted topic index string
   */
  formatIndex(topics: readonly CkmTopic[], toolName: string): string;

  /**
   * Formats a single topic's content for terminal display.
   *
   * @param topic - The topic to format
   * @returns Formatted topic content string
   */
  formatTopic(topic: CkmTopic): string;

  /**
   * Formats JSON output.
   *
   * Default implementation: `JSON.stringify(data, null, 2)`.
   *
   * @param data - The data object to format as JSON
   * @returns Formatted JSON string
   */
  formatJson(data: object): string;
}

/**
 * Options for configuring a CLI adapter.
 *
 * @public
 */
export interface CkmAdapterOptions {
  /** Subcommand name to register (default: "ckm"). */
  readonly commandName?: string;

  /** Tool name in help output (default: inferred from program). */
  readonly toolName?: string;

  /** Custom output formatter (default: built-in plain text). */
  readonly formatter?: CkmFormatter;
}

/**
 * CLI framework adapter interface.
 *
 * Each adapter bridges the CKM engine to a specific CLI framework
 * (e.g., Commander.js, Click, Clap). The generic type parameter `TProgram`
 * represents the host CLI program type.
 *
 * Every adapter MUST support all four progressive disclosure levels:
 * - Level 0: `ckm` (no args) -- topic index
 * - Level 1: `ckm <topic>` -- topic content
 * - Level 1J: `ckm <topic> --json` -- structured topic JSON
 * - Level 2: `ckm --json` -- full index JSON
 *
 * @typeParam TProgram - The host CLI framework's program type
 *
 * @public
 */
export interface CkmCliAdapter<TProgram> {
  /** Adapter identifier (e.g., "commander", "click", "clap"). */
  readonly name: string;

  /** Framework display name (e.g., "Commander.js", "Click", "Clap"). */
  readonly framework: string;

  /**
   * Registers a `ckm [topic]` subcommand on the host program.
   *
   * MUST support all four progressive disclosure levels.
   * MUST handle: no topic (Level 0), topic (Level 1), --json (Level 1J/2).
   *
   * @param program - Host CLI program object
   * @param engine - Configured CKM engine instance
   * @param options - Optional adapter configuration
   */
  register(program: TProgram, engine: CkmEngine, options?: CkmAdapterOptions): void;
}
