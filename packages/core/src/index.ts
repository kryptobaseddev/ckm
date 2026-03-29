/**
 * CKM (Codebase Knowledge Manifest) SDK.
 *
 * @remarks
 * Reusable SDK for consuming `ckm.json` manifests. Provides
 * auto-derived topics, machine-readable JSON output, and
 * human-readable terminal formatting — all with zero manual mapping.
 *
 * @example
 * ```ts
 * import { createCkmEngine } from 'ckm';
 *
 * const engine = createCkmEngine(JSON.parse(ckmJsonString));
 * console.log(engine.getTopicIndex('mytool'));
 * console.log(engine.getTopicContent('calver'));
 * console.log(JSON.stringify(engine.getTopicJson('calver'), null, 2));
 * ```
 *
 * @packageDocumentation
 * @public
 */

// Core engine
export { createCkmEngine } from './engine.js';

// Migration and version detection
export { detectVersion, migrateV1toV2 } from './migrate.js';
export type { CkmManifestV1 } from './migrate.js';

// Validation
export { validateManifest } from './validate.js';

// Formatting utilities
export { formatTopicContent, formatTopicIndex } from './format.js';

// All public types
export type {
  CanonicalType,
  CkmConcept,
  CkmConfigEntry,
  CkmConstraint,
  CkmEngine,
  CkmErrorResult,
  CkmInput,
  CkmInspectResult,
  CkmManifest,
  CkmMeta,
  CkmOperation,
  CkmOutput,
  CkmProperty,
  CkmTopic,
  CkmTopicIndex,
  CkmTopicIndexEntry,
  CkmTypeRef,
  CkmValidationError,
  CkmValidationResult,
  CkmWorkflow,
  CkmWorkflowStep,
} from './types.js';

// Adapter types
export type {
  CkmAdapterOptions,
  CkmCliAdapter,
  CkmFormatter,
} from './adapters/types.js';

// Adapter registry
export { ADAPTER_TABLE, loadAdapter } from './adapters/registry.js';
