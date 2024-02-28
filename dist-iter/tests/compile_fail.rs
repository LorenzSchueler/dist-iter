// include as module for `cargo fmt` and `cargo clippy` to work
//mod compile_fail_tests;

#[test]
fn compile_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail_tests/method_macro_mismatch/*.rs");
    t.compile_fail("tests/compile_fail_tests/task_macro_wrong_args/*.rs");
}
