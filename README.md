# xaerominimap-rust

A Rust rewrite of **Xaero's Minimap** (the Java/Fabric mod shipped as
`xaerominimap-fabric-26.3-26.6.2.jar` in this repository) for
**Minecraft: Java Edition 26.3**, following the game version of the same name:
[Horizon](https://playhorizon.dev) — *Minecraft: Java Edition 26.3 rewritten in
Rust* (repository `horizon-rust/horizon`).

The rewrite is a Cargo workspace of small crates:

| Crate                  | What it holds | Mirrors (Java packages) |
| ---------------------- | ------------- | ----------------------- |
| `xaerominimap-json`    | Dependency-free JSON tree/parser used instead of Gson/NBT by the serializers | Gson / NBT config resources |
| `xaerominimap-core`    | Domain data: waypoints, lifecycle handlers, minimap worlds/containers/state paths, tiles/chunks, object pool, biome blending, HUD presets (incl. built-ins), radar state/colours | `xaero/common/minimap`, `xaero/common/pool`, `xaero/hud/minimap/waypoint(set)`, `xaero/hud/minimap/world(container)`, `xaero/hud/minimap/radar/state|color`, `xaero/hud/preset` |
| `xaerominimap-settings`| Settings store, `xaero/hud/category` rules/data/serialization, entity radar categories, config-option UI type table, info-display data/codecs/IO and legacy config IO (`writeSettings` / `readSetting`) | `xaero/common/settings`, `xaero/common/config`, `xaero/hud/category`, `xaero/hud/minimap/radar/category`, `xaero/hud/minimap/info` |
| `xaerominimap-controls`| Key mappings, controller registration, held/press/release functions and tick handling | `xaero/hud/controls`, `xaero/hud/minimap/controls/key` |
| `xaerominimap-network` | Packet codecs (FriendlyByteBuf/NBT subset), registered packets, synced tracked players, OPAC party tracker, client world data, config sync codecs, server properties and server-mod detection | `xaero/hud/packet`, `xaero/common/server/level`, `xaero/common/server/mods`, `xaero/common/mods/pac/party`, `xaero/common/minimap/mcworld` |
| `xaerominimap-render`  | Rendering core: abstract canvas, the minimap renderer (map squares, waypoint markers, off-map arrows), map highlighters, `WaypointRenderInfo`, third-party waypoint origins, entity radar icon definitions, variant ids/caches, prerenderer flags and model field paths | `xaero/common/minimap/render`, `xaero/hud/minimap/render`, `xaero/hud/minimap/waypoint(set|render)`, `xaero/hud/minimap/radar/icon` |
| `xaerominimap`         | Facade: `MinimapSession` (world + settings + icon definitions/cache + radar categories + controls + third-party waypoints) and re-exports of the other crates | `xaero/hud/minimap/module`, `xaero/common/settings` (accessors) |
| `xaerominimap-horizon` | Horizon adapter: HUD `Layer`/`Canvas` bridge, runtime view settings, chunk tile grid, packet/control/key bridges, options page, camera/waypoint/radar state and hook registration against `horizon-hooks` | Horizon platform mod API (`horizon_hooks::hud`, `horizon_hooks::pages`, `horizon_hooks::Hooks`) |

See `crates/*/src` for the module-by-module mapping to the original mod and the
`CHANGELOG.md` for the full parity table. All data/logic subsystems present in
this JAR are ported. To actually run the minimap inside Horizon, the port is
bridged by `xaerominimap-horizon` and still needs a small engine-side native
mod (mixin bundle): camera/tile providers, live waypoint/radar sync, controls,
options pages and packet channels. See `HORIZON_INTEGRATION.md` for the exact
steps and source links; the WorldMap protocol additionally needs Xaero's
WorldMap JAR because its classes are not shipped here.

## Building

```sh
cargo build --workspace
cargo test --workspace
```

## Contributing

Contributions welcome: build and play-test the generated Horizon native bundle
with a real Horizon instance, add waypoint GUI screens, and document WorldMap
protocol support. See `HORIZON_INTEGRATION.md` and `CHANGELOG.md`. A native
bundle with camera/tiles/key mixins and the options page can be staged with:

```sh
python3 scripts/package_native_mod.py target/xaero-minimap-native
```
