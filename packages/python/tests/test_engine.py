"""Tests for the CKM PyO3 wrapper.

Verifies that the thin Python wrapper correctly delegates to the Rust core
and marshals data between Python dicts and Rust types via JSON.
"""

import json
import os
from pathlib import Path

import pytest

from ckm import (
    CkmEngine,
    create_engine,
    detect_version,
    migrate_v1_to_v2,
    validate_manifest,
)

# ─── Fixtures ───────────────────────────────────────────────────────────────

CONFORMANCE_DIR = Path(__file__).resolve().parent.parent.parent.parent / "conformance"
FIXTURES_DIR = CONFORMANCE_DIR / "fixtures"
EXPECTED_DIR = CONFORMANCE_DIR / "expected"

MINIMAL_V2 = json.dumps(
    {
        "$schema": "https://ckm.dev/schemas/v2.json",
        "version": "2.0.0",
        "meta": {
            "project": "test",
            "language": "typescript",
            "generator": "hand-authored",
            "generated": "2026-01-01T00:00:00.000Z",
        },
        "concepts": [
            {
                "id": "concept-calver-config",
                "name": "CalVerConfig",
                "slug": "calver",
                "what": "Configures CalVer validation rules.",
                "tags": ["config"],
                "properties": [
                    {
                        "name": "format",
                        "type": {"canonical": "string", "original": "CalVerFormat"},
                        "description": "Calendar format.",
                        "required": True,
                        "default": "YYYY.MM.DD",
                    }
                ],
            }
        ],
        "operations": [
            {
                "id": "op-validate",
                "name": "validate",
                "what": "Validates a calver version string.",
                "tags": ["calver"],
                "inputs": [
                    {
                        "name": "version",
                        "type": {"canonical": "string"},
                        "required": True,
                        "description": "The version string.",
                    }
                ],
            }
        ],
        "constraints": [
            {
                "id": "constraint-no-future",
                "rule": "No future dates.",
                "enforcedBy": "validate",
                "severity": "error",
            }
        ],
        "workflows": [],
        "configSchema": [
            {
                "key": "calver.format",
                "type": {"canonical": "string", "original": "CalVerFormat"},
                "description": "Calendar format.",
                "default": "YYYY.MM.DD",
                "required": True,
            }
        ],
    }
)

V1_LEGACY = json.dumps(
    {
        "project": "legacy",
        "generated": "2025-01-01T00:00:00.000Z",
        "concepts": [
            {
                "id": "concept-CalVerConfig",
                "name": "CalVerConfig",
                "what": "Configures CalVer.",
                "properties": [
                    {
                        "name": "format",
                        "type": "CalVerFormat",
                        "description": "The format.",
                    }
                ],
            }
        ],
        "operations": [],
        "constraints": [],
        "workflows": [],
        "configSchema": [],
    }
)


# ─── create_engine tests ───────────────────────────────────────────────────


class TestCreateEngine:
    """Tests for the create_engine function."""

    def test_create_engine_v2(self):
        engine = create_engine(MINIMAL_V2)
        assert isinstance(engine, CkmEngine)

    def test_create_engine_invalid_json(self):
        with pytest.raises(ValueError, match="Invalid JSON"):
            create_engine("not json")

    def test_create_engine_v1_auto_migration(self):
        engine = create_engine(V1_LEGACY)
        assert isinstance(engine, CkmEngine)
        # v1 manifests are auto-migrated; should still derive topics
        assert engine.topics_count() >= 1


# ─── CkmEngine method tests ────────────────────────────────────────────────


class TestCkmEngine:
    """Tests for CkmEngine methods."""

    @pytest.fixture()
    def engine(self):
        return create_engine(MINIMAL_V2)

    def test_topics_count(self, engine):
        assert engine.topics_count() == 1

    def test_get_topic_index(self, engine):
        output = engine.get_topic_index("my-tool")
        assert "my-tool CKM" in output or "my-tool" in output
        assert "calver" in output

    def test_get_topic_index_default(self, engine):
        output = engine.get_topic_index()
        assert "calver" in output

    def test_get_topic_content_found(self, engine):
        content = engine.get_topic_content("calver")
        assert content is not None
        assert "CalVerConfig" in content

    def test_get_topic_content_not_found(self, engine):
        content = engine.get_topic_content("nonexistent")
        assert content is None

    def test_get_topic_json_index(self, engine):
        result = engine.get_topic_json()
        assert isinstance(result, dict)
        assert "topics" in result
        assert len(result["topics"]) == 1
        assert result["topics"][0]["name"] == "calver"

    def test_get_topic_json_single(self, engine):
        result = engine.get_topic_json("calver")
        assert isinstance(result, dict)
        assert result["name"] == "calver"

    def test_get_topic_json_error(self, engine):
        result = engine.get_topic_json("nonexistent")
        assert isinstance(result, dict)
        assert "error" in result
        assert "calver" in result["topics"]

    def test_get_manifest(self, engine):
        manifest = engine.get_manifest()
        assert isinstance(manifest, dict)
        assert manifest["meta"]["project"] == "test"
        assert manifest["version"] == "2.0.0"

    def test_inspect(self, engine):
        info = engine.inspect()
        assert isinstance(info, dict)
        assert info["meta"]["project"] == "test"
        assert info["counts"]["concepts"] == 1
        assert info["counts"]["operations"] == 1
        assert info["counts"]["topics"] == 1
        assert "calver" in info["topicNames"]


