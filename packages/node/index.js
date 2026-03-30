const binding = require('./ckm.node');

// ─── Consumer API (reading manifests) ─────────────────────────────

/**
 * Creates a CKM engine from a manifest (object or JSON string).
 * If the manifest is v1, it is automatically migrated to v2 internally.
 *
 * @param {object | string} manifest - CKM manifest (v1 or v2)
 * @returns {object} CKM engine with progressive disclosure methods
 */
function createCkmEngine(manifest) {
  const json = typeof manifest === 'string' ? manifest : JSON.stringify(manifest);
  const engine = binding.createCkmEngine(json);

  return {
    get topicsCount() { return engine.topicsCount(); },
    getTopicIndex(toolName) { return engine.getTopicIndex(toolName); },
    getTopicContent(topicName) { return engine.getTopicContent(topicName); },
    getTopicJson(topicName) { return JSON.parse(engine.getTopicJson(topicName)); },
    getManifest() { return JSON.parse(engine.getManifest()); },
    inspect() { return JSON.parse(engine.inspect()); },
  };
}

/**
 * Validates a CKM manifest against the v2 schema.
 * @param {object | string} data
 * @returns {{ valid: boolean, errors: Array<{ path: string, message: string }> }}
 */
function validateManifest(data) {
  const json = typeof data === 'string' ? data : JSON.stringify(data);
  return JSON.parse(binding.validateManifest(json));
}

/**
 * Migrates a v1 manifest to v2 format.
 * @param {object | string} data - v1 manifest
 * @returns {object} v2 manifest
 */
function migrateV1toV2(data) {
  const json = typeof data === 'string' ? data : JSON.stringify(data);
  return JSON.parse(binding.migrateV1ToV2(json));
}

/**
 * Detects whether a manifest is v1 or v2.
 * @param {object | string} data
 * @returns {number} 1 or 2
 */
function detectVersion(data) {
  const json = typeof data === 'string' ? data : JSON.stringify(data);
  return binding.detectVersion(json);
}

// ─── Producer API (building manifests) ────────────────────────────

/**
 * Creates a new CKM manifest builder.
 *
 * This is the PRODUCER side — generators (like forge-ts) use this to
 * construct valid manifests with type safety instead of hand-rolling JSON.
 *
 * @param {string} project - Project name
 * @param {string} language - Source language (e.g., "typescript", "python", "rust")
 * @returns {CkmManifestBuilder}
 *
 * @example
 * const builder = createManifestBuilder('my-tool', 'typescript');
 * builder.generator('forge-ts@1.0.0');
 * builder.addConcept('CalVerConfig', 'calver', 'Configures CalVer.', ['config']);
 * builder.addOperation('validate', 'Validates a version.', ['calver']);
 * const manifest = builder.buildJson(); // returns parsed object
 */
function createManifestBuilder(project, language) {
  const inner = new binding.CkmManifestBuilderWrapper(project, language);

  return {
    generator(name) { inner.generator(name); return this; },
    sourceUrl(url) { inner.sourceUrl(url); return this; },
    addConcept(name, slug, what, tags) { inner.addConcept(name, slug, what, tags || []); return this; },
    addConceptProperty(slug, name, type, desc, required, defaultVal) {
      inner.addConceptProperty(slug, name, type, desc, required !== false, defaultVal || null);
      return this;
    },
    addOperation(name, what, tags) { inner.addOperation(name, what, tags || []); return this; },
    addOperationInput(opName, paramName, type, required, desc) {
      inner.addOperationInput(opName, paramName, type, required !== false, desc);
      return this;
    },
    addConstraint(rule, enforcedBy, severity) {
      inner.addConstraint(rule, enforcedBy, severity || 'error');
      return this;
    },
    addConfig(key, type, desc, required, defaultVal) {
      inner.addConfig(key, type, desc, required !== false, defaultVal || null);
      return this;
    },
    /** Returns the manifest as a JSON string. */
    build() { return inner.build(); },
    /** Returns the manifest as a parsed object. */
    buildJson() { return JSON.parse(inner.build()); },
  };
}

module.exports = {
  // Consumer API
  createCkmEngine,
  validateManifest,
  migrateV1toV2,
  detectVersion,
  // Producer API
  createManifestBuilder,
};
