# bevycheck

Bevycheck helps debug bevy errors by adding procedural macros which aid the compiler in emitting better compilation errors.

If you get an error like ``the trait `IntoSystem<(), (), _>` is not implemented for fn item `for<'a> fn(bevy::prelude::ResMut<'a, [type error]>) {system}``, simply add `#[bevycheck::system]` to your function, and a more helpful error messages should appear:


```rust
#[bevycheck::system]
fn system(commands: &mut Commands, query: Query<(Entity, &GlobalTransform)>) {
    // ...
}
```

## How does it work?

It works by replacing
```rust
fn system(commands: &mut Commands, query: Query<(Entity, &GlobalTransform)>) {
  // ...
}
```
with
```rust
fn system() {
  assert_is_system_param::<&mut Commands>();
  assert_is_system_param::<Query<(Entity, &GlobalTransform)>>();
  panic!("remove bevycheck before running");
}
```
That way, without parameters the system is a valid system and the `add_system` call doesn't error anymore, and by asserting that each specific parameter must be a valid system param,
rust can figure out which one's the culprit and print a more directed error message.


## Bevy support table

|bevy|bevycheck|
|---|---|
|0.10|0.5|
|0.9|0.4|
|0.7|0.3|
|0.6|0.2|
|0.5|0.1|
