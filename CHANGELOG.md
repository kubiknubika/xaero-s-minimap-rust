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
- `xaerominimap-json`: the dependency-free JSON parser was moved out of
  `xaerominimap-settings::categories` so the render crate can read resource
  files with it (the old `categories::json` path still works as a re-export).
- `xaerominimap-render::radar_icons`: entity radar icon definition resources
  (`xaero/hud/minimap/radar/icon/definition`): `IconFormType`, `IconForm`,
  `ModelConfig`, `IconDefinition` (JSON loading, `construct`/`constructForm`,
  variant fallbacks) and `IconDefinitionManager` with the 13 built-in
  definition JSONs from the mod jar. The facade session now loads them too.
- `xaerominimap-render::radar_icon_cache`: entity radar icon cache and variant
  ids (`xaero/hud/minimap/radar/icon/cache` and the built-in variant resolver):
  `IconVariant` with the Java `toString()` formats, `EntityVariantData` /
  `VariantKind`, `IconKey`, `IconArmor` + `ArmorHandler`, `EntityIconCache` and
  `IconCache`. The session now owns the cache and can look up a form for a
  typed variant.
- `xaerominimap-settings::radar_categories`: entity radar category system
  (`xaero/hud/minimap/radar/category`): the 15 category settings with their
  Java defaults, hard rules and list rule types, `RadarEntityInfo` predicates,
  `RadarCategory` trees with include/exclude lists and setting overrides,
  `RadarCategoryData` JSON serialization and `RadarCategoryManager` with the
  default category tree. The session can now select a category and its
  effective settings for an entity.
- `xaerominimap-controls`: key mappings and controller ticking
  (`xaero/hud/controls` and `xaero/hud/minimap/controls/key`): the 21
  `MinimapKeyMappings` definitions with their default key codes and the
  `xaerominimap:controls` category, the 20 function registrations of
  `MinimapKeyMappingFunctions`, `KeyMappingController` /
  `KeyMappingControllerManager` with the Java duplicate-registration errors,
  `KeyMappingTickHandler` with its extra-press/release/press passes and the
  held vs. repeated function semantics. Input is abstracted through
  `KeyStateSource` and delivered as `ControlEvent`s; `MinimapSession` now owns
  a `ControlsRegister` and exposes `tick_controls`.
- `xaerominimap-render::radar_icon_prerender`: the engine-independent half of
  the radar icon prerenderers and model field resolvers
  (`xaero/hud/minimap/radar/icon/creator/render`): `FormPrerenderer` with the
  flags and failure icons of the six `RadarIconFormTypes`, `ModelFieldResolver`
  with the Java `field[index]` syntax and `modelPartsFields`/`modelRootPath`
  selection, `FieldReferenceType`, `ModelRenderTrace` and
  `ModelPartRenderTrace`. Engine objects are represented by opaque ids so a
  Horizon renderer can feed its own model handles.
- `xaerominimap-network`: the five packets registered by
  `MinimapPacketRegister` (ids 0–4) with id-prefixed binary encoding, a
  `FriendlyByteBuf`-style codec and the NBT subset used by the packets
  (`xaero/hud/packet`, `xaero/common/server/level`): `LevelMapProperties`
  (including the `xaeromap.txt` `id:<n>` format), `HandshakePacket` with
  `NETWORK_COMPATIBILITY = 3`, `TrackedPlayerPacket` with both UUID encodings,
  `TrackerResetPacket` and `RulesPacket` with the `cm`/`ncm`/`r` keys. Packet
  handlers are represented by `PacketEffect`; the facade re-exports the crate
  as `network`.
- `xaerominimap-render::third_party_waypoints`: mod-supplied waypoint origins
  (`xaero/hud/minimap/waypoint/thirdparty`): `ThirdPartyWaypoints` stores
  waypoints by id and by stable slot, keeps render-info overrides and an
  optional enabled-state getter; `ThirdPartyWaypointManager` registers origins
  in order and clears them like the Java manager. `Waypoint` gained the
  `third_party_origin` field from `setThirdPartyOrigin(Identifier)`, and
  `MinimapSession` now owns the manager. Saving the root config after removals
  stays in the game glue.
