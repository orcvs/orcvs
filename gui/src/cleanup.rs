use bevy::prelude::*;
#[derive(Component)]
pub struct LevelUnload;
#[derive(Component)]
pub struct MenuClose;

fn cleanup_system<T: Component>(mut commands: Commands, q: Query<Entity, With<T>>) {
    for e in q.iter() {
        commands.entity(e).despawn_recursive();
    }
}
