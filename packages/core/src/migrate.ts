/**
 * CKM v1 to v2 migration and version detection.
 *
 * @packageDocumentation
 */

import type {
  CanonicalType,
  CkmConcept,
  CkmConfigEntry,
  CkmConstraint,
  CkmInput,
  CkmManifest,
  CkmMeta,
  CkmOperation,
  CkmOutput,
  CkmProperty,
  CkmTypeRef,
  CkmWorkflow,
  CkmWorkflowStep,
} from './types.js';

// ─── v1 Types ────────────────────────────────────────────────────

/** v1 manifest structure (from forge-ts). */
export interface CkmManifestV1 {
  $schema?: string;
  project?: string;
  generated?: string;
  concepts?: CkmConceptV1[];
  operations?: CkmOperationV1[];
  constraints?: CkmConstraintV1[];
  workflows?: CkmWorkflowV1[];
  configSchema?: CkmConfigEntryV1[];
}

interface CkmConceptV1 {
  id: string;
  name: string;
  what: string;
  properties?: { name: string; type: string; description: string }[];
}

interface CkmOperationV1 {
  id: string;
  name: string;
  what: string;
  inputs?: { name: string; type: string; required: boolean; description: string }[];
  outputs?: { text: string };
}

interface CkmConstraintV1 {
  id: string;
  rule: string;
  enforcedBy: string;
}

interface CkmWorkflowV1 {
  id: string;
  goal: string;
  steps: { command?: string; manual?: string; note?: string }[];
}

interface CkmConfigEntryV1 {
  key: string;
  type: string;
  description: string;
  default?: string;
}

// ─── Version Detection ───────────────────────────────────────────

/**
 * Detects the schema version of a parsed manifest.
 *
 * @param data - Parsed JSON object.
 * @returns 1 for v1 manifests, 2 for v2 manifests.
 *
 * @public
 */
export function detectVersion(data: unknown): 1 | 2 {
  if (typeof data !== 'object' || data === null) return 1;
  const obj = data as Record<string, unknown>;

  if (obj.meta && typeof obj.meta === 'object') return 2;
  if (typeof obj.$schema === 'string' && obj.$schema.includes('v2')) return 2;

  return 1;
}

// ─── Migration ───────────────────────────────────────────────────

/**
 * Deterministic migration from v1 format to v2 format.
 *
 * @param v1 - A parsed v1 manifest.
 * @returns A valid v2 CkmManifest.
 *
 * @public
 */
export function migrateV1toV2(v1: CkmManifestV1): CkmManifest {
  const meta: CkmMeta = {
    project: v1.project || 'unknown',
    language: 'typescript',
    generator: 'unknown',
    generated: v1.generated || new Date().toISOString(),
  };

  const concepts = (v1.concepts || []).map(migrateConcept);

  const operations = (v1.operations || []).map((op) => migrateOperation(op, concepts));

  const constraints = (v1.constraints || []).map(migrateConstraint);

  const workflows = (v1.workflows || []).map(migrateWorkflow);

  const configSchema = (v1.configSchema || []).map((entry) => migrateConfigEntry(entry, concepts));

  return {
    $schema: 'https://ckm.dev/schemas/v2.json',
    version: '2.0.0',
    meta,
    concepts,
    operations,
    constraints,
    workflows,
    configSchema,
  };
}

// ─── Helpers ─────────────────────────────────────────────────────

function deriveSlug(name: string): string {
  let slug = name;
  if (slug.endsWith('Config')) slug = slug.slice(0, -6);
  else if (slug.endsWith('Result')) slug = slug.slice(0, -6);
  else if (slug.endsWith('Options')) slug = slug.slice(0, -7);
  return slug.toLowerCase();
}

function inferTags(name: string): string[] {
  const tags: string[] = [];
  if (name.endsWith('Config')) tags.push('config');
  if (name.endsWith('Result')) tags.push('result');
  if (name.endsWith('Options')) tags.push('options');
  return tags;
}

