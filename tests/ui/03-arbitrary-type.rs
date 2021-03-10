#![allow(unused)]
use bevy::prelude::*;

struct Foo;

#[bevycheck::system]
fn system(_: Foo) {}

fn main() {
    system.system();
}
