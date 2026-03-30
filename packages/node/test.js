#!/usr/bin/env node
/**
 * CKM SDK Node.js tests.
 * Run: node test.js
 */
const fs = require('fs');
const path = require('path');
const ckm = require('./index.js');

const fixturesDir = path.join(__dirname, '../../conformance/fixtures');
let passed = 0;
let failed = 0;

function assert(condition, msg) {
  if (condition) { passed++; }
  else { failed++; console.error(`FAIL: ${msg}`); }
}

// ─── Consumer API ─────────────────────────────────────
const fixtures = fs.readdirSync(fixturesDir).filter(f => f.endsWith('.ckm.json'));

for (const f of fixtures) {
  const name = f.replace('.ckm.json', '');
  const data = fs.readFileSync(path.join(fixturesDir, f), 'utf-8');

  // Engine constructs
  const engine = ckm.createCkmEngine(data);
  assert(engine, `${name}: engine creates`);
  assert(typeof engine.topicsCount === 'number', `${name}: topicsCount is number`);

  // getTopicIndex returns string
  const index = engine.getTopicIndex('test');
  assert(typeof index === 'string' && index.includes('CKM'), `${name}: getTopicIndex`);

  // getTopicJson returns object
  const json = engine.getTopicJson();
  assert(json && json.topics && json.ckm, `${name}: getTopicJson index`);

  // inspect returns object
  const insp = engine.inspect();
  assert(insp && insp.meta && insp.counts, `${name}: inspect`);

  // getManifest returns object
  const manifest = engine.getManifest();
  assert(manifest && manifest.meta, `${name}: getManifest`);

  // Per-topic queries
  for (const t of json.topics) {
    const content = engine.getTopicContent(t.name);
    assert(typeof content === 'string', `${name}/${t.name}: getTopicContent`);

    const topicJson = engine.getTopicJson(t.name);
    assert(topicJson && topicJson.name === t.name, `${name}/${t.name}: getTopicJson`);
  }

  // Unknown topic returns error
  const err = engine.getTopicJson('__nonexistent__');
  assert(err && err.error, `${name}: unknown topic error`);
}

// ─── Validation ───────────────────────────────────────
const minimal = fs.readFileSync(path.join(fixturesDir, 'minimal.ckm.json'), 'utf-8');
assert(ckm.validateManifest(minimal).valid === true, 'validate: minimal is valid');
assert(ckm.validateManifest('{}').valid === false, 'validate: {} is invalid');

const v1 = fs.readFileSync(path.join(fixturesDir, 'v1-legacy.ckm.json'), 'utf-8');
assert(ckm.validateManifest(v1).valid === false, 'validate: v1 is invalid');

// ─── Version Detection ───────────────────────────────
assert(ckm.detectVersion(minimal) === 2, 'detect: minimal is v2');
assert(ckm.detectVersion(v1) === 1, 'detect: v1-legacy is v1');

// ─── Migration ────────────────────────────────────────
const migrated = ckm.migrateV1toV2(v1);
assert(migrated.meta && migrated.version === '2.0.0', 'migrate: v1 to v2');

// ─── Producer API (Builder) ───────────────────────────
const builder = ckm.createManifestBuilder('test-tool', 'typescript')
  .generator('test@1.0')
  .addConcept('FooConfig', 'foo', 'A foo config.', ['config'])
  .addConceptProperty('foo', 'bar', 'string', 'Bar prop.', true, 'baz')
  .addOperation('doFoo', 'Does foo.', ['foo'])
  .addOperationInput('doFoo', 'x', 'string', true, 'Input x')
  .addConstraint('Must foo', 'doFoo', 'error')
  .addConfig('foo.bar', 'string', 'Bar config.', true, 'baz');

const built = builder.buildJson();
assert(built.version === '2.0.0', 'builder: version');
assert(built.meta.project === 'test-tool', 'builder: project');
assert(built.concepts.length === 1, 'builder: 1 concept');
assert(built.operations.length === 1, 'builder: 1 operation');
assert(built.constraints.length === 1, 'builder: 1 constraint');
assert(built.configSchema.length === 1, 'builder: 1 config');

// Builder output validates
assert(ckm.validateManifest(built).valid === true, 'builder: output validates');

// Builder output loads in engine
const bEngine = ckm.createCkmEngine(built);
assert(bEngine.topicsCount >= 1, 'builder: engine loads built manifest');

// ─── Summary ──────────────────────────────────────────
console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed > 0 ? 1 : 0);
