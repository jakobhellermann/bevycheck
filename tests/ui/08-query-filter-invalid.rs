use bevy::prelude::*;

fn main() {
    IntoSystem::into_system(some_system);
}

struct Player;
struct SpawnPlayer;
struct Test<T>(T);
#[bevycheck::system]
fn some_system(query: Query<Entity, Or<(Player, SpawnPlayer)>>) {}
