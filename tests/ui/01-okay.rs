#![allow(unused)]
use bevy::prelude::*;

struct Foo;

#[bevycheck::system]
fn system(_: Commands, _: Res<Foo>) {}

fn main() {
    system.system();
}
