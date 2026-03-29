/**
 * CKM engine — auto-generates topic index from a ckm.json manifest.
 *
 * @remarks
 * This module implements the SPEC.md algorithm for topic derivation,
 * JSON output, and terminal formatting. It handles both v1 and v2
 * manifests transparently.
 *
 * @packageDocumentation
 */

import { formatTopicContent, formatTopicIndex } from './format.js';
import { detectVersion, migrateV1toV2 } from './migrate.js';
import type {
  CkmConcept,
  CkmEngine,
  CkmErrorResult,
  CkmInspectResult,
  CkmManifest,
  CkmTopic,
  CkmTopicIndex,
  CkmTopicIndexEntry,
} from './types.js';

/**
 * Creates a CKM engine from a parsed manifest.
 *
 * @remarks
 * Main entry point. Accepts a parsed JSON object (v1 or v2).
 * If v1 is detected, auto-migrates to v2 internally.
 * Derives topics at construction time.
 * Returns a configured, immutable engine instance.
 *
 * @param data - Parsed CKM manifest (v1 or v2).
 * @returns A configured CKM engine.
 *
 * @public
 */
export function createCkmEngine(data: unknown): CkmEngine {
  const version = detectVersion(data);
  const manifest: CkmManifest =
    version === 1 ? migrateV1toV2(data as Record<string, unknown>) : (data as CkmManifest);

  const topics = deriveTopics(manifest);

  return {
    topics,
    getTopicIndex: (toolName = 'tool') => formatTopicIndex(topics, toolName),
    getTopicContent: (name) => formatTopicContent(topics, name),
    getTopicJson: (name) => {
      if (name === undefined || name === null) {
        return buildTopicIndexJson(topics, manifest);
      }
      return buildTopicJson(topics, name);
    },
    getManifest: () => manifest,
    inspect: () => buildInspect(manifest, topics),
  };
}

// ─── Topic Derivation (SPEC.md Section 3) ────────────────────────

function deriveTopics(manifest: CkmManifest): CkmTopic[] {
  const topics: CkmTopic[] = [];

  for (const concept of manifest.concepts) {
    // Step 1: Only concepts tagged "config" become topics
    if (!concept.tags.includes('config')) continue;

    const slug = concept.slug;
    if (!slug) continue;

    // Step 2: Collect related concepts
    const relatedConcepts: CkmConcept[] = [concept];
    for (const other of manifest.concepts) {
      if (other.id === concept.id) continue;
      const otherSlug = deriveSlugFromName(other.name);
      if (other.name.toLowerCase().includes(slug) || slug.includes(otherSlug)) {
        relatedConcepts.push(other);
      }
    }
    const conceptNames = relatedConcepts.map((c) => c.name);

    // Step 3: Match operations by tags or keywords
    const matchedOperations = manifest.operations.filter((op) => {
      if (hasTagOverlap(op.tags, [slug])) return true;
      return operationMatchesByKeyword(op, slug, conceptNames);
    });

    // Step 4: Match config entries by key prefix
    const matchedConfig = manifest.configSchema.filter((entry) => {
      const keyPrefix = entry.key.split('.')[0];
      return keyPrefix === slug;
    });

    // Step 5: Match constraints
    const matchedConstraints = manifest.constraints.filter((constraint) => {
      if (conceptNames.some((name) => constraint.enforcedBy.includes(name))) return true;
      return matchedOperations.some((op) => constraint.enforcedBy.includes(op.name));
    });

    // Step 6: Build topic
    topics.push({
      name: slug,
      summary: concept.what,
      concepts: relatedConcepts,
      operations: matchedOperations,
      configSchema: matchedConfig,
      constraints: matchedConstraints,
    });
  }

  return topics;
}

function deriveSlugFromName(name: string): string {
  let slug = name;
  if (slug.endsWith('Config')) slug = slug.slice(0, -6);
  else if (slug.endsWith('Result')) slug = slug.slice(0, -6);
  else if (slug.endsWith('Options')) slug = slug.slice(0, -7);
  return slug.toLowerCase();
}

function hasTagOverlap(tags1: readonly string[], tags2: readonly string[]): boolean {
  for (const t1 of tags1) {
    for (const t2 of tags2) {
      if (t1.toLowerCase() === t2.toLowerCase()) return true;
    }
  }
  return false;
}

function operationMatchesByKeyword(
  op: { name: string; what: string },
  slug: string,
  conceptNames: string[],
): boolean {
  const haystack = `${op.name} ${op.what}`.toLowerCase();
  if (haystack.includes(slug)) return true;
  return conceptNames.some((name) => haystack.includes(name.toLowerCase()));
}

// ─── JSON Output (SPEC.md Section 4.3, 4.4) ─────────────────────

function buildTopicIndexJson(topics: CkmTopic[], manifest: CkmManifest): CkmTopicIndex {
  return {
    topics: topics.map(
      (t): CkmTopicIndexEntry => ({
        name: t.name,
        summary: t.summary,
        concepts: t.concepts.length,
        operations: t.operations.length,
        configFields: t.configSchema.length,
        constraints: t.constraints.length,
      }),
    ),
    ckm: {
      concepts: manifest.concepts.length,
      operations: manifest.operations.length,
      constraints: manifest.constraints.length,
      workflows: manifest.workflows.length,
      configSchema: manifest.configSchema.length,
    },
  };
}

function buildTopicJson(topics: CkmTopic[], topicName: string): CkmTopic | CkmErrorResult {
  const topic = topics.find((t) => t.name === topicName);
  if (!topic) {
    return {
      error: `Unknown topic: ${topicName}`,
      topics: topics.map((t) => t.name),
    };
  }
  return topic;
}

// ─── Inspection (SPEC.md Section 5) ──────────────────────────────

function buildInspect(manifest: CkmManifest, topics: CkmTopic[]): CkmInspectResult {
  return {
    meta: manifest.meta,
    counts: {
      concepts: manifest.concepts.length,
      operations: manifest.operations.length,
      constraints: manifest.constraints.length,
      workflows: manifest.workflows.length,
      configKeys: manifest.configSchema.length,
      topics: topics.length,
    },
    topicNames: topics.map((t) => t.name),
  };
}
