use bevy::prelude::*;

fn main() {
    some_system.system();
}

struct Player;
struct SpawnPlayer;
struct Test<T>(T);
#[bevycheck::system]
fn some_system(query: Query<Entity, Or<(Player, SpawnPlayer)>>) {}
