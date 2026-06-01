use php_compiler::run_source;
use std::fs;
use std::path::Path;
use std::process::{Command, Output};

#[test]
fn iconv_scalar_builtins_count_search_and_slice_bounded_encodings() {
    let execution = run_source(
        r#"<?php
iconv_set_encoding("internal_encoding", "UTF-8");
$japanese = base64_decode("5pel5pys6Kqe44OG44Kt44K544OI44Gn44GZ44CCMDEyMzTvvJXvvJbvvJfvvJjvvJnjgII=");
$period = base64_decode("44CC");
var_dump(iconv_strlen("abc def"));
var_dump(iconv_strlen($japanese, "UTF-8"));
var_dump(iconv_strpos($japanese, $period));
var_dump(iconv_strrpos($japanese, $period));
iconv_set_encoding("internal_encoding", "ISO-8859-1");
var_dump(bin2hex(iconv_substr($japanese, 2, 7)));
var_dump(bin2hex(iconv_substr($japanese, 2, 7, "utf-8")));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "int(7)\n",
            "int(21)\n",
            "int(9)\n",
            "int(20)\n",
            "string(14) \"a5e69cace8aa9e\"\n",
            "string(42) \"e8aa9ee38386e382ade382b9e38388e381a7e38199\"\n",
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn iconv_encoding_accessors_and_metadata_are_bounded() {
    let execution = run_source(
        r#"<?php
var_dump(extension_loaded("iconv"));
var_dump(iconv_get_encoding("internal_encoding"));
var_dump(iconv_set_encoding("internal_encoding", "ISO-8859-1"));
var_dump(iconv_get_encoding("internal_encoding"));
var_dump(iconv_get_encoding("missing"));
foreach (["iconv_strlen", "iconv_substr"] as $name) {
    echo function_exists($name) ? "1" : "0";
    $fn = new ReflectionFunction($name);
    echo ":", $fn->getExtensionName(), ":", $fn->getNumberOfRequiredParameters(), "/", $fn->getNumberOfParameters(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "bool(true)\n",
            "string(5) \"UTF-8\"\n",
            "bool(true)\n",
            "string(10) \"ISO-8859-1\"\n",
            "bool(false)\n",
            "1:iconv:1/2\n",
            "1:iconv:2/4\n",
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn iconv_scalar_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join("tests/fixtures/milestone2305/iconv_scalar.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8");

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args(["run", relative_fixture])
        .output()
        .unwrap_or_else(|error| panic!("failed to run phpc for {relative_fixture}: {error}"));

    let expected = fs::read_to_string(fixture.with_extension("cli"))
        .unwrap_or_else(|error| panic!("failed to read iconv CLI snapshot: {error}"));
    assert_eq!(render_cli_snapshot(&output), expected);
}

fn render_cli_snapshot(output: &Output) -> String {
    let exit_code = output.status.code().unwrap_or(1);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    format!(
        "exit: {exit_code}\nstdout:\n{stdout}--- stdout end ---\nstderr:\n{stderr}--- stderr end ---\n"
    )
}
