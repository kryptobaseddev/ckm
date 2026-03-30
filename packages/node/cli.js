#!/usr/bin/env node
/**
 * CKM CLI — browse, validate, migrate, and inspect ckm.json manifests.
 * Part of the ckm-sdk package.
 *
 * Usage:
 *   npx ckm-sdk browse [topic] [--json] [--file path]
 *   npx ckm-sdk validate <file>
 *   npx ckm-sdk inspect <file>
 *   npx ckm-sdk migrate <file> [--dry-run]
 */
const fs = require('fs');
const path = require('path');
const ckm = require('./index.js');

const args = process.argv.slice(2);
const command = args[0];

function findFlag(name) {
  const i = args.indexOf(name);
  if (i === -1) return null;
  return args[i + 1] || true;
}

function hasFlag(name) {
  return args.includes(name);
}

function loadManifest(filePath) {
  const paths = [
    filePath,
    'ckm.json',
    'docs/ckm.json',
    '.ckm/ckm.json',
  ].filter(Boolean);

  for (const p of paths) {
    const resolved = path.resolve(p);
    if (fs.existsSync(resolved)) {
      return fs.readFileSync(resolved, 'utf-8');
    }
  }
  console.error('No ckm.json found. Searched:', paths.join(', '));
  process.exit(1);
}

function loadFile(filePath) {
  if (!filePath) { console.error('Usage: ckm-sdk validate <file>'); process.exit(1); }
  const resolved = path.resolve(filePath);
  if (!fs.existsSync(resolved)) { console.error(`File not found: ${filePath}`); process.exit(1); }
  return fs.readFileSync(resolved, 'utf-8');
}

if (!command || command === 'help' || command === '--help' || command === '-h') {
  console.log(`ckm-sdk — Codebase Knowledge Manifest CLI

Usage:
  ckm-sdk browse [topic] [--json] [--file <path>]   Browse topics
  ckm-sdk validate <file>                            Validate manifest
  ckm-sdk inspect <file>                             Show manifest stats
  ckm-sdk migrate <file> [--dry-run]                 Migrate v1 to v2

Examples:
  npx ckm-sdk browse
  npx ckm-sdk browse calver --file docs/ckm.json
  npx ckm-sdk browse --json
  npx ckm-sdk validate docs/ckm.json
  npx ckm-sdk inspect docs/ckm.json`);
  process.exit(0);
}

if (command === 'browse') {
  const topic = args[1] && !args[1].startsWith('-') ? args[1] : undefined;
  const file = findFlag('--file');
  const json = hasFlag('--json');
  const data = loadManifest(typeof file === 'string' ? file : null);
  const engine = ckm.createCkmEngine(data);

  if (json) {
    console.log(JSON.stringify(engine.getTopicJson(topic), null, 2));
  } else if (topic) {
    const content = engine.getTopicContent(topic);
    if (!content) {
      console.error(`Unknown topic: ${topic}`);
      console.log(engine.getTopicIndex('ckm'));
      process.exit(1);
    }
    console.log(content);
  } else {
    console.log(engine.getTopicIndex('ckm'));
  }
} else if (command === 'validate') {
  const data = loadFile(args[1]);
  const result = ckm.validateManifest(data);
  if (result.valid) {
    console.log('Valid ckm.json v2 manifest.');
  } else {
    console.error('Invalid manifest:');
    for (const err of result.errors) {
      console.error(`  ${err.path}: ${err.message}`);
    }
    process.exit(1);
  }
} else if (command === 'inspect') {
  const data = loadFile(args[1]);
  const engine = ckm.createCkmEngine(data);
  const r = engine.inspect();
  console.log(`Project:    ${r.meta.project}`);
  console.log(`Language:   ${r.meta.language}`);
  console.log(`Generator:  ${r.meta.generator}`);
  console.log('');
  console.log(`Concepts:     ${r.counts.concepts}`);
  console.log(`Operations:   ${r.counts.operations}`);
  console.log(`Constraints:  ${r.counts.constraints}`);
  console.log(`Workflows:    ${r.counts.workflows}`);
  console.log(`Config keys:  ${r.counts.configKeys}`);
  console.log(`Topics:       ${r.counts.topics} (auto-derived)`);
} else if (command === 'migrate') {
  const data = loadFile(args[1]);
  const version = ckm.detectVersion(data);
  if (version === 2) {
    console.log('Already v2 format.');
  } else {
    const migrated = ckm.migrateV1toV2(data);
    if (hasFlag('--dry-run')) {
      console.log(JSON.stringify(migrated, null, 2));
    } else {
      const outPath = args[1].replace('.json', '.v2.json');
      fs.writeFileSync(outPath, JSON.stringify(migrated, null, 2) + '\n');
      console.log(`Migrated to: ${outPath}`);
    }
  }
} else {
  console.error(`Unknown command: ${command}. Run 'npx ckm-sdk help' for usage.`);
  process.exit(1);
}
