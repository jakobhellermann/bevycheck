#![allow(unused)]
use bevy::prelude::*;

struct Foo;

#[bevycheck::system]
fn system(_: Query<&Foo, With<Added<Foo>>>) {}

fn main() {
    App::new().add_system(system).run();
}
