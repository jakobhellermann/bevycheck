# bevycheck

Bevycheck helps debug bevy errors by adding procedural macros which display nice error messages.

### Disclaimer
`bevycheck` can't figure out all valid system parameters, for example, custom types with `#[derive(SystemParam)]` won't work. If you find a type that should work but get's reported, please file an issue.

This crate is meant as a temporary measure to figure out why something doesn't compile, but shouldn't be kept in the final code.

## Invalid Systems

If you get an error like 
- `no method named "system" found for fn item "for<'r, 's> fn(Query<'r, &'s Component>, Commands) {my_system}" in the current scope "my_system" is a function, perhaps you wish to call it`
- `the trait bound "Component: WorldQuery" is not satisfied the trait "WorldQuery" is not implemented for "Component"`

simply add `#[bevycheck::system]` to your function and helpful error messages will appear.


```rust
#[bevycheck::system]
fn system(commands: &mut Commands, query: Query<(Entity, GlobalTransform)>) {
    // ...
}
```


```rust
error: invalid system parameter
 --> examples/test.rs:4:21
  |
4 | fn system(commands: &mut Commands, query: Query<(Entity, GlobalTransform)>) {}
  |                     ^^^^^^^^^^^^^
  |
  = help: use `mut commands: Commands`

error: invalid query parameter
 --> examples/test.rs:4:58
  |
4 | fn system(commands: &mut Commands, query: Query<(Entity, GlobalTransform)>) {}
  |                                                          ^^^^^^^^^^^^^^^
  |
  = note: `GlobalTransform` is not a valid query type
  = help: if you want to query for a resource, use `&GlobalTransform` or `&mut GlobalTransform`

error: aborting due to 2 previous errors

error: could not compile `bevycheck`

To learn more, run the command again with --verbose.
```

## Bevy support table

|bevy|beyvcheck|
|---|---|
|0.5|0.1|
