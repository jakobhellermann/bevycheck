#![allow(unused)]
use bevy::prelude::*;

struct Foo;
struct Bar;

#[bevycheck::system]
fn system(_: Query<(Entity, Foo, Option<&Foo>, Flags<Foo>), Bar>) {}

fn main() {
    system.system();
}