# ─── validate_manifest tests ───────────────────────────────────────────────


class TestValidateManifest:
    """Tests for the validate_manifest function."""

    def test_valid_manifest(self):
        result = validate_manifest(MINIMAL_V2)
        assert isinstance(result, dict)
        assert result["valid"] is True
        assert result["errors"] == []

    def test_invalid_manifest_missing_meta(self):
        data = json.dumps(
            {
                "version": "2.0.0",
                "concepts": [],
                "operations": [],
                "constraints": [],
                "workflows": [],
                "configSchema": [],
            }
        )
        result = validate_manifest(data)
        assert result["valid"] is False
        assert len(result["errors"]) > 0

    def test_invalid_json(self):
        with pytest.raises(ValueError, match="Invalid JSON"):
            validate_manifest("not json")


# ─── detect_version tests ──────────────────────────────────────────────────


class TestDetectVersion:
    """Tests for the detect_version function."""

    def test_v2_manifest(self):
        assert detect_version(MINIMAL_V2) == 2

    def test_v1_manifest(self):
        assert detect_version(V1_LEGACY) == 1

    def test_malformed_data(self):
        assert detect_version("42") == 1

    def test_invalid_json(self):
        with pytest.raises(ValueError, match="Invalid JSON"):
            detect_version("not json")


# ─── migrate_v1_to_v2 tests ────────────────────────────────────────────────


class TestMigrateV1ToV2:
    """Tests for the migrate_v1_to_v2 function."""

    def test_basic_migration(self):
        result = migrate_v1_to_v2(V1_LEGACY)
        assert isinstance(result, dict)
        assert result["version"] == "2.0.0"
        assert result["meta"]["project"] == "legacy"

    def test_concepts_migrated(self):
        result = migrate_v1_to_v2(V1_LEGACY)
        assert len(result["concepts"]) == 1
        concept = result["concepts"][0]
        assert concept["name"] == "CalVerConfig"
        assert concept["slug"] == "calver"
        assert "config" in concept["tags"]

    def test_invalid_json(self):
        with pytest.raises(ValueError, match="Invalid JSON"):
            migrate_v1_to_v2("not json")


# ─── Conformance fixture tests ──────────────────────────────────────────────


@pytest.mark.skipif(
    not FIXTURES_DIR.exists(),
    reason="Conformance fixtures not found",
)
class TestConformance:
    """Tests using the shared conformance fixtures."""

    def test_minimal_fixture_loads(self):
        fixture_path = FIXTURES_DIR / "minimal.ckm.json"
        manifest_json = fixture_path.read_text()
        engine = create_engine(manifest_json)
        assert engine.topics_count() == 1

    def test_minimal_validate(self):
        fixture_path = FIXTURES_DIR / "minimal.ckm.json"
        manifest_json = fixture_path.read_text()
        result = validate_manifest(manifest_json)
        assert result["valid"] is True

    def test_minimal_inspect(self):
        fixture_path = FIXTURES_DIR / "minimal.ckm.json"
        manifest_json = fixture_path.read_text()
        engine = create_engine(manifest_json)
        info = engine.inspect()
        assert info["meta"]["project"] == "minimal-tool"
        assert info["counts"]["topics"] == 1

    def test_v1_legacy_detection(self):
        fixture_path = FIXTURES_DIR / "v1-legacy.ckm.json"
        manifest_json = fixture_path.read_text()
        assert detect_version(manifest_json) == 1

    def test_v1_legacy_migration(self):
        fixture_path = FIXTURES_DIR / "v1-legacy.ckm.json"
        manifest_json = fixture_path.read_text()
        result = migrate_v1_to_v2(manifest_json)
        assert result["version"] == "2.0.0"

    def test_v1_legacy_engine(self):
        fixture_path = FIXTURES_DIR / "v1-legacy.ckm.json"
        manifest_json = fixture_path.read_text()
        engine = create_engine(manifest_json)
        # v1 manifests auto-migrate; engine should work
        assert engine.topics_count() >= 0

    def test_edge_cases_validate(self):
        fixture_path = FIXTURES_DIR / "edge-cases.ckm.json"
        if fixture_path.exists():
            manifest_json = fixture_path.read_text()
            result = validate_manifest(manifest_json)
            assert isinstance(result, dict)
            assert "valid" in result
