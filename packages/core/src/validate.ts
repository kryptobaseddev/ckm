/**
 * CKM manifest validation against the v2 JSON Schema.
 *
 * @remarks
 * Provides lightweight structural validation without external
 * JSON Schema library dependencies. Validates required fields,
 * types, and enum values as defined in ckm.schema.json.
 *
 * @packageDocumentation
 */

import type { CkmValidationError, CkmValidationResult } from './types.js';

/**
 * Validates a parsed JSON object against the ckm.json v2 schema.
 *
 * @param data - Parsed JSON value.
 * @returns Validation result with JSON pointer paths for errors.
 *
 * @public
 */
export function validateManifest(data: unknown): CkmValidationResult {
  const errors: CkmValidationError[] = [];

  if (typeof data !== 'object' || data === null || Array.isArray(data)) {
    errors.push({ path: '', message: 'Manifest must be an object' });
    return { valid: false, errors };
  }

  const obj = data as Record<string, unknown>;

  // Required top-level fields
  validateRequiredString(obj, 'version', '', errors);
  validateRequiredObject(obj, 'meta', '', errors);
  validateRequiredArray(obj, 'concepts', '', errors);
  validateRequiredArray(obj, 'operations', '', errors);
  validateRequiredArray(obj, 'constraints', '', errors);
  validateRequiredArray(obj, 'workflows', '', errors);
  validateRequiredArray(obj, 'configSchema', '', errors);

  // Meta validation
  if (typeof obj.meta === 'object' && obj.meta !== null) {
    const meta = obj.meta as Record<string, unknown>;
    validateRequiredString(meta, 'project', '/meta', errors);
    validateRequiredString(meta, 'language', '/meta', errors);
    validateRequiredString(meta, 'generator', '/meta', errors);
    validateRequiredString(meta, 'generated', '/meta', errors);
  }

  // Concepts validation
  if (Array.isArray(obj.concepts)) {
    for (let i = 0; i < obj.concepts.length; i++) {
      const path = `/concepts/${i}`;
      const concept = obj.concepts[i] as Record<string, unknown>;
      if (typeof concept !== 'object' || concept === null) {
        errors.push({ path, message: 'Concept must be an object' });
        continue;
      }
      validateRequiredString(concept, 'id', path, errors);
      validateRequiredString(concept, 'name', path, errors);
      validateRequiredString(concept, 'slug', path, errors);
      validateRequiredString(concept, 'what', path, errors);
      validateRequiredArray(concept, 'tags', path, errors);

      if (Array.isArray(concept.properties)) {
        for (let j = 0; j < concept.properties.length; j++) {
          validateProperty(concept.properties[j], `${path}/properties/${j}`, errors);
        }
      }
    }
  }

  // Operations validation
  if (Array.isArray(obj.operations)) {
    for (let i = 0; i < obj.operations.length; i++) {
      const path = `/operations/${i}`;
      const op = obj.operations[i] as Record<string, unknown>;
      if (typeof op !== 'object' || op === null) {
        errors.push({ path, message: 'Operation must be an object' });
        continue;
      }
      validateRequiredString(op, 'id', path, errors);
      validateRequiredString(op, 'name', path, errors);
      validateRequiredString(op, 'what', path, errors);
      validateRequiredArray(op, 'tags', path, errors);
    }
  }

  // Constraints validation
  if (Array.isArray(obj.constraints)) {
    for (let i = 0; i < obj.constraints.length; i++) {
      const path = `/constraints/${i}`;
      const c = obj.constraints[i] as Record<string, unknown>;
      if (typeof c !== 'object' || c === null) {
        errors.push({ path, message: 'Constraint must be an object' });
        continue;
      }
      validateRequiredString(c, 'id', path, errors);
      validateRequiredString(c, 'rule', path, errors);
      validateRequiredString(c, 'enforcedBy', path, errors);
      if (typeof c.severity === 'string') {
        if (!['error', 'warning', 'info'].includes(c.severity)) {
          errors.push({
            path: `${path}/severity`,
            message: `Invalid severity: "${c.severity}". Must be "error", "warning", or "info"`,
          });
        }
      } else {
        errors.push({ path: `${path}/severity`, message: 'Missing required field: severity' });
      }
    }
  }

  // Workflows validation
  if (Array.isArray(obj.workflows)) {
    for (let i = 0; i < obj.workflows.length; i++) {
      const path = `/workflows/${i}`;
      const wf = obj.workflows[i] as Record<string, unknown>;
      if (typeof wf !== 'object' || wf === null) {
        errors.push({ path, message: 'Workflow must be an object' });
        continue;
      }
      validateRequiredString(wf, 'id', path, errors);
      validateRequiredString(wf, 'goal', path, errors);
      validateRequiredArray(wf, 'tags', path, errors);
      if (!Array.isArray(wf.steps) || wf.steps.length === 0) {
        errors.push({ path: `${path}/steps`, message: 'Workflow must have at least one step' });
      }
    }
  }

  // ConfigSchema validation
  if (Array.isArray(obj.configSchema)) {
    for (let i = 0; i < obj.configSchema.length; i++) {
      const path = `/configSchema/${i}`;
      const entry = obj.configSchema[i] as Record<string, unknown>;
      if (typeof entry !== 'object' || entry === null) {
        errors.push({ path, message: 'Config entry must be an object' });
        continue;
      }
      validateRequiredString(entry, 'key', path, errors);
      validateRequiredString(entry, 'description', path, errors);
      if (typeof entry.required !== 'boolean') {
        errors.push({ path: `${path}/required`, message: 'Missing required field: required' });
      }
      validateTypeRef(entry.type, `${path}/type`, errors);
    }
  }

  return { valid: errors.length === 0, errors };
}