function inferCanonical(typeStr: string): CanonicalType {
  const lower = typeStr.toLowerCase();
  if (lower === 'string') return 'string';
  if (lower === 'boolean') return 'boolean';
  if (lower === 'number') return 'number';
  if (lower === 'integer') return 'integer';
  if (lower === 'null' || lower === 'undefined' || lower === 'void') return 'null';
  if (lower.includes('[]') || lower.startsWith('array')) return 'array';
  if (lower === 'object' || lower === 'record') return 'object';
  if (lower === 'unknown' || lower === 'any') return 'any';
  if (lower.includes('|')) return 'string';
  return 'object';
}

function migrateTypeString(typeStr: string | undefined | null): CkmTypeRef {
  if (!typeStr) return { canonical: 'any' };
  return { canonical: inferCanonical(typeStr), original: typeStr };
}

function migrateConcept(c: CkmConceptV1): CkmConcept {
  return {
    id: c.id,
    name: c.name,
    slug: deriveSlug(c.name),
    what: c.what,
    tags: inferTags(c.name),
    properties: c.properties?.map(migrateProperty),
  };
}

function migrateProperty(p: { name: string; type: string; description: string }): CkmProperty {
  return {
    name: p.name,
    type: migrateTypeString(p.type),
    description: p.description || '',
    required: true,
    default: null,
  };
}

function migrateOperation(op: CkmOperationV1, concepts: CkmConcept[]): CkmOperation {
  return {
    id: op.id,
    name: op.name,
    what: op.what,
    tags: inferOperationTags(op, concepts),
    inputs: op.inputs?.map(migrateInput),
    outputs: migrateOutput(op.outputs),
  };
}

function inferOperationTags(op: { name: string; what: string }, concepts: CkmConcept[]): string[] {
  const tags: string[] = [];
  const haystack = `${op.name} ${op.what}`.toLowerCase();
  for (const concept of concepts) {
    if (concept.slug && haystack.includes(concept.slug)) {
      tags.push(concept.slug);
    }
  }
  return [...new Set(tags)];
}

function migrateInput(i: {
  name: string;
  type: string;
  required: boolean;
  description: string;
}): CkmInput {
  return {
    name: i.name,
    type: migrateTypeString(i.type),
    required: i.required ?? true,
    description: i.description || '',
  };
}

function migrateOutput(output: { text: string } | undefined): CkmOutput | undefined {
  if (!output) return undefined;
  return {
    type: migrateTypeString(output.text || 'unknown'),
    description: output.text || '',
  };
}

function migrateConstraint(c: CkmConstraintV1): CkmConstraint {
  return {
    id: c.id,
    rule: c.rule,
    enforcedBy: c.enforcedBy,
    severity: 'error',
  };
}

function migrateWorkflow(wf: CkmWorkflowV1): CkmWorkflow {
  return {
    id: wf.id,
    goal: wf.goal,
    tags: [],
    steps: wf.steps.map(migrateWorkflowStep),
  };
}

function migrateWorkflowStep(step: {
  command?: string;
  manual?: string;
  note?: string;
}): CkmWorkflowStep {
  if (step.command) {
    return { action: 'command', value: step.command, note: step.note };
  }
  if (step.manual) {
    return { action: 'manual', value: step.manual, note: step.note };
  }
  return { action: 'manual', value: '', note: step.note };
}

function migrateConfigEntry(entry: CkmConfigEntryV1, concepts: CkmConcept[]): CkmConfigEntry {
  return {
    key: migrateConfigKey(entry.key, concepts),
    type: migrateTypeString(entry.type),
    description: entry.description || '',
    default: entry.default ?? null,
    required: true,
  };
}

function migrateConfigKey(key: string, concepts: CkmConcept[]): string {
  const parts = key.split('.');
  if (parts.length >= 2) {
    const conceptPart = parts[0] ?? '';
    const restParts = parts.slice(1);
    for (const concept of concepts) {
      if (concept.name === conceptPart) {
        return [concept.slug, ...restParts].join('.');
      }
    }
    return [conceptPart.toLowerCase(), ...restParts].join('.');
  }
  return key.toLowerCase();
}