- `xaerominimap-core::waypoint_handlers`: the deterministic parts of
  `xaero/hud/minimap/waypoint` lifecycle handlers: `DeathpointHandler`
  (old-deathpoint demotion and creation with the Java translation keys/colors),
  `TemporaryWaypointHandler` (dimension scaling and top/bottom insertion),
  `WaypointCollector` (all-sets/current-set plus third-party/custom waypoints,
  skipping deleted ones) and `DestinationHandler` (reach radius 4 and the
  5000 ms deathpoint grace period). Player access, GUI gates and world saving
  remain game-glue inputs.
- `xaerominimap-network::server_mods`: server-side integration detection
  (`xaero/common/server/mods`): `ServerModSupport::detect` mirrors the four
  `Class.forName` probes of `SupportServerMods.check` (FTB Teams, Argonauts,
  OPAC, WorldMap), `SupportWorldMap.supportsTrackedPlayers()` uses the Java
  boundary 22, `SyncedTrackerSystem` selects the first available team system,
  and `is_position_sync_allowed` reproduces the OPAC party/mutual-ally share
  checks.
- `xaerominimap-network::tracked_players`: synced tracked-player state
  (`xaero/common/server/radar/tracker`, `xaero/hud/minimap/player/tracker/synced`):
  `SyncedTrackedPlayer` with `matchesEnough`/reader accessors,
  `ClientSyncedTrackedPlayerManager` with update/remove/reset and direct
  application of `PacketEffect`s from the tracked-player packets, and
  `SyncedPlayerTrackerSystemManager` for the registered team systems.
- `xaerominimap-render::highlight`: `xaero/common/minimap/highlight` data
  layer with the 16×16 result store (`setResult` index `y << 4 | x`), ARGB
  channel blending, `ChunkHighlighter.getSideBlend` and the highlighter
  registry that freezes on `end()`.
- `xaerominimap-core`: `MinimapWorldState` now mirrors all Java fields,
  including the four outdated root-container slots, the ignore-case cache,
  the `IllegalStateException` guards (returned as `Result`) and
  `getCurrentContainerPath`/`getCurrentRootContainerPath`; `XaeroPath` gained
  `parent`/`root`/`resolve`/`isSubOf`/`getSubPath`. `MinimapWorldManager`
  stores the server level id from `MinimapWorldStateUpdater.onServerLevelId`
  and implements `hasServerLevelId` with the config opt-out.
- `xaerominimap-settings::config_ui`: the 115 `registerUIType` calls of
  `MinimapConfigOptionUIRegister.registerAll` extracted into a static table
  (`UI_BINDINGS`), with `ConfigUiType` (7 control kinds),
  `ConfigUiBinding` and `ConfigUiTypeManager` (option → control lookup).
  This is the data layer of the options GUI; widget construction stays in
  the game client.
- `xaerominimap-settings::info_display` and `xaerominimap-network::config_sync`:
  info-display config data (`InfoDisplayConfigData`,
  `InfoDisplayManagerConfigData`, builder, `EMPTY` equivalent) and the
  `MinimapValueSyncCodecs.INFO_DISPLAY_CONFIG` NBT codec (`o` id list, `c`
  config compounds with `s`/`b`/`t` keys). NBT gained `TAG_Int` and `TAG_List`
  support for this codec.
- `xaerominimap-core::{biome_blend, optimized_math}`: biome colour blending
  from `BiomeBlendCalculator` (1- or 3×3 offsets, per-channel averaging and
  forced opaque alpha) and `OptimizedMath.myFloor`.
- `xaerominimap-core::chunk`: data side of `MinimapChunk` — 4×4 tile grid,
  reset/recycle, change and upload flags, five refresh slots and the
  `getLevelToRefresh(int)` cyclic scan.
- `xaerominimap-settings::info_display_io`: legacy `.txt` codec of
  `InfoDisplayIO` (`infoDisplayOrder:` and `infoDisplay:` lines, `-` for
  absent values) with encode/decode round-trip support.
- `xaerominimap-settings::builtin_info_displays`: data descriptors for the 14
  `BuiltInInfoDisplays` (id, translation key, default state, codec, widget
  kind, legacy getter) plus the `InfoDisplayManager.adaptOrder` algorithm.
  Compilers and widgets remain render/GUI glue.