// ─── Helpers ─────────────────────────────────────────────────────

const CANONICAL_TYPES = [
  'string',
  'boolean',
  'number',
  'integer',
  'array',
  'object',
  'null',
  'any',
];

function validateRequiredString(
  obj: Record<string, unknown>,
  field: string,
  parentPath: string,
  errors: CkmValidationError[],
): void {
  if (typeof obj[field] !== 'string') {
    errors.push({ path: `${parentPath}/${field}`, message: `Missing required field: ${field}` });
  }
}

function validateRequiredObject(
  obj: Record<string, unknown>,
  field: string,
  parentPath: string,
  errors: CkmValidationError[],
): void {
  if (typeof obj[field] !== 'object' || obj[field] === null) {
    errors.push({ path: `${parentPath}/${field}`, message: `Missing required field: ${field}` });
  }
}

function validateRequiredArray(
  obj: Record<string, unknown>,
  field: string,
  parentPath: string,
  errors: CkmValidationError[],
): void {
  if (!Array.isArray(obj[field])) {
    errors.push({ path: `${parentPath}/${field}`, message: `Missing required field: ${field}` });
  }
}

function validateTypeRef(typeRef: unknown, path: string, errors: CkmValidationError[]): void {
  if (typeof typeRef !== 'object' || typeRef === null) {
    errors.push({ path, message: 'Type reference must be an object' });
    return;
  }
  const ref = typeRef as Record<string, unknown>;
  if (typeof ref.canonical !== 'string' || !CANONICAL_TYPES.includes(ref.canonical)) {
    errors.push({
      path: `${path}/canonical`,
      message: `Invalid canonical type. Must be one of: ${CANONICAL_TYPES.join(', ')}`,
    });
  }
}

function validateProperty(prop: unknown, path: string, errors: CkmValidationError[]): void {
  if (typeof prop !== 'object' || prop === null) {
    errors.push({ path, message: 'Property must be an object' });
    return;
  }
  const p = prop as Record<string, unknown>;
  validateRequiredString(p, 'name', path, errors);
  validateRequiredString(p, 'description', path, errors);
  if (typeof p.required !== 'boolean') {
    errors.push({ path: `${path}/required`, message: 'Missing required field: required' });
  }
  validateTypeRef(p.type, `${path}/type`, errors);
}
