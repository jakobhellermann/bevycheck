#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/01-okay.rs");
    t.compile_fail("tests/ui/02-commands.rs");
    t.compile_fail("tests/ui/03-arbitrary-type.rs");
    t.compile_fail("tests/ui/04-query-invalid.rs");
    t.compile_fail("tests/ui/05-too-many-params.rs");
    t.compile_fail("tests/ui/06-query-set.rs");
    t.compile_fail("tests/ui/07-query-with-added.rs");
}
