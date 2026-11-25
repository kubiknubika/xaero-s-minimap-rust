#!/usr/bin/env python3
"""Stage a Horizon native-mod bundle that links the Xaero Minimap port.

The bundle is a *native source mod*: the Horizon launcher compiles it into a
custom client. It contains the port crates as `[[crate]]`s and an `[api]` trait
mod that registers the HUD layer and hooks before the game starts.

    python3 scripts/package_native_mod.py [output-dir]

The output directory is created from scratch. Game resources and a Horizon
checkout are not part of this repository; build the result with:

    horizon-launcher native-build <output-dir> --instance <instance> --trust-native-code
"""

from __future__ import annotations

import re
import shutil
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
HORIZON_REVISION = "fed88405ed23fb3262760448480977a720b4a6ef"
HORIZON_HOOKS_GIT = "https://github.com/promikailamin/horizon.git"
CRATES = [
    "xaerominimap-json",
    "xaerominimap-core",
    "xaerominimap-settings",
    "xaerominimap-controls",
    "xaerominimap-network",
    "xaerominimap-render",
    "xaerominimap-horizon",
]
REPOSITORY = "https://github.com/kubiknubika/xaero-s-minimap-rust"

API_TOML = f'''[package]
name = "xaero-minimap-native"
version = "1.0.0"
edition = "2021"
license = "MIT"
description = "Horizon native trait mod that runs the Xaero Minimap Rust port"

[lib]
path = "src/lib.rs"

[dependencies]
# The launcher replaces this with the instance's Horizon source.
horizon-client = {{ git = "https://github.com/horizon-rust/horizon", rev = "{HORIZON_REVISION}" }}
horizon-hooks = {{ git = "{HORIZON_HOOKS_GIT}", rev = "{HORIZON_REVISION}" }}
xaerominimap-horizon = {{ path = "../xaerominimap_horizon" }}
'''

API_LIB = '''//! Native trait mod entry point. The launcher calls `hooks()` before the game
//! starts, which is the only moment Horizon accepts HUD/hook registration.
//!
//! Camera updates and chunk colours still come from a mixin (see
//! `CAMERA_AND_TILES.md`); this file owns the shared state and registration.

use std::sync::{Arc, Mutex, OnceLock};

use horizon_client::mod_api::Hooks;
use horizon_hooks::hud::Corner;
use xaerominimap_horizon::{
    register_hooks, register_layer, register_settings_page, ChunkColorGrid, ControlBridge,
    ControlEvent, HorizonControls, MinimapRenderer, MinimapState, Settings,
};

static STATE: OnceLock<Arc<MinimapState>> = OnceLock::new();
static TILES: OnceLock<Arc<ChunkColorGrid>> = OnceLock::new();
static CONTROLS: OnceLock<Mutex<ControlBridge>> = OnceLock::new();
static INPUT: OnceLock<Mutex<HorizonControls>> = OnceLock::new();
static SETTINGS: OnceLock<Arc<Mutex<Settings>>> = OnceLock::new();

/// The state mixins use to push camera/waypoint/radar updates.
pub fn state() -> Arc<MinimapState> {
    STATE.get_or_init(|| Arc::new(MinimapState::new())).clone()
}

/// The tile grid a world mixin fills from loaded chunks.
pub fn tiles() -> Arc<ChunkColorGrid> {
    TILES.get_or_init(|| Arc::new(ChunkColorGrid::new())).clone()
}

/// Called by a client-tick mixin with the local player's block position.
pub fn set_camera(x: i32, y: i32, z: i32) {
    state().set_camera(x, y, z);
}

/// Called by a chunk-load mixin; colours use the port's RGBA `Color`.
pub fn set_chunk_color(chunk_x: i32, chunk_z: i32, color: xaerominimap_horizon::Color) {
    tiles().set(chunk_x, chunk_z, color);
}

/// The generated frame mixin samples `native_biome` for nearby chunks. A
/// stable hash keeps a biome's colour constant between frames, so the map
/// does not flicker while the player moves.
pub fn set_biome_chunk(chunk_x: i32, chunk_z: i32, biome: Option<&str>) {
    use xaerominimap_horizon::Color;

    let color = match biome {
        Some(name) => {
            let mut hash = 2_166_136_261u32;
            for byte in name.bytes() {
                hash ^= byte as u32;
                hash = hash.wrapping_mul(16_777_619);
            }
            Color::rgb(
                (hash & 0xff) as u8 / 2 + 60,
                ((hash >> 8) & 0xff) as u8 / 2 + 60,
                ((hash >> 16) & 0xff) as u8 / 2 + 60,
            )
        }
        None => Color::rgb(24, 28, 32),
    };
    tiles().set(chunk_x, chunk_z, color);
}

/// Convenience for the frame mixin: bakes one averaged chunk tile.
pub fn set_chunk_rgb(chunk_x: i32, chunk_z: i32, r: u8, g: u8, b: u8) {
    set_chunk_color(chunk_x, chunk_z, xaerominimap_horizon::Color::rgb(r, g, b));
}

/// Maps a Horizon key name (`key.keyboard.o`) to the integer code the port's
/// key mappings store. Unknown keys are ignored.
pub fn key_code(name: &str) -> Option<i32> {
    xaerominimap_horizon::horizon_key_code(name)
}

/// Called after loading `MinimapSession` waypoints so the HUD shows them.
pub fn sync_waypoints(waypoints: &[xaerominimap_horizon::Waypoint]) {
    state().sync_waypoints(waypoints);
}

/// Applies `ControlEvent`s produced by `MinimapKeyMappings.tick(...)`.
pub fn apply_control_events(events: &[ControlEvent]) -> usize {
    CONTROLS
        .get_or_init(|| Mutex::new(ControlBridge::new(state())))
        .lock()
        .unwrap()
        .apply_all(events)
}

/// Feeds a Horizon key code (the integer stored by `MinimapKeyMappings`).
pub fn set_key(code: i32, pressed: bool) {
    INPUT
        .get_or_init(|| Mutex::new(HorizonControls::new(state())))
        .lock()
        .unwrap()
        .set_key(code, pressed);
}

/// Runs one key tick and applies the resulting HUD changes.
pub fn tick_controls() -> usize {
    INPUT
        .get_or_init(|| Mutex::new(HorizonControls::new(state())))
        .lock()
        .unwrap()
        .tick()
}

/// The port's settings store, persisted by the engine when options close.
pub fn settings() -> Arc<Mutex<Settings>> {
    SETTINGS
        .get_or_init(|| Arc::new(Mutex::new(Settings::new())))
        .clone()
}

pub fn hooks() -> Box<dyn Hooks> {
    if STATE.get().is_none() {
        let state = state();
        let renderer = MinimapRenderer::new(160.0, 1.0, 16);
        // Registration seals Horizon's registries: do it exactly once.
        let _ = register_layer(state.clone(), renderer, tiles(), Corner::TopLeft);
        let _ = register_hooks(state.clone());
        let _ = register_settings_page(settings(), state);
    }
    Box::new(Mod)
}

struct Mod;

impl Hooks for Mod {}
'''

