//! End-to-end test of the facade: session, world, waypoints and the
//! waypoint IO round trip.

use xaerominimap::{
    DimensionKey, MinimapSession, MinimapWorld, Waypoint, WaypointColor, WaypointIO, MinimapWorldContainer,
};

#[test]
fn session_world_waypoint_io_round_trip() {
    let mut session = MinimapSession::new();
    let world = session.ensure_world(DimensionKey::overworld());
    world.add_waypoint_set("Set1");
    world
        .get_waypoint_set_mut("Set1")
        .unwrap()
        .add(Waypoint::new(1, 2, 3, "test", "").with_color(WaypointColor::Red));
    assert_eq!(session.world().unwrap().get_set_count(), 1);

    // Waypoint IO round trip into a fresh world.
    let mut text: Vec<u8> = Vec::new();
    WaypointIO::new()
        .save_waypoints(session.world().unwrap(), &mut text)
        .unwrap();
    let mut world2 = MinimapWorld::new("copy", DimensionKey::overworld());
    let added = WaypointIO::new()
        .load_waypoints(&mut world2, std::str::from_utf8(&text).unwrap())
        .unwrap();
    assert_eq!(added, 1);
    let set = world2.get_waypoint_set("Set1").unwrap();
    assert_eq!(set.get_waypoints().len(), 1);
    assert_eq!(set.get_waypoints()[0].name, "test");
    assert_eq!(set.get_waypoints()[0].color, Some(WaypointColor::Red));
}

#[test]
fn session_settings_and_world_dimensions() {
    let mut session = MinimapSession::new();
    session.settings_mut().set("minimapScale", 0.5f32);
    assert_eq!(session.settings().float("minimapScale", 0.0), 0.5);
    session.ensure_world(DimensionKey::the_nether());
    assert_eq!(
        session.world().unwrap().get_dim_id(),
        &DimensionKey::the_nether()
    );
}

#[test]
fn container_holds_worlds() {
    let mut container = MinimapWorldContainer::new();
    container.add_world(MinimapWorld::new("w1", DimensionKey::overworld()));
    assert_eq!(container.get_worlds().count(), 1);
}
