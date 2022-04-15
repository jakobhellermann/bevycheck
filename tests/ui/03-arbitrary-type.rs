#![allow(unused)]
use bevy::prelude::*;

struct Foo;

#[bevycheck::system]
fn system(_: Foo) {}

fn main() {
    IntoSystem::into_system(system);
}
