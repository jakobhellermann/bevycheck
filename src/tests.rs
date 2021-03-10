#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/01-okay.rs");
    t.compile_fail("tests/ui/02-commands.rs");
    t.compile_fail("tests/ui/03-arbitrary-type.rs");
}
