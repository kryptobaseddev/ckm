/**
 * CKM terminal formatter — plain text output with no color dependencies.
 *
 * @packageDocumentation
 */

import type { CkmConcept, CkmConfigEntry, CkmProperty, CkmTopic, CkmTypeRef } from './types.js';

/**
 * Formats the topic index for terminal display (Level 0).
 *
 * @param topics - All derived topics.
 * @param toolName - CLI tool name for the usage line.
 * @returns Formatted string within 300 token budget.
 *
 * @public
 */
export function formatTopicIndex(topics: readonly CkmTopic[], toolName: string): string {
  const lines = [
    `${toolName} CKM — Codebase Knowledge Manifest`,
    '',
    `Usage: ${toolName} ckm [topic] [--json] [--llm]`,
    '',
    'Topics:',
  ];

  const maxName = Math.max(...topics.map((t) => t.name.length), 0);
  for (const topic of topics) {
    lines.push(`  ${topic.name.padEnd(maxName + 2)}${topic.summary}`);
  }

  lines.push('');
  lines.push('Flags:');
  lines.push('  --json    Machine-readable CKM output (concepts, operations, config schema)');
  lines.push('  --llm     Full API context for LLM agents (forge-ts llms.txt)');

  return lines.join('\n');
}

/**
 * Formats a topic's content for human-readable terminal display (Level 1).
 *
 * @param topics - All derived topics.
 * @param topicName - Topic slug to display.
 * @returns Formatted content, or null if not found. Within 800 token budget.
 *
 * @public
 */
export function formatTopicContent(topics: readonly CkmTopic[], topicName: string): string | null {
  const topic = topics.find((t) => t.name === topicName);
  if (!topic) return null;

  const lines: string[] = [`# ${topic.summary}`, ''];

  // Concepts
  if (topic.concepts.length > 0) {
    lines.push('## Concepts', '');
    for (const c of topic.concepts) {
      lines.push(`  ${c.name} — ${c.what}`);
      if (c.properties) {
        for (const p of c.properties) {
          const def = findDefault(topic.configSchema, c, p);
          const typeStr = formatTypeRef(p.type);
          lines.push(`    ${p.name}: ${typeStr}${def}`);
          if (p.description) {
            lines.push(`      ${p.description}`);
          }
        }
      }
      lines.push('');
    }
  }

  // Operations
  if (topic.operations.length > 0) {
    lines.push('## Operations', '');
    for (const o of topic.operations) {
      lines.push(`  ${o.name}() — ${o.what}`);
      if (o.inputs) {
        for (const i of o.inputs) {
          lines.push(`    @param ${i.name}: ${i.description}`);
        }
      }
      lines.push('');
    }
  }

  // Config schema
  if (topic.configSchema.length > 0) {
    lines.push('## Config Fields', '');
    for (const c of topic.configSchema) {
      const typeStr = formatTypeRef(c.type);
      const defaultStr = c.default ? ` = ${c.default}` : '';
      lines.push(`  ${c.key}: ${typeStr}${defaultStr}`);
      if (c.description) {
        lines.push(`    ${c.description}`);
      }
    }
    lines.push('');
  }

  // Constraints
  if (topic.constraints.length > 0) {
    lines.push('## Constraints', '');
    for (const c of topic.constraints) {
      lines.push(`  [${c.id}] ${c.rule}`);
      lines.push(`    Enforced by: ${c.enforcedBy}`);
    }
    lines.push('');
  }

  return lines.join('\n');
}

function formatTypeRef(typeRef: CkmTypeRef | string): string {
  if (typeof typeRef === 'string') return typeRef;
  if (typeRef.original) return typeRef.original;
  return typeRef.canonical;
}

function findDefault(
  configSchema: readonly CkmConfigEntry[],
  concept: CkmConcept,
  property: CkmProperty,
): string {
  for (const entry of configSchema) {
    if (entry.key.endsWith(`.${property.name}`) && entry.default !== null) {
      return ` = ${entry.default}`;
    }
  }
  return '';
}