- `xaerominimap-settings::info_display_codec`: `InfoDisplayCommonStateCodecs`
  boolean/integer state codecs (strict `true` parse, `Integer.parseInt`,
  codec max lengths 5 and 11).
- `xaerominimap-core::preset`: data side of `xaero/hud/preset` —
  `HudPreset`/`ModulePreset`/`HudPresetManager` and the placement action with
  the seven `ModuleTransform` fields, including `apply`/`confirm`/`cancel`/
  `applyAndConfirm` semantics (modules identified by string id).
- `xaerominimap-core::preset::builtin_presets`: the four
  `xaero/hud/xminimap/preset/BuiltInHudPresets` corner presets and
  `addAll(HudPresetManager)` equivalent.
- `xaerominimap-core::radar_color`: 16-value `RadarColor` enum (format chars,
  ARGB hex, translation keys, `fromIndex`/random-by-fraction) and the
  `RadarColorHelper` colour resolution/height-fade calculations.
- `xaerominimap-network::world_data`: data side of
  `MinimapClientWorldData`/`MinimapClientWorldDataHelper` — dimension shadow
  tints, server-mod network version, server level id, lazily defaulted synced
  rules and a dimension-keyed world registry.
- `xaerominimap-settings::categories`: `FilterObjectCategoryData` with the
  `h`/`i`/`m`/`l`/`e` serialized fields, `ExcludeListMode.name()`/valueOf
  helpers and a JSON round trip; re-exported by the facade.
- `xaerominimap-core::pool`: `xaero/common/pool` `PoolUnit`/`MapPool` with typed
  `PoolArg` values, synchronized LIFO recycling and a `maxSize` guard; tests
  cover reuse and cross-thread access.
- `xaerominimap-network::party`: `OPACPlayerTrackerSystem` /
  `OPACTrackedPlayerReader` with the ally iterator, dimension-key cache and
  `PartyTracker` interface; re-exported by the facade.
- `xaerominimap-horizon`: first runnable Horizon bridge — HUD canvas adapter,
  `MinimapLayer`, runtime view options (`MinimapViewConfig`), `ChunkColorGrid`,
  `PacketBridge` for the five packet effects, hook registration and 13 tests
  against the real pinned `horizon-hooks` crate.
- `xaerominimap-horizon::settings_page`: Horizon options page built from all
  115 `UI_BINDINGS`, with HUD toggles mirrored into `MinimapState`;
  `horizon_key_code` maps `key.keyboard.*` names to LWJGL key codes.
- `xaerominimap-horizon::ControlBridge`: applies `ControlEvent`s from
  `xaerominimap-controls` to map/radar toggles, zoom, enlarge-map and cave mode;
  `HorizonKeyState`/`HorizonControls` feed key codes through the real
  `ControlsRegister.tick(...)`; `MinimapState::sync_waypoints` converts ported
  waypoints to HUD markers.
