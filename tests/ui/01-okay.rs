#![allow(unused)]
use bevy::prelude::*;

#[derive(Resource)]
struct Foo;

#[bevycheck::system]
fn system(_: Commands, _: Res<Foo>, _: (EventWriter<Foo>, EventReader<Foo>)) {}

fn main() {
    IntoSystem::into_system(system);
}
