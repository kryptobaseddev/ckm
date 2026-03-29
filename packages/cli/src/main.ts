import { existsSync, readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { createCkmEngine, detectVersion, migrateV1toV2, validateManifest } from 'ckm';
import { Command } from 'commander';

const program = new Command();

program.name('ckm').version('0.1.0').description('CKM CLI — Codebase Knowledge Manifest tools');

// ─── ckm [topic] ────────────────────────────────────────────────

program
  .command('browse [topic]', { isDefault: true })
  .description('Browse CKM topics')
  .option('-f, --file <path>', 'Path to ckm.json')
  .option('--json', 'Machine-readable output')
  .action((topic: string | undefined, opts: { file?: string; json?: boolean }) => {
    const manifest = loadManifest(opts.file);
    const engine = createCkmEngine(manifest);

    if (opts.json) {
      console.log(JSON.stringify(engine.getTopicJson(topic), null, 2));
    } else if (topic) {
      const content = engine.getTopicContent(topic);
      if (content === null) {
        console.error(`Unknown topic: ${topic}`);
        console.log(engine.getTopicIndex('ckm'));
        process.exitCode = 1;
      } else {
        console.log(content);
      }
    } else {
      console.log(engine.getTopicIndex('ckm'));
    }
  });

// ─── ckm validate <file> ────────────────────────────────────────

program
  .command('validate <file>')
  .description('Validate a ckm.json against the v2 schema')
  .action((file: string) => {
    const data = loadFile(file);
    const result = validateManifest(data);

    if (result.valid) {
      console.log('Valid ckm.json v2 manifest.');
    } else {
      console.error('Invalid manifest:');
      for (const err of result.errors) {
        console.error(`  ${err.path}: ${err.message}`);
      }
      process.exitCode = 1;
    }
  });

// ─── ckm migrate <file> ─────────────────────────────────────────

program
  .command('migrate <file>')
  .description('Migrate a v1 ckm.json to v2 format')
  .option('--dry-run', 'Show output without writing')
  .option('-o, --output <path>', 'Output file path')
  .action(async (file: string, opts: { dryRun?: boolean; output?: string }) => {
    const data = loadFile(file);
    const version = detectVersion(data);

    if (version === 2) {
      console.log('Already v2 format. No migration needed.');
      return;
    }

    const migrated = migrateV1toV2(data as any);
    const output = JSON.stringify(migrated, null, 2);

    if (opts.dryRun) {
      console.log(output);
    } else {
      const outputPath = opts.output || file.replace('.json', '.v2.json');
      const fs = await import('node:fs');
      fs.writeFileSync(outputPath, `${output}\n`);
      console.log(`Migrated to: ${outputPath}`);
    }
  });

// ─── ckm inspect <file> ─────────────────────────────────────────

program
  .command('inspect <file>')
  .description('Show manifest statistics')
  .action((file: string) => {
    const data = loadFile(file);
    const engine = createCkmEngine(data);
    const result = engine.inspect();

    console.log(`Project:    ${result.meta.project}`);
    console.log(`Language:   ${result.meta.language}`);
    console.log(`Generator:  ${result.meta.generator}`);
    console.log('');
    console.log(`Concepts:     ${result.counts.concepts}`);
    console.log(`Operations:   ${result.counts.operations}`);
    console.log(`Constraints:  ${result.counts.constraints}`);
    console.log(`Workflows:    ${result.counts.workflows}`);
    console.log(`Config keys:  ${result.counts.configKeys}`);
    console.log(`Topics:       ${result.counts.topics} (auto-derived)`);
  });

// ─── Helpers ─────────────────────────────────────────────────────

function loadManifest(filePath?: string): unknown {
  const searchPaths = [filePath, 'ckm.json', 'docs/ckm.json', '.ckm/ckm.json'].filter(
    Boolean,
  ) as string[];

  for (const p of searchPaths) {
    const resolved = resolve(p);
    if (existsSync(resolved)) {
      return JSON.parse(readFileSync(resolved, 'utf-8'));
    }
  }

  console.error('No ckm.json found. Searched:');
  for (const p of searchPaths) {
    console.error(`  ${p}`);
  }
  process.exit(1);
}

function loadFile(filePath: string): unknown {
  const resolved = resolve(filePath);
  if (!existsSync(resolved)) {
    console.error(`File not found: ${filePath}`);
    process.exit(1);
  }
  return JSON.parse(readFileSync(resolved, 'utf-8'));
}

program.parse();
