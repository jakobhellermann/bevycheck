#![allow(unused)]
use bevy::prelude::*;

struct Foo;
struct Bar;

#[bevycheck::system]
fn system(
    _: QuerySet<(
        Query<Transform, With<Foo>>,
        Query<&mut Transform, Without<Foo>>,
    )>,
) {
}

fn main() {
    system.system();
}
