use php_compiler::run_source;

#[test]
fn array_walk_too_many_arguments_are_catchable_argument_count_errors() {
    let source = r#"<?php
$items = [1];

function needs_three($value, $key, $userdata) {}

try {
    array_walk($items, "needs_three");
} catch (Throwable $e) {
    echo $e->getMessage(), "\n";
}

try {
    array_walk_recursive($items, "needs_three");
} catch (Throwable $e) {
    echo $e->getMessage(), "\n";
}

try {
    array_walk($items, "strval", "userdata", "extra");
} catch (Throwable $e) {
    echo $e->getMessage(), "\n";
}

try {
    array_walk_recursive($items, "strval", "userdata", "extra");
} catch (Throwable $e) {
    echo $e->getMessage(), "\n";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Too few arguments to function needs_three(), 2 passed and exactly 3 expected\nToo few arguments to function needs_three(), 2 passed and exactly 3 expected\narray_walk() expects at most 3 arguments, 4 given\narray_walk_recursive() expects at most 3 arguments, 4 given\n"
    );
    assert_eq!(execution.exit_code, 0);
}
