use bevy::{
    app::MainScheduleOrder, ecs::schedule::ScheduleLabel, prelude::*, window::PrimaryWindow,
};

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MouseUpdate;

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PostMouseUpdate;

pub struct CameraMousePlugin;

impl Plugin for CameraMousePlugin {
    fn build(&self, app: &mut App) {
        app.init_schedule(MouseUpdate)
            .init_schedule(PostMouseUpdate);

        app.world
            .resource_mut::<MainScheduleOrder>()
            .insert_after(PreUpdate, MouseUpdate);

        app.world
            .resource_mut::<MainScheduleOrder>()
            .insert_after(MouseUpdate, PostMouseUpdate);

        app.init_resource::<MyWorldCoords>()
            .add_event::<MouseCollisionEvent>()
            .add_systems(Startup, setup_camera)
            .add_systems(
                MouseUpdate,
                (update_cursor_position, mouse_box_collision_events).chain(),
            );
    }
}

#[derive(Component)]
pub struct MainCameraMarker;

#[derive(Component)]
pub struct MouseCollider(pub Transform);

#[derive(Resource, Default)]
struct MyWorldCoords(Vec2);

#[derive(Event, Debug)]
pub struct MouseCollisionEvent(pub Entity);

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), MainCameraMarker));
}

fn update_cursor_position(
    mut mycoords: ResMut<MyWorldCoords>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCameraMarker>>,
) {
    let (camera, camera_transform) = q_camera.single();
    let window = q_window.single();
    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        mycoords.0 = world_position;
        debug!("World coords: {}/{}", world_position.x, world_position.y)
    }
}

fn point_box_collision(point: Vec2, box_transform: Transform) -> bool {
    let Vec2 { x, y } = point;
    let Vec2 { x: b_x, y: b_y } = box_transform.translation.truncate();
    let Vec2 { x: w, y: h } = (box_transform.scale * 32.0).truncate();
    (b_x - (w / 2.) <= x) && (x <= b_x + (w / 2.)) && (b_y - (h / 2.) <= y) && (y <= b_y + (h / 2.))
}

fn mouse_box_collision_events(
    mouse_pos: Res<MyWorldCoords>,
    query: Query<(&MouseCollider, Entity)>,
    mut ev_mouse_collision: EventWriter<MouseCollisionEvent>,
) {
    let pos = mouse_pos.0;
    for (mouse_collider, entity) in &query {
        if point_box_collision(pos, mouse_collider.0) {
            ev_mouse_collision.send(MouseCollisionEvent(entity));
        }
    }
}
