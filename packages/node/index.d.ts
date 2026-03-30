// ─── Schema Types (importable by generators like forge-ts) ───────

/** Canonical type set — maps to JSON Schema primitives. */
export type CanonicalType = 'string' | 'boolean' | 'number' | 'integer' | 'array' | 'object' | 'null' | 'any';

/** Portable type reference. */
export interface CkmTypeRef {
  canonical: CanonicalType;
  original?: string | null;
  enum?: string[] | null;
}

/** Property within a concept. */
export interface CkmProperty {
  name: string;
  type: CkmTypeRef;
  description: string;
  required: boolean;
  default?: string | null;
}

/** Domain concept (interface, type, struct). */
export interface CkmConcept {
  id: string;
  name: string;
  slug: string;
  what: string;
  tags: string[];
  properties?: CkmProperty[] | null;
}

/** Function parameter. */
export interface CkmInput {
  name: string;
  type: CkmTypeRef;
  required: boolean;
  description: string;
}

/** Function return value. */
export interface CkmOutput {
  type: CkmTypeRef;
  description: string;
}

/** User-facing operation. */
export interface CkmOperation {
  id: string;
  name: string;
  what: string;
  tags: string[];
  inputs?: CkmInput[] | null;
  outputs?: CkmOutput | null;
}

/** Enforced constraint. */
export interface CkmConstraint {
  id: string;
  rule: string;
  enforcedBy: string;
  severity: 'error' | 'warning' | 'info';
}

/** Workflow step. */
export interface CkmWorkflowStep {
  action: 'command' | 'manual';
  value: string;
  note?: string | null;
}

/** Multi-step workflow. */
export interface CkmWorkflow {
  id: string;
  goal: string;
  tags: string[];
  steps: CkmWorkflowStep[];
}

/** Configuration entry. */
export interface CkmConfigEntry {
  key: string;
  type: CkmTypeRef;
  description: string;
  default?: string | null;
  required: boolean;
}

/** Manifest metadata. */
export interface CkmMeta {
  project: string;
  language: string;
  generator: string;
  generated: string;
  sourceUrl?: string | null;
}

/** Complete CKM v2 manifest. */
export interface CkmManifest {
  $schema: string;
  version: string;
  meta: CkmMeta;
  concepts: CkmConcept[];
  operations: CkmOperation[];
  constraints: CkmConstraint[];
  workflows: CkmWorkflow[];
  configSchema: CkmConfigEntry[];
}

// ─── Derived Types (computed by engine) ──────────────────────────

export interface CkmTopic {
  name: string;
  summary: string;
  concepts: CkmConcept[];
  operations: CkmOperation[];
  configSchema: CkmConfigEntry[];
  constraints: CkmConstraint[];
}

export interface CkmTopicIndexEntry {
  name: string;
  summary: string;
  concepts: number;
  operations: number;
  configFields: number;
  constraints: number;
}

export interface CkmTopicIndex {
  topics: CkmTopicIndexEntry[];
  ckm: {
    concepts: number;
    operations: number;
    constraints: number;
    workflows: number;
    configSchema: number;
  };
}

export interface CkmInspectResult {
  meta: CkmMeta;
  counts: {
    concepts: number;
    operations: number;
    constraints: number;
    workflows: number;
    configKeys: number;
    topics: number;
  };
  topicNames: string[];
}

export interface CkmValidationError {
  path: string;
  message: string;
}

export interface CkmValidationResult {
  valid: boolean;
  errors: CkmValidationError[];
}

export interface CkmErrorResult {
  error: string;
  topics: string[];
}

// ─── Consumer API (reading manifests) ────────────────────────────

export interface CkmEngine {
  readonly topicsCount: number;
  getTopicIndex(toolName?: string): string;
  getTopicContent(topicName: string): string | null;
  getTopicJson(topicName?: string): CkmTopicIndex | CkmTopic | CkmErrorResult;
  getManifest(): CkmManifest;
  inspect(): CkmInspectResult;
}

/** Creates a CKM engine from a manifest (object or JSON string). */
export function createCkmEngine(manifest: CkmManifest | string): CkmEngine;

/** Validates a manifest against the v2 schema. */
export function validateManifest(data: unknown): CkmValidationResult;

/** Migrates a v1 manifest to v2 format. */
export function migrateV1toV2(data: unknown): CkmManifest;

/** Detects whether a manifest is v1 or v2. */
export function detectVersion(data: unknown): 1 | 2;

// ─── Producer API (building manifests) ───────────────────────────

export interface CkmManifestBuilder {
  generator(name: string): CkmManifestBuilder;
  sourceUrl(url: string): CkmManifestBuilder;
  addConcept(name: string, slug: string, what: string, tags?: string[]): CkmManifestBuilder;
  addConceptProperty(conceptSlug: string, name: string, canonicalType: CanonicalType, description: string, required?: boolean, defaultValue?: string | null): CkmManifestBuilder;
  addOperation(name: string, what: string, tags?: string[]): CkmManifestBuilder;
  addOperationInput(opName: string, paramName: string, canonicalType: CanonicalType, required?: boolean, description?: string): CkmManifestBuilder;
  addConstraint(rule: string, enforcedBy: string, severity?: 'error' | 'warning' | 'info'): CkmManifestBuilder;
  addConfig(key: string, canonicalType: CanonicalType, description: string, required?: boolean, defaultValue?: string | null): CkmManifestBuilder;
  /** Returns the manifest as a JSON string. */
  build(): string;
  /** Returns the manifest as a parsed object. */
  buildJson(): CkmManifest;
}

/** Creates a new manifest builder (producer API for generators). */
export function createManifestBuilder(project: string, language: string): CkmManifestBuilder;
