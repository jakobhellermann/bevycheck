#![allow(unused)]
use bevy::prelude::*;

#[derive(Component)]
struct Component;

#[derive(Resource)]
struct Resource;

// #[bevycheck::system]
fn system(commands: Commands, query: Query<&Component>) {}

fn main() {
    App::new().add_system(system).run();
}
