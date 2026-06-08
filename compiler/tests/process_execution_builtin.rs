use php_compiler::run_source;

#[test]
fn shell_exec_captures_stdout_and_empty_output_as_null() {
    let execution = run_source(
        r#"<?php
var_dump(shell_exec('printf phpc-shell'));
var_dump(shell_exec('true'));
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "string(10) \"phpc-shell\"\nNULL\n");
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}