FRAME_HELPER = '''// HEAD hook on `native_frame`: push the local player position, then bake
// map tiles from the client world snapshot Horizon shares with native mods.
// Each tile is the average vanilla map colour of the 16x16 surface columns,
// with a biome tint where a column has no map colour. `pub(super)` helpers
// are visible because the helper compiles into the game's module.
fn on_frame(game: &mut crate::game::Game<'_>) {
    let Some(player) = game.native_player() else {
        return;
    };
    let x = player.x.floor() as i32;
    let y = player.y.floor() as i32;
    let z = player.z.floor() as i32;
    xaero_minimap_native::set_camera(x, y, z);

    let center_x = x.div_euclid(16);
    let center_z = z.div_euclid(16);
    let from = [center_x - 5, center_z - 5];
    let to = [center_x + 5, center_z + 5];

    let Ok(chunks) = game.native_client_chunks(from, to) else {
        bake_biome_fallback(game, center_x, center_z);
        return;
    };
    let biomes = game.native_client_biomes();
    let height = chunks.height();

    for chunk_x in from[0]..=to[0] {
        for chunk_z in from[1]..=to[1] {
            if !chunks.loaded([chunk_x, chunk_z]) {
                continue;
            }
            let (mut red, mut green, mut blue) = (0u64, 0u64, 0u64);
            for lx in 0..16 {
                for lz in 0..16 {
                    let bx = chunk_x * 16 + lx;
                    let bz = chunk_z * 16 + lz;
                    let top = chunks.section_top([bx, bz]) - 1;
                    let mut color = 0u32;
                    for dy in 0..8 {
                        let by = top - dy;
                        if by < height.start {
                            break;
                        }
                        if let Some((_, map_color)) = game.native_block_map_color(chunks.block([bx, by, bz])) {
                            if map_color != 0 {
                                color = map_color;
                                break;
                            }
                        }
                    }
                    if color == 0 {
                        if let (Some(biome), Some(table)) = (chunks.biome([bx, top, bz]), &biomes) {
                            color = table.colors(biome, [bx, bz]).grass;
                        }
                    }
                    red += u64::from((color >> 16) as u8);
                    green += u64::from((color >> 8) as u8);
                    blue += u64::from(color as u8);
                }
            }
            let count = 256u64;
            xaero_minimap_native::set_chunk_rgb(
                chunk_x,
                chunk_z,
                (red / count) as u8,
                (green / count) as u8,
                (blue / count) as u8,
            );
        }
    }
}

fn bake_biome_fallback(game: &mut crate::game::Game<'_>, center_x: i32, center_z: i32) {
    for dx in -5..=5 {
        for dz in -5..=5 {
            let chunk_x = center_x + dx;
            let chunk_z = center_z + dz;
            let biome = game.native_biome([chunk_x * 16 + 8, 64, chunk_z * 16 + 8]);
            xaero_minimap_native::set_biome_chunk(chunk_x, chunk_z, biome.as_deref());
        }
    }
}
'''

