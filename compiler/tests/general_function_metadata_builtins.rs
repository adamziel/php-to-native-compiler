use php_compiler::error::Phase;
use php_compiler::run_source;

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn connection_and_pid_metadata_cover_general_function_rows() {
    let execution = run_source(
        r#"<?php
echo connection_aborted(), "|", connection_status(), "|";
echo connection_status() == CONNECTION_NORMAL ? "normal" : "bad";
echo "|", CONNECTION_ABORTED, ":", CONNECTION_TIMEOUT, "\n";
$pid = getmypid();
echo get_debug_type($pid), ":", ($pid > 0 ? "positive" : "bad"), "\n";
foreach (["connection_aborted", "connection_status", "getmypid"] as $name) {
    echo function_exists($name) ? "1" : "0";
    echo is_callable($name) ? "1" : "0";
}
echo "\n";
$call = "connection_status";
echo $call();
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "0|0|normal|1:2\nint:positive\n111111\n0");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn get_loaded_extensions_reports_bounded_registry() {
    let execution = run_source(
        r#"<?php
$extensions = get_loaded_extensions();
var_dump($extensions);
foreach ($extensions as $extension) {
    echo extension_loaded($extension) ? "1" : "0";
}
echo "\n";
$call = "get_loaded_extensions";
echo count($call()), "\n";
echo function_exists("get_loaded_extensions") ? "exists" : "missing";
echo "|";
echo is_callable("get_loaded_extensions") ? "callable" : "not-callable";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "array(5) {\n",
            "  [0]=>\n",
            "  string(6) \"bcmath\"\n",
            "  [1]=>\n",
            "  string(4) \"json\"\n",
            "  [2]=>\n",
            "  string(4) \"hash\"\n",
            "  [3]=>\n",
            "  string(3) \"PDO\"\n",
            "  [4]=>\n",
            "  string(9) \"pdo_mysql\"\n",
            "}\n",
            "11111\n",
            "5\n",
            "exists|callable",
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn metadata_helpers_reject_arguments_outside_current_subset() {
    let pid_error = runtime_error("<?php\ngetmypid(1);\n");
    assert_eq!(pid_error.line, 2);
    assert_eq!(
        pid_error.message,
        "arity mismatch for getmypid(): expected 0 argument(s), got 1"
    );

    let extensions_error = runtime_error("<?php\nget_loaded_extensions(false);\n");
    assert_eq!(extensions_error.line, 2);
    assert_eq!(
        extensions_error.message,
        "arity mismatch for get_loaded_extensions(): expected 0 argument(s), got 1"
    );

    let connection_error = runtime_error("<?php\nconnection_status(1);\n");
    assert_eq!(connection_error.line, 2);
    assert_eq!(
        connection_error.message,
        "arity mismatch for connection_status(): expected 0 argument(s), got 1"
    );
}
