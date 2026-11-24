# xaerominimap-rust

A Rust rewrite of **Xaero's Minimap** (the Java/Fabric mod shipped as
`xaerominimap-fabric-26.3-26.6.2.jar` in this repository) for
**Minecraft: Java Edition 26.3**, following the game version of the same name:
[Horizon](https://playhorizon.dev) — *Minecraft: Java Edition 26.3 rewritten in
Rust* (repository `horizon-rust/horizon`).

The rewrite is a Cargo workspace of small crates:

| Crate                  | What it holds | Mirrors (Java packages) |
| ---------------------- | ------------- | ----------------------- |
| `xaerominimap-core`    | Domain data: waypoints, waypoint sets and IO, minimap worlds and containers, minimap tiles and radar state | `xaero/common/minimap`, `xaero/hud/minimap/waypoint(set)`, `xaero/hud/minimap/world(container)`, `xaero/hud/minimap/radar/state` |
| `xaerominimap-settings`| Settings store, the `xaero/hud/category` settings system and legacy config IO (`writeSettings` / `readSetting`) | `xaero/common/settings`, `xaero/common/config`, `xaero/hud/category` |
| `xaerominimap-render`  | Rendering core: abstract canvas, the minimap renderer (map squares, waypoint markers, off-map arrows) and `WaypointRenderInfo` | `xaero/common/minimap/render`, `xaero/hud/minimap/render`, `xaero/hud/minimap/waypoint(set|render)` |
| `xaerominimap`         | Facade: `MinimapSession` (current world + settings) and re-exports of the other crates | `xaero/hud/minimap/module`, `xaero/common/settings` (accessors) |

See `crates/*/src` for the module-by-module mapping to the original mod and the
`CHANGELOG.md` for what is implemented so far.

## Building

```sh
cargo build --workspace
cargo test --workspace
```

## Contributing

Contributions welcome: port more subsystems (e.g. the GUI option panels, the
entity radar icons, the shared-waypoint manager), write tests, fix
documentation. See `CHANGELOG.md` for the planned module order.
