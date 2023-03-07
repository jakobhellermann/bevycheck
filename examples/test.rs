#![allow(unused)]
use bevy::prelude::*;

#[bevycheck::system]
fn system(_: &mut Commands) {}

fn main() {
    IntoSystem::into_system(system);
}