KEY_HELPER = '''// HEAD hook on `native_key`: translate Horizon's key name to the integer
// code used by the port's MinimapKeyMappings, then feed the input bridge.
fn on_key(key: &str, pressed: bool) {
    if let Some(code) = xaero_minimap_native::key_code(key) {
        xaero_minimap_native::set_key(code, pressed);
    }
}
'''

MIXIN_TOML = f'''[mod]
id = "xaero-minimap"
version = "1.0.0"
priority = 100

[compatibility]
horizon_revision = "{HORIZON_REVISION}"

[api]
path = "api"

[[injection]]
id = "xaero-frame"
hook = "frame"
hook_version = 1
kind = "HEAD"
helper = "frame.rs"
bindings = [{{ name = "game", ty = "&mut crate::game::Game<'_>", value = "&mut *self" }}]

[[injection]]
id = "xaero-key"
hook = "key"
hook_version = 1
kind = "HEAD"
helper = "key.rs"
bindings = [
  {{ name = "key", ty = "&str", value = "key" }},
  {{ name = "pressed", ty = "bool", value = "pressed" }},
]

# The port crates are sibling folders of `[api]`, so the launcher stages them
# as native-crates/xaero-minimap/<crate>/ and the trait mod can depend on them.
'''

GUIDE = '''# Horizon native bundle (generated)

This folder was generated by `scripts/package_native_mod.py` from the Rust
workspace. It is a compile-time native mod, not a drop-in JAR.

Build it against a Horizon instance with native-code trust:

```sh
horizon-launcher native-build . --instance <instance> --trust-native-code
```

The generated `api/src/lib.rs` registers the HUD layer and `Hooks`.
`frame.rs` bakes tiles from `native_client_chunks`/`native_client_biomes`
(vanilla map colours averaged over each chunk) with a biome fallback, and
`key.rs` feeds `native_key` events into the ported controls system. See `HORIZON_INTEGRATION.md` in the source repository.

Do not edit the copied `xaerominimap_*` crates here: regenerate the bundle
after changing the workspace.
'''


def crate_dir(name: str) -> str:
    return name.replace("-", "_")


def rewrite_manifest(text: str) -> str:
    text = text.replace("version.workspace = true", 'version = "1.0.0"')
    text = text.replace("edition.workspace = true", 'edition = "2021"')
    text = text.replace("license.workspace = true", 'license = "MIT"')
    text = text.replace("repository.workspace = true", f'repository = "{REPOSITORY}"')
    for name in CRATES:
        text = text.replace(f'path = "../{name}"', f'path = "../{crate_dir(name)}"')
    return text


def main() -> int:
    out = Path(sys.argv[1] if len(sys.argv) > 1 else "target/xaero-minimap-native")
    if not out.is_absolute():
        out = ROOT / out
    if out.exists():
        shutil.rmtree(out)
    out.mkdir(parents=True)

    for name in CRATES:
        source = ROOT / "crates" / name
        target = out / crate_dir(name)
        shutil.copytree(source, target, ignore=shutil.ignore_patterns("target"))
        manifest = target / "Cargo.toml"
        manifest.write_text(rewrite_manifest(manifest.read_text()))

    api = out / "api"
    (api / "src").mkdir(parents=True)
    (api / "Cargo.toml").write_text(API_TOML)
    (api / "src" / "lib.rs").write_text(API_LIB)

    mixin = MIXIN_TOML + "".join(
        f'\n[[crate]]\nid = "{crate_dir(name)}"\npath = "{crate_dir(name)}"\n'
        for name in CRATES
    )
    (out / "horizon-mixin.toml").write_text(mixin)
    (out / "frame.rs").write_text(FRAME_HELPER)
    (out / "key.rs").write_text(KEY_HELPER)
    (out / "README.md").write_text(GUIDE)

    # Fail early if we produced invalid TOML.
    for manifest in [api / "Cargo.toml", out / "horizon-mixin.toml"]:
        with manifest.open("rb") as handle:
            tomllib.load(handle)

    print(f"Staged {len(CRATES)} crates + trait mod in {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
