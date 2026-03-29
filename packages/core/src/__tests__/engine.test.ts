import { readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';
import { describe, expect, it } from 'vitest';
import { createCkmEngine } from '../engine.js';
import { detectVersion, migrateV1toV2 } from '../migrate.js';
import { validateManifest } from '../validate.js';

const fixturesDir = join(import.meta.dirname, '../../../../conformance/fixtures');
const expectedDir = join(import.meta.dirname, '../../../../conformance/expected');

function loadFixture(name: string): unknown {
  return JSON.parse(readFileSync(join(fixturesDir, `${name}.ckm.json`), 'utf-8'));
}

function loadExpected(fixture: string, file: string): unknown {
  const path = join(expectedDir, fixture, file);
  return JSON.parse(readFileSync(path, 'utf-8'));
}

// ─── Engine Construction ─────────────────────────────────────────

describe('createCkmEngine', () => {
  it('constructs from a v2 manifest', () => {
    const data = loadFixture('minimal');
    const engine = createCkmEngine(data);
    expect(engine.topics).toBeDefined();
    expect(engine.topics.length).toBeGreaterThan(0);
  });

  it('constructs from a v1 manifest (auto-migrates)', () => {
    const data = loadFixture('v1-legacy');
    const engine = createCkmEngine(data);
    expect(engine.topics).toBeDefined();
    // After migration, should have config-tagged topics
    expect(engine.topics.length).toBeGreaterThan(0);
  });

  it('handles edge case with empty arrays', () => {
    const data = loadFixture('edge-cases');
    const engine = createCkmEngine(data);
    expect(engine.topics).toHaveLength(0);
  });
});

// ─── Version Detection ───────────────────────────────────────────

describe('detectVersion', () => {
  it('detects v2 manifests (has meta block)', () => {
    const data = loadFixture('minimal');
    expect(detectVersion(data)).toBe(2);
  });

  it('detects v1 manifests (no meta block)', () => {
    const data = loadFixture('v1-legacy');
    expect(detectVersion(data)).toBe(1);
  });

  it('returns 1 for non-objects', () => {
    expect(detectVersion(null)).toBe(1);
    expect(detectVersion('string')).toBe(1);
    expect(detectVersion(42)).toBe(1);
  });
});

// ─── Migration ───────────────────────────────────────────────────

describe('migrateV1toV2', () => {
  it('migrates v1 to v2 format', () => {
    const v1 = loadFixture('v1-legacy') as Record<string, unknown>;
    const v2 = migrateV1toV2(v1 as any);
    expect(v2.meta).toBeDefined();
    expect(v2.meta.language).toBe('typescript');
    expect(v2.version).toBe('2.0.0');
    expect(v2.$schema).toBe('https://ckm.dev/schemas/v2.json');
    // Concepts should have slugs and tags
    for (const concept of v2.concepts) {
      expect(concept.slug).toBeDefined();
      expect(concept.tags).toBeDefined();
    }
  });
});

// ─── Validation ──────────────────────────────────────────────────

describe('validateManifest', () => {
  it('validates a valid v2 manifest', () => {
    const data = loadFixture('minimal');
    const result = validateManifest(data);
    expect(result.valid).toBe(true);
    expect(result.errors).toHaveLength(0);
  });

  it('rejects a v1 manifest', () => {
    const data = loadFixture('v1-legacy');
    const result = validateManifest(data);
    expect(result.valid).toBe(false);
    expect(result.errors.length).toBeGreaterThan(0);
  });

  it('rejects non-objects', () => {
    const result = validateManifest('not an object');
    expect(result.valid).toBe(false);
  });

  it('validates multi-topic manifest', () => {
    const data = loadFixture('multi-topic');
    const result = validateManifest(data);
    expect(result.valid).toBe(true);
  });

  it('validates edge-cases manifest', () => {
    const data = loadFixture('edge-cases');
    const result = validateManifest(data);
    expect(result.valid).toBe(true);
  });
});

// ─── Topic Derivation ────────────────────────────────────────────

describe('topic derivation', () => {
  it('derives topics from minimal fixture', () => {
    const engine = createCkmEngine(loadFixture('minimal'));
    expect(engine.topics.length).toBe(1);
    expect(engine.topics[0]?.name).toBe('calver');
  });

  it('derives multiple topics from multi-topic fixture', () => {
    const engine = createCkmEngine(loadFixture('multi-topic'));
    expect(engine.topics.length).toBeGreaterThanOrEqual(3);
    const names = engine.topics.map((t) => t.name);
    expect(names).toContain('calver');
    expect(names).toContain('semver');
    expect(names).toContain('git');
  });

  it('handles polyglot fixture', () => {
    const engine = createCkmEngine(loadFixture('polyglot'));
    expect(engine.topics.length).toBeGreaterThan(0);
  });
});

// ─── Topic Queries ───────────────────────────────────────────────

describe('getTopicIndex', () => {
  it('returns formatted string', () => {
    const engine = createCkmEngine(loadFixture('minimal'));
    const index = engine.getTopicIndex('mytool');
    expect(index).toContain('mytool CKM');
    expect(index).toContain('calver');
    expect(index).toContain('--json');
  });

  it('stays within 300 token budget (~1200 chars)', () => {
    const engine = createCkmEngine(loadFixture('multi-topic'));
    const index = engine.getTopicIndex('tool');
    expect(index.length).toBeLessThan(1200);
  });
});

describe('getTopicContent', () => {
  it('returns content for a valid topic', () => {
    const engine = createCkmEngine(loadFixture('minimal'));
    const content = engine.getTopicContent('calver');
    expect(content).not.toBeNull();
    expect(content).toContain('CalVerConfig');
    expect(content).toContain('## Concepts');
  });

  it('returns null for unknown topic', () => {
    const engine = createCkmEngine(loadFixture('minimal'));
    expect(engine.getTopicContent('nonexistent')).toBeNull();
  });

  it('stays within 800 token budget (~3200 chars)', () => {
    const engine = createCkmEngine(loadFixture('minimal'));
    const content = engine.getTopicContent('calver');
    expect(content?.length).toBeLessThan(3200);
  });
});

describe('getTopicJson', () => {
  it('returns topic index when no arg', () => {
    const engine = createCkmEngine(loadFixture('minimal'));
    const result = engine.getTopicJson() as any;
    expect(result.topics).toBeDefined();
    expect(result.ckm).toBeDefined();
    expect(result.ckm.concepts).toBeGreaterThan(0);
  });

  it('returns topic data for valid topic', () => {
    const engine = createCkmEngine(loadFixture('minimal'));
    const result = engine.getTopicJson('calver') as any;
    expect(result.name).toBe('calver');
    expect(result.concepts).toBeDefined();
  });

  it('returns error for unknown topic', () => {
    const engine = createCkmEngine(loadFixture('minimal'));
    const result = engine.getTopicJson('nonexistent') as any;
    expect(result.error).toBeDefined();
    expect(result.topics).toBeDefined();
  });
});

// ─── Inspect ─────────────────────────────────────────────────────

describe('inspect', () => {
  it('returns inspection result', () => {
    const engine = createCkmEngine(loadFixture('minimal'));
    const result = engine.inspect();
    expect(result.meta).toBeDefined();
    expect(result.meta.project).toBeDefined();
    expect(result.counts.concepts).toBeGreaterThan(0);
    expect(result.counts.topics).toBeGreaterThan(0);
    expect(result.topicNames).toContain('calver');
  });
});

// ─── Get Manifest ────────────────────────────────────────────────

describe('getManifest', () => {
  it('returns v2 manifest', () => {
    const engine = createCkmEngine(loadFixture('minimal'));
    const manifest = engine.getManifest();
    expect(manifest.meta).toBeDefined();
    expect(manifest.concepts.length).toBeGreaterThan(0);
  });

  it('returns migrated manifest for v1 input', () => {
    const engine = createCkmEngine(loadFixture('v1-legacy'));
    const manifest = engine.getManifest();
    expect(manifest.meta).toBeDefined();
    expect(manifest.version).toBe('2.0.0');
  });
});

// ─── Conformance: All Fixtures ───────────────────────────────────

describe('conformance suite', () => {
  const fixtures = readdirSync(fixturesDir)
    .filter((f) => f.endsWith('.ckm.json'))
    .map((f) => f.replace('.ckm.json', ''));

  for (const fixture of fixtures) {
    describe(`fixture: ${fixture}`, () => {
      it('engine constructs without error', () => {
        const data = loadFixture(fixture);
        const engine = createCkmEngine(data);
        expect(engine).toBeDefined();
      });

      it('getTopicIndex returns string', () => {
        const engine = createCkmEngine(loadFixture(fixture));
        const index = engine.getTopicIndex('test');
        expect(typeof index).toBe('string');
      });

      it('getTopicJson returns object', () => {
        const engine = createCkmEngine(loadFixture(fixture));
        const json = engine.getTopicJson();
        expect(json).toBeDefined();
        expect(typeof json).toBe('object');
      });

      it('inspect returns valid result', () => {
        const engine = createCkmEngine(loadFixture(fixture));
        const result = engine.inspect();
        expect(result.meta).toBeDefined();
        expect(result.counts).toBeDefined();
        expect(result.topicNames).toBeDefined();
      });
    });
  }
});
