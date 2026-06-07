use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

#[test]
fn url_encoding_builtins_cover_form_and_rfc3986_variants() {
    let execution = run_source(
        r#"<?php
echo urlencode("a b~+"), "\n";
echo rawurlencode("a b~+"), "\n";
echo urldecode("a+b%2B%7E%zz%"), "\n";
echo rawurldecode("a+b%2B%7E%zz%"), "\n";
echo bin2hex(rawurldecode("%FF%20")), "\n";
echo bin2hex(urldecode("%00+%2B"));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "a+b%7E%2B\na%20b~%2B\na b+~%zz%\na+b+~%zz%\nff20\n00202b"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn url_encoding_metadata_and_dynamic_calls_are_available() {
    let execution = run_source(
        r#"<?php
foreach (["urlencode", "rawurlencode", "urldecode", "rawurldecode"] as $name) {
    echo function_exists($name) ? "1" : "0";
    echo is_callable($name) ? "1" : "0";
    $function = new ReflectionFunction($name);
    echo ":", $function->getName(), "/", $function->getNumberOfRequiredParameters(), "/", $function->getNumberOfParameters(), ";";
}
echo "|";
$call = "rawurlencode";
echo $call("wp admin");
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "11:urlencode/1/1;11:rawurlencode/1/1;11:urldecode/1/1;11:rawurldecode/1/1;|wp%20admin"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn url_encoding_rejects_arrays_in_current_subset() {
    let execution = run_source("<?php\nrawurlencode(['admin']);\n").unwrap();

    assert_eq!(execution.exit_code, 255);
    assert!(
        execution.stdout.contains(
            "TypeError: rawurlencode(): Argument #1 ($string) must be of type string, array given"
        ),
        "{}",
        execution.stdout
    );
}

#[test]
fn emit_ir_folds_url_encoding_metadata_but_rejects_direct_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("urlencode") ? "1" : "0";
echo is_callable("urlencode") ? "1" : "0";
echo function_exists("rawurlencode") ? "1" : "0";
echo is_callable("rawurlencode") ? "1" : "0";
echo function_exists("urldecode") ? "1" : "0";
echo is_callable("urldecode") ? "1" : "0";
echo function_exists("rawurldecode") ? "1" : "0";
echo is_callable("rawurldecode") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 8, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source("<?php\nrawurlencode('wp admin');\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}

#[test]
fn url_encoding_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture_dir = workspace_root.join("tests/fixtures/milestone2305");
    let mut fixtures = cli_snapshot_fixtures(&fixture_dir);

    fixtures.sort();
    assert!(
        !fixtures.is_empty(),
        "expected URL encoding CLI snapshot fixtures"
    );

    for fixture in fixtures {
        let file_name = fixture
            .file_name()
            .and_then(|value| value.to_str())
            .expect("URL encoding fixture file name is valid UTF-8");
        let fixture_arg = format!("tests/fixtures/milestone2305/{file_name}");
        let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
            .current_dir(workspace_root)
            .args(["run", &fixture_arg])
            .output()
            .unwrap_or_else(|error| panic!("failed to run phpc for {fixture_arg}: {error}"));

        let expected = fs::read_to_string(fixture.with_extension("cli")).unwrap_or_else(|error| {
            panic!("failed to read CLI snapshot for {fixture_arg}: {error}")
        });
        let actual = render_cli_snapshot(&output);

        assert_eq!(actual, expected, "CLI snapshot mismatch for {fixture_arg}");
    }
}

fn cli_snapshot_fixtures(fixture_dir: &Path) -> Vec<PathBuf> {
    fs::read_dir(fixture_dir)
        .expect("URL encoding fixture directory is readable")
        .map(|entry| {
            entry
                .expect("URL encoding fixture entry is readable")
                .path()
        })
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("php"))
        .filter(|path| path.file_name().and_then(|name| name.to_str()) == Some("url_encoding.php"))
        .filter(|path| path.with_extension("cli").exists())
        .collect()
}

fn render_cli_snapshot(output: &Output) -> String {
    let exit_code = output.status.code().unwrap_or(1);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    format!(
        "exit: {exit_code}\nstdout:\n{stdout}--- stdout end ---\nstderr:\n{stderr}--- stderr end ---\n"
    )
}
