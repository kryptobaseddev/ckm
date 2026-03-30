// CKM SDK — TypeScript Type Definitions
// These types are the compile-time contract between producers and consumers.
// Import: import type { CkmManifest } from 'ckm-sdk'

export type Extensions = Record<string, unknown>;
export type CanonicalType = 'string' | 'boolean' | 'number' | 'integer' | 'array' | 'object' | 'null' | 'any';

export interface CkmTypeRef { canonical: CanonicalType; original?: string | null; enum?: string[] | null; }
export interface CkmProperty { name: string; type: CkmTypeRef; description: string; required: boolean; default?: string | null; }
export interface CkmConcept { id: string; name: string; slug: string; what: string; tags: string[]; properties?: CkmProperty[] | null; rules?: string[] | null; relatedTo?: string[] | null; extensions?: Extensions; }
export interface CkmInput { name: string; type: CkmTypeRef; required: boolean; description: string; }
export interface CkmOutput { type: CkmTypeRef; description: string; }
export interface CkmOperation { id: string; name: string; what: string; tags: string[]; preconditions?: string[] | null; inputs?: CkmInput[] | null; outputs?: CkmOutput | null; exitCodes?: Record<string, string> | null; checksPerformed?: string[] | null; extensions?: Extensions; }
export interface CkmConstraint { id: string; rule: string; enforcedBy: string; severity: 'error' | 'warning' | 'info'; configKey?: string | null; default?: string | null; security?: boolean | null; extensions?: Extensions; }
export interface CkmWorkflowStep { action: 'command' | 'manual'; value: string; expect?: string | null; note?: string | null; }
export interface CkmWorkflow { id: string; goal: string; tags: string[]; steps: CkmWorkflowStep[]; extensions?: Extensions; }
export interface CkmConfigEntry { key: string; type: CkmTypeRef; description: string; default?: string | null; required: boolean; effect?: string | null; extensions?: Extensions; }
export interface CkmDeclaredTopic { name: string; summary: string; conceptIds?: string[]; operationIds?: string[]; constraintIds?: string[]; configKeys?: string[]; }
export interface CkmMeta { project: string; language: string; generator: string; generated: string; sourceUrl?: string | null; }
export interface CkmManifest { $schema: string; version: string; meta: CkmMeta; concepts: CkmConcept[]; operations: CkmOperation[]; constraints: CkmConstraint[]; workflows: CkmWorkflow[]; configSchema: CkmConfigEntry[]; topics?: CkmDeclaredTopic[] | null; extensions?: Extensions; }

export interface CkmTopic { name: string; summary: string; concepts: CkmConcept[]; operations: CkmOperation[]; configSchema: CkmConfigEntry[]; constraints: CkmConstraint[]; }
export interface CkmTopicIndexEntry { name: string; summary: string; concepts: number; operations: number; configFields: number; constraints: number; }
export interface CkmTopicIndex { topics: CkmTopicIndexEntry[]; ckm: { concepts: number; operations: number; constraints: number; workflows: number; configSchema: number }; }
export interface CkmInspectResult { meta: CkmMeta; counts: { concepts: number; operations: number; constraints: number; workflows: number; configKeys: number; topics: number }; topicNames: string[]; }
export interface CkmValidationResult { valid: boolean; errors: Array<{ path: string; message: string }>; }
export interface CkmErrorResult { error: string; topics: string[]; }

// Consumer API
export interface CkmEngine { readonly topicsCount: number; getTopicIndex(toolName?: string): string; getTopicContent(topicName: string): string | null; getTopicJson(topicName?: string): CkmTopicIndex | CkmTopic | CkmErrorResult; getManifest(): CkmManifest; inspect(): CkmInspectResult; }
export function createCkmEngine(manifest: CkmManifest | string): CkmEngine;
export function validateManifest(data: unknown): CkmValidationResult;
export function migrateV1toV2(data: unknown): CkmManifest;
export function detectVersion(data: unknown): 1 | 2;

// Producer API
export interface CkmManifestBuilder { generator(name: string): CkmManifestBuilder; sourceUrl(url: string): CkmManifestBuilder; addConcept(name: string, slug: string, what: string, tags?: string[]): CkmManifestBuilder; addConceptProperty(slug: string, name: string, type: CanonicalType, desc: string, required?: boolean, defaultVal?: string | null): CkmManifestBuilder; addOperation(name: string, what: string, tags?: string[]): CkmManifestBuilder; addOperationInput(opName: string, param: string, type: CanonicalType, required?: boolean, desc?: string): CkmManifestBuilder; addConstraint(rule: string, enforcedBy: string, severity?: 'error' | 'warning' | 'info'): CkmManifestBuilder; addConfig(key: string, type: CanonicalType, desc: string, required?: boolean, defaultVal?: string | null): CkmManifestBuilder; build(): string; buildJson(): CkmManifest; }
export function createManifestBuilder(project: string, language: string): CkmManifestBuilder;
