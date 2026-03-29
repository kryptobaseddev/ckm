const { createCkmEngine: _createEngine, validateManifest: _validate, migrateV1ToV2: _migrate, detectVersion: _detect } = require('./ckm.node');

/**
 * Creates a CKM engine from a manifest (object or JSON string).
 * If the manifest is v1, it is automatically migrated to v2 internally.
 *
 * @param {object | string} manifest - CKM manifest (v1 or v2)
 * @returns {object} CKM engine with progressive disclosure methods
 */
function createCkmEngine(manifest) {
  const json = typeof manifest === 'string' ? manifest : JSON.stringify(manifest);
  const engine = _createEngine(json);

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
  return JSON.parse(_validate(json));
}

/**
 * Migrates a v1 manifest to v2 format.
 * @param {object | string} data - v1 manifest
 * @returns {object} v2 manifest
 */
function migrateV1toV2(data) {
  const json = typeof data === 'string' ? data : JSON.stringify(data);
  return JSON.parse(_migrate(json));
}

/**
 * Detects whether a manifest is v1 or v2.
 * @param {object | string} data
 * @returns {number} 1 or 2
 */
function detectVersion(data) {
  const json = typeof data === 'string' ? data : JSON.stringify(data);
  return _detect(json);
}

module.exports = { createCkmEngine, validateManifest, migrateV1toV2, detectVersion };
