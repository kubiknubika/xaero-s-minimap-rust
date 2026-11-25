#!/usr/bin/env python3
"""Smoke test for scripts/package_native_mod.py (no Horizon checkout needed)."""

from __future__ import annotations

import subprocess
import sys
import tempfile
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "package_native_mod.py"
CRATES = [
    "xaerominimap-json",
    "xaerominimap-core",
    "xaerominimap-settings",
    "xaerominimap-controls",
    "xaerominimap-network",
    "xaerominimap-render",
    "xaerominimap-horizon",
]


def main() -> int:
    with tempfile.TemporaryDirectory() as tmp:
        out = Path(tmp) / "bundle"
        subprocess.run([sys.executable, str(SCRIPT), str(out)], check=True)

        mixin = tomllib.loads((out / "horizon-mixin.toml").read_text())
        assert mixin["api"]["path"] == "api"
        assert {entry["id"] for entry in mixin["crate"]} == {
            name.replace("-", "_") for name in CRATES
        }
        assert mixin["compatibility"]["horizon_revision"]
        injection = mixin["injection"][0]
        assert injection["hook"] == "frame"
        assert injection["helper"] == "frame.rs"
        assert (out / "frame.rs").is_file()
        assert {i["id"] for i in mixin["injection"]} == {"xaero-frame", "xaero-key"}
        assert (out / "key.rs").is_file()

        api = tomllib.loads((out / "api" / "Cargo.toml").read_text())
        assert "xaerominimap-horizon" in api["dependencies"]

        lib = (out / "api" / "src" / "lib.rs").read_text()
        assert "register_layer" in lib and "register_hooks" in lib
        assert "set_camera" in lib and "set_chunk_color" in lib
        assert "set_biome_chunk" in lib and "sync_waypoints" in lib
        assert "apply_control_events" in lib
        assert "set_key" in lib and "tick_controls" in lib
        assert "key_code" in lib and "set_chunk_rgb" in lib
        assert "register_settings_page" in lib and "fn settings()" in lib

        for name in CRATES:
            directory = out / name.replace("-", "_")
            manifest = tomllib.loads((directory / "Cargo.toml").read_text())
            assert manifest["package"]["name"] == name
            assert "version.workspace" not in (directory / "Cargo.toml").read_text()

    print("package_native_mod smoke test passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