- `scripts/package_native_mod.py`: stages a Horizon native-mod bundle from the
  workspace (port crates as `[[crate]]`s plus an `[api]` trait mod that
  registers the layer/hooks, a `frame.rs` HEAD injection that bakes chunk tiles
  from `native_client_chunks`/`native_client_biomes`, a `key.rs` `native_key`
  injection, and `scripts/test_package_native_mod.py` smoke test. See
  `HORIZON_INTEGRATION.md`.

### Fixed

- Workspace member paths, several type/borrow errors and a wrong bounds
  assertion in the raster canvas test: `cargo test --workspace` is green again.

## Parity with the original mod

| Area | Status |
| ---- | ------ |
| Waypoints model, sets, IO, sorting | Ported (`xaerominimap-core::waypoints`) |
| Waypoint lifecycle handlers (deathpoint/temporary/destination/collector) | Ported (`xaerominimap-core::waypoint_handlers`) |
| Minimap worlds / containers / states | Ported (`xaerominimap-core::world`) |
| Minimap worlds root containers + `RootConfig` | Ported (`xaerominimap-core::world`) |
| `MinimapWorldManager` | Ported (`xaerominimap-core::world`) |
| Minimap tiles (heights, colours, grids) | Ported (`xaerominimap-core::tile`) |
| Biome blend calculation / optimized floor | Ported (`xaerominimap-core::{biome_blend, optimized_math}`) |
| Minimap chunk data (tile grid, refresh slots) | Ported (`xaerominimap-core::chunk`) |
| HUD preset data and placement action | Ported (`xaerominimap-core::preset`) |
| Settings categories system (rules, filters, serialization, filter category data) | Ported (`xaerominimap-settings::categories`) |
| Settings / config load & save | Ported (`xaerominimap-settings`) |
| Map rendering (squares, markers, arrows) | Ported (`xaerominimap-render`) |
| Waypoint render info | Ported (`xaerominimap-render::waypoint_render`) |
| Entity radar state/lists | Ported (`xaerominimap-core::radar`) |
| Radar colours and height fading | Ported (`xaerominimap-core::radar_color`) |
| Entity radar categories, rules and config JSON | Ported (`xaerominimap-settings::radar_categories`) |
| Session state (`MinimapSession`) + facade re-exports | Ported (`xaerominimap`) |
| Entity radar icon definitions (JSON, forms, variants) | Ported (`xaerominimap-render::radar_icons`) |
| Entity radar icon cache, variant ids and armour keys | Ported (`xaerominimap-render::radar_icon_cache`) |
| Entity radar icon prerenderers and model field resolvers | Ported (data side: `xaerominimap-render::radar_icon_prerender`) |
| Config-option UI type table (`MinimapConfigOptionUIRegister`) | Ported (`xaerominimap-settings::config_ui`) |
| Info-display config data, NBT sync codec, text IO, state codecs and built-in display table | Ported (`xaerominimap-settings::info_display*`/`builtin_info_displays`, `xaerominimap-network::config_sync`) |
| GUI (widgets and option screens) | Not ported yet |
| Controls (key mappings, keybind functions) | Ported (`xaerominimap-controls`) |
| Server packets (five registered packet codecs + `xaeromap.txt`) | Ported (`xaerominimap-network`) |
| Synced tracked players (state + packet effects) | Ported (`xaerominimap-network::tracked_players`) |
| Map highlighter registry and colour blending | Ported (`xaerominimap-render::highlight`) |
| Server mod detection (FTB Teams, Argonauts, OPAC, WorldMap) | Ported (`xaerominimap-network::server_mods`) |
| Client world data (shadows, network version, synced rules) | Ported (`xaerominimap-network::world_data`) |
| WorldMap protocol (map image/region exchange) | Not ported yet — not present in this JAR |
| Third-party waypoint origins/manager (Waystones, Fwaystones, …) | Ported (`xaerominimap-render::third_party_waypoints`) |

**Port status (2026-11-24):** all data/logic subsystems available in
`xaerominimap-fabric-26.3-26.6.2.jar` are ported, tested and re-exported by the
facade. The first Horizon runtime adapter (`xaerominimap-horizon`) is in place:
it draws the ported map through `horizon_hooks::hud::Layer`/`Canvas` and keeps
camera, waypoint and radar state. The remaining rows are engine glue inside a
Horizon native mod (camera/tile mixin, controls, options pages, packet
channels) and the WorldMap protocol, whose classes are not part of this JAR.
`cargo test --workspace` is green (174 tests). Building the launcher itself is
the only blocked step in this sandbox: fetching crates.io dependencies fails
with an SSL connect error. See `HORIZON_INTEGRATION.md`.
| Horizon HUD adapter (canvas bridge, minimap layer, state, hooks) | Ported (`xaerominimap-horizon`) |
| Object pool (`MapPool`/`PoolUnit`) | Ported (`xaerominimap-core::pool`) |
| OPAC party tracker (`OPACPlayerTrackerSystem`/`OPACTrackedPlayerReader`) | Ported (`xaerominimap-network::party`) |
| Horizon runtime view settings, tile grid, packet/control/key bridges, settings page, waypoint sync | Ported (`xaerominimap-horizon::{MinimapViewConfig,ChunkColorGrid,PacketBridge,ControlBridge,HorizonControls,MinimapSettingsPage}`) |
| Horizon native-mod bundle staging script + camera/key mixins | Added (`scripts/package_native_mod.py`, `frame.rs`, `key.rs`) |
