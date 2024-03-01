// include as module for `cargo fmt` and `cargo clippy` to work
//mod compile_fail_tests;

#[test]
fn compile_fail() {
    let t = trybuild2::TestCases::new();
    t.pass("tests/compile_fail_tests/empty.rs"); // by having a pass test all tests get build and not just type checked (`cargo build` instead of `cargo check`)

    t.compile_fail_check_sub(
        "tests/compile_fail_tests/chunk_size/chunk_size_zero.rs",
        "the evaluated program panicked at 'CHUNK_SIZE must be greater than 0'",
    );

    t.compile_fail_check_sub(
        "tests/compile_fail_tests/method_macro_mismatch/map_filter_map_syntax.rs",
        "^^^ no rules expected this token in macro call",
    );
    t.compile_fail_check_sub(
        "tests/compile_fail_tests/method_macro_mismatch/map_filter_filter_syntax.rs",
        "expected `MapTask<_>`, found `FilterTask<ThisTask>`",
    );

    t.compile_fail_check_sub(
        "tests/compile_fail_tests/task_macro_misuse/map_chunk_into_iter_missing_chunk_size.rs",
        "note: while trying to match `INPUT_CHUNK_SIZE`",
    );
    t.compile_fail_check_sub(
        "tests/compile_fail_tests/task_macro_misuse/map_chunk_into_iter_missing_input_type.rs",
        "note: while trying to match `:`",
    );
    t.compile_fail_check_sub(
        "tests/compile_fail_tests/task_macro_misuse/map_chunk_into_iter_missing_return_type.rs",
        "note: while trying to match `->`",
    );
    t.compile_fail_check_sub(
        "tests/compile_fail_tests/task_macro_misuse/map_chunk_into_iter_missing_return_value.rs",
        "`()` is not an iterator",
    );
    t.compile_fail_check_sub(
        "tests/compile_fail_tests/task_macro_misuse/map_chunk_into_iter_wrong_input_type.rs",
        "expected `i32`, found `u32`",
    );
    t.compile_fail_check_sub(
        "tests/compile_fail_tests/task_macro_misuse/map_chunk_into_iter_wrong_return_type_iter_item.rs",
        "expected `i32`, found `u32`"
    );
    t.compile_fail_check_sub(
        "tests/compile_fail_tests/task_macro_misuse/map_chunk_into_iter_wrong_return_type_not_iter.rs",
        "`{integer}` is not an iterator"
    );
    t.compile_fail_check_sub(
        "tests/compile_fail_tests/task_macro_misuse/map_chunk_mut_buffer_missing_chunk_size.rs",
        "note: while trying to match `INPUT_CHUNK_SIZE`",
    );
    t.compile_fail_check_sub(
        "tests/compile_fail_tests/task_macro_misuse/map_chunk_mut_buffer_missing_input_type.rs",
        "note: while trying to match `:`",
    );
    t.compile_fail_check_sub(
        "tests/compile_fail_tests/task_macro_misuse/map_chunk_mut_buffer_with_return_value.rs",
        "expected `()`, found `&mut [i32]`",
    );
    t.compile_fail_check_sub(
        "tests/compile_fail_tests/task_macro_misuse/map_missing_chunk_size.rs",
        "note: while trying to match `CHUNK_SIZE`",
    );
    t.compile_fail_check_sub(
        "tests/compile_fail_tests/task_macro_misuse/map_missing_input_type.rs",
        "note: while trying to match `:`",
    );
    t.compile_fail_check_sub(
        "tests/compile_fail_tests/task_macro_misuse/map_missing_return_type.rs",
        "note: while trying to match `->`",
    );
    t.compile_fail_check_sub(
        "tests/compile_fail_tests/task_macro_misuse/map_missing_return_value.rs",
        "expected `()`, found `i32`",
    );
    t.compile_fail_check_sub(
        "tests/compile_fail_tests/task_macro_misuse/map_wrong_input_type.rs",
        "expected `i32`, found `u32`",
    );
    t.compile_fail_check_sub(
        "tests/compile_fail_tests/task_macro_misuse/map_wrong_return_type.rs",
        "expected `i32`, found `u32`",
    );

    t.compile_fail_check_sub(
        "tests/compile_fail_tests/setup/missing_setup_fn.rs",
        "cannot find function `setup` in this scope",
    );
    t.compile_fail_check_sub(
        "tests/compile_fail_tests/setup/setup_fn_with_args.rs",
        "note: expected fn pointer `fn()`",
    );
}
