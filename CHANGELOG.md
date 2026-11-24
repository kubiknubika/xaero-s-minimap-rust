# Changelog

All notable changes to this project are documented here. The project is a
rewrite of Xaero's Minimap 26.6.2 (Java/Fabric, for Minecraft 26.3) into Rust;
the section "Parity with the original mod" lists the areas and how far the
Rust port has gone.

## Unreleased

### Added

- `xaerominimap-core`: waypoint model (`Waypoint`, `WaypointColor`,
  `WaypointPurpose`, `WaypointVisibilityType`, `WaypointUtil`), waypoint sets
  (`WaypointSet`) and waypoint file IO (`WaypointIO`) mirroring
  `xaero/common/minimap/waypoints` and `xaero/hud/minimap/waypoint(set|io)`.
- `xaerominimap-core`: minimap world model (`MinimapWorld`, `MinimapWorldContainer`,
  `MinimapWorldState`, `MinimapDimensionHelper`) and the minimap tile model
  (`MinimapTile`: heights, per-pixel red/green/blue and chunk grid/slime chunk flags).
- `xaerominimap-settings`: settings store and the legacy config IO used by
  `writeSettings` / `readSetting` (`xaero/common/settings`, `xaero/common/config`).
- `xaerominimap-settings::categories` — the `xaero/hud/category` settings
  system: `ObjectCategory`, `ObjectCategorySetting`, `ObjectCategoryRule`
  (hard + list rules, include/exclude lists), `FilterObjectCategory`,
  `ObjectCategoryRuleResolver` and `ObjectCategoryData` with JSON
  serialization (`to_json` / `from_json`).
- `xaerominimap-core`: `RootConfig` + `MinimapWorldRootContainer.worlds`
  (`xaero/hud/minimap/world/container(config)`), `MinimapWorldManager`
  (`xaero/hud/minimap/world/MinimapWorldManager`).
- `xaerominimap-render`: `WaypointRenderInfo` (`xaero/hud/minimap/waypoint`).
- `xaerominimap`: facade crate with `MinimapSession` (current world + settings)
  and re-exports of the other crates (`xaero/hud/minimap/module`).

## Parity with the original mod

| Area | Status |
| ---- | ------ |
| Waypoints model, sets, IO, sorting | Ported (`xaerominimap-core::waypoints`) |
| Minimap worlds / containers / states | Ported (`xaerominimap-core::world`) |
| Minimap worlds root containers + `RootConfig` | Ported (`xaerominimap-core::world`) |
| `MinimapWorldManager` | Ported (`xaerominimap-core::world`) |
| Minimap tiles (heights, colours, grids) | Ported (`xaerominimap-core::tile`) |
| Settings categories system (rules, filters, serialization) | Ported (`xaerominimap-settings::categories`) |
| Settings / config load & save | Ported (`xaerominimap-settings`) |
| Map rendering (squares, markers, arrows) | Ported (`xaerominimap-render`) |
| Waypoint render info | Ported (`xaerominimap-render::waypoint_render`) |
| Entity radar state/lists | Ported (`xaerominimap-core::radar`) |
| Session state (`MinimapSession`) + facade re-exports | Ported (`xaerominimap`) |
| Entity radar icons (models, sprites, custom forms) | Not ported yet |
| GUI (all option screens, the settings UI) | Not ported yet |
| Controls (key mappings, keybind functions) | Not ported yet |
| Server packets, WorldMap protocol | Not ported yet |
| Third-party waypoint managers (Waystones, Fwaystones, …) | Not ported yet |
