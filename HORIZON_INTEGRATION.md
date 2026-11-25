# How to finish: Xaero's Minimap inside Horizon

This repository now contains the full data/logic port plus a first Horizon
adapter (`crates/xaerominimap-horizon`). The remaining work is engine glue that
must live in a **Horizon native mod**. Here is the concrete path.

## What was verified about Horizon

- The upstream repository named in the README (`horizon-rust/horizon`) is not
  reachable from this workspace. The public fork
  `https://github.com/promikailamin/horizon` contains the source; the adapter
  pins `horizon-hooks` to commit `fed88405ed23fb3262760448480977a720b4a6ef`.
- Horizon native mods are **compile-time mixin bundles** (`.hnmod`), not runtime
  plugins. Start from `https://github.com/levzzz5154/horizon-mod-template`.
  `https://github.com/promikailamin/horizon_modloader` can prebuild an
  `.hmmod` so players do not compile the client themselves.
- The platform API exposes `horizon_hooks::hud::{Layer, Canvas, Corner}` for
  HUD drawing, `horizon_hooks::Hooks` for frame/tick callbacks, and the
  `settings`/`pages` registries for options screens. Packet observation is a
  mixin hook documented as `native_client_packet` in `docs/native-mods.md`.

## What the adapter already does

`xaerominimap-horizon` implements:

- `OffsetCanvas`: the port's pixel `Canvas` on top of Horizon's HUD `Canvas`,
  with ARGB conversion and clipping to the corner box.
- `MinimapLayer`: draws the map square, waypoint markers, labels and radar dots
  through `MinimapRenderer`, honouring runtime view options.
- `MinimapState` / `MinimapViewConfig`: atomics for camera, connection,
  enable/size/scale/corner/labels/radar plus mutex-protected waypoint and radar
  lists.
- `ChunkColorGrid` / `SolidTiles`: ready thread-safe `TileSource`s for chunk
  colours supplied by a mixin.
- `PacketBridge`: applies the five ported packet effects to `ClientWorldData`,
  `ClientSyncedTrackedPlayerManager` and the HUD radar list.
- `ControlBridge`: applies `ControlEvent`s from `MinimapKeyMappings` to map
  toggle, zoom, enlarge-map, radar and cave-mode view options.
- `HorizonKeyState` / `HorizonControls`: feeds key codes into the ported
  `ControlsRegister.tick(...)` and applies the resulting events; repeated
  presses do not queue extra clicks.
- `MinimapState::sync_waypoints(&[Waypoint])`: converts ported waypoints into
  HUD markers (deleted/disabled ones are skipped).
- `MinimapSettingsPage`: registers all 115 `UI_BINDINGS` as Horizon settings
  rows (Toggle/Cycle/Slider/Field/button) and mirrors the two HUD toggles into
  `MinimapState`.
- `horizon_key_code(name)`: maps `key.keyboard.*` names to the LWJGL codes the
  Java `KeyMapping`s use.
- `MinimapHooks`: clears transient lists when the connection id changes.
- `register_layer(...)` / `register_hooks(...)`: registration functions that
  must be called before the game starts.

Twenty-four adapter tests cover conversion, clipping, drawing, view options,
tile lookup, waypoint conversion, packet/control/key effects, settings rows and
connection changes. `cargo test --workspace` is green (174 tests).

## Remaining implementation steps

1. **Native mod shell.** Generate one with
   `python3 scripts/package_native_mod.py target/xaero-minimap-native`. It
   stages all port crates as `[[crate]]`s, writes `horizon-mixin.toml` pinned to
   the same Horizon commit, and generates an `[api]` trait mod whose `hooks()`
   calls `register_layer` and `register_hooks`. Build it with
   `horizon-launcher native-build <dir> --instance <instance> --trust-native-code`.
   Validate staging with `python3 scripts/test_package_native_mod.py`.
2. **Camera + tiles mixin.** Generated `frame.rs` reads `native_player()`,
   takes `native_client_chunks(...)` and `native_client_biomes()` and bakes each
   16×16 chunk tile from vanilla block map colours (biome tint fallback). It is
   a HEAD injection on the `frame` hook and fills `ChunkColorGrid`.
3. **Waypoints + radar sync.** Feed `MinimapSession`'s waypoint sets into
   `MinimapState::set_waypoints` and the radar lists into `set_radar`. Wire the
   waypoint lifecycle handlers already ported in `xaerominimap-core`.
4. **Controls.** The generated `key.rs` HEAD injection on `native_key`
   translates Horizon key names via `horizon_key_code(...)` and feeds
   `api::set_key(...)` → `api::tick_controls()` → `ControlBridge`. HUD-visible
   actions (toggle, zoom, enlarge, radar, cave mode) work end to end. The
   waypoint-menu/add-waypoint *screens* still need GUI glue if you want them
   in-game rather than only file-based waypoint editing.
5. **Options GUI.** `MinimapSettingsPage` registers all 115 bindings in
   Horizon's pages registry. The engine draws the widgets; the port handles
   `act(...)` and persists values in the shared `Settings` store.
6. **Server packets.** Register the five ported packet codecs on Horizon's
   client/server channels and apply `PacketEffect`s to `MinimapState`/
   `MinimapSession` (rules, server level id, synced tracked players). Observe
   vanilla/mod payloads with the `native_client_packet` mixin hook.
7. **WorldMap protocol — optional.** The classes `xaero.map.*` are **not** in
   the minimap JAR, so the protocol cannot be extracted here. To support map
   image/region exchange you need Xaero's WorldMap JAR (CurseForge/Modrinth)
   or a written protocol spec; then add its packets beside the existing five.
   The minimap itself works without it.
8. **Build and test.** Build the `.hnmod` with the template, optionally convert
   to `.hmmod` with hmloader, then run the game with the mod. Add a differential
   fixture that compares a rendered frame against the Java mod for a fixed
   seed/position.

## Acceptance checklist

- [x] Ported data/logic crates and facade re-exports
- [x] Horizon HUD adapter compiled and tested against the real `horizon-hooks`
- [x] Native mod bundle staging script with registration calls
- [ ] Native `.hnmod` actually built against a Horizon instance
- [x] Camera/tile mixin with exact block map colours via `frame.rs`
- [ ] Waypoint/radar live sync
- [x] Control/key bridges and `native_key` mixin
- [x] Options page with all 115 bindings
- [ ] Packet channels + `PacketEffect` application
- [ ] Optional WorldMap protocol (needs an external source)
