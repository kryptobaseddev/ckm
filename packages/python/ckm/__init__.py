"""CKM — Codebase Knowledge Manifest SDK (powered by Rust core)."""

try:
    from ckm_python import (
        CkmEngine,
        create_engine,
        validate_manifest,
        migrate_v1_to_v2,
        detect_version,
    )
except ImportError:
    from ckm.ckm_python import (
        CkmEngine,
        create_engine,
        validate_manifest,
        migrate_v1_to_v2,
        detect_version,
    )

__all__ = [
    "CkmEngine",
    "create_engine",
    "validate_manifest",
    "migrate_v1_to_v2",
    "detect_version",
]
