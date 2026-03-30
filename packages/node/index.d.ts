// ─── CKM Schema Types (the compile-time contract) ───────────────
// Import these in your generator: import type { CkmManifest } from 'ckm-sdk'

/** Freeform extension data. Producers can attach arbitrary key-value pairs. */
export type Extensions = Record<string, unknown>;

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
  rules?: string[] | null;
  relatedTo?: string[] | null;
  extensions?: Extensions;
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
  preconditions?: string[] | null;
  inputs?: CkmInput[] | null;
  outputs?: CkmOutput | null;
  exitCodes?: Record<string, string> | null;
  checksPerformed?: string[] | null;
  extensions?: Extensions;
}

/** Enforced constraint. */
export interface CkmConstraint {
  id: string;
  rule: string;
  enforcedBy: string;
  severity: 'error' | 'warning' | 'info';
  configKey?: string | null;
  default?: string | null;
  security?: boolean | null;
  extensions?: Extensions;
}

/** Workflow step. */
export interface CkmWorkflowStep {
  action: 'command' | 'manual';
  value: string;
  expect?: string | null;
  note?: string | null;
}

/** Multi-step workflow. */
export interface CkmWorkflow {
  id: string;
  goal: string;
  tags: string[];
  steps: CkmWorkflowStep[];
  extensions?: Extensions;
}

/** Configuration entry. */
export interface CkmConfigEntry {
  key: string;
  type: CkmTypeRef;
  description: string;
  default?: string | null;
  required: boolean;
  effect?: string | null;
  extensions?: Extensions;
}

/** Producer-declared topic. Overrides engine-derived topics when present. */
export interface CkmDeclaredTopic {
  name: string;
  summary: string;
  conceptIds?: string[];
  operationIds?: string[];
  constraintIds?: string[];
  configKeys?: string[];
}

/** Manifest metadata. */
export interface CkmMeta {
  project: string;
  language: string;
  generator: string;
  generated: string;
  sourceUrl?: string | null;
}

/** Complete CKM v2 manifest — the universal contract. */
export interface CkmManifest {
  $schema: string;
  version: string;
  meta: CkmMeta;
  concepts: CkmConcept[];
  operations: CkmOperation[];
  constraints: CkmConstraint[];
  workflows: CkmWorkflow[];
  configSchema: CkmConfigEntry[];
  topics?: CkmDeclaredTopic[] | null;
  extensions?: Extensions;
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
  ckm: { concepts: number; operations: number; constraints: number; workflows: number; configSchema: number };
}

export interface CkmInspectResult {
  meta: CkmMeta;
  counts: { concepts: number; operations: number; constraints: number; workflows: number; configKeys: number; topics: number };
  topicNames: string[];
}

export interface CkmValidationResult {
  valid: boolean;
  errors: Array<{ path: string; message: string }>;
}

export interface CkmErrorResult {
  error: string;
  topics: string[];
}

// ─── Consumer API ────────────────────────────────────────────────

export interface CkmEngine {
  readonly topicsCount: number;
  getTopicIndex(toolName?: string): string;
  getTopicContent(topicName: string): string | null;
  getTopicJson(topicName?: string): CkmTopicIndex | CkmTopic | CkmErrorResult;
  getManifest(): CkmManifest;
  inspect(): CkmInspectResult;
}

export function createCkmEngine(manifest: CkmManifest | string): CkmEngine;
export function validateManifest(data: unknown): CkmValidationResult;
export function migrateV1toV2(data: unknown): CkmManifest;
export function detectVersion(data: unknown): 1 | 2;

// ─── Producer API ────────────────────────────────────────────────

export interface CkmManifestBuilder {
  generator(name: string): CkmManifestBuilder;
  sourceUrl(url: string): CkmManifestBuilder;
  addConcept(name: string, slug: string, what: string, tags?: string[]): CkmManifestBuilder;
  addConceptProperty(slug: string, name: string, type: CanonicalType, desc: string, required?: boolean, defaultVal?: string | null): CkmManifestBuilder;
  addOperation(name: string, what: string, tags?: string[]): CkmManifestBuilder;
  addOperationInput(opName: string, param: string, type: CanonicalType, required?: boolean, desc?: string): CkmManifestBuilder;
  addConstraint(rule: string, enforcedBy: string, severity?: 'error' | 'warning' | 'info'): CkmManifestBuilder;
  addConfig(key: string, type: CanonicalType, desc: string, required?: boolean, defaultVal?: string | null): CkmManifestBuilder;
  build(): string;
  buildJson(): CkmManifest;
}

export function createManifestBuilder(project: string, language: string): CkmManifestBuilder;
