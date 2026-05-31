use std::env;
use std::sync::{Mutex, MutexGuard, OnceLock};

use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

fn env_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .expect("environment lock is not poisoned")
}

fn set_env_var(name: &str, value: &str) {
    env::set_var(name, value);
}

fn restore_env_var(name: &str, previous: Option<std::ffi::OsString>) {
    if let Some(value) = previous {
        env::set_var(name, value);
    } else {
        env::remove_var(name);
    }
}

#[test]
fn ini_get_reads_current_deterministic_registry() {
    let execution = run_source(
        r#"<?php
echo ini_get("memory_limit"), "|";
echo ini_get("MEMORY_LIMIT"), "|";
echo ini_get("mbstring.func_overload"), "|";
echo ini_get("missing.option") === false ? "false" : "not-false";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "128M|128M|0|false");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn ini_set_updates_current_deterministic_registry() {
    let execution = run_source(
        r#"<?php
echo ini_set("display_errors", 0), "|";
echo ini_get("display_errors"), "|";
echo ini_set("DISPLAY_ERRORS", true), "|";
echo ini_get("display_errors"), "|";
echo ini_set("display_errors", null), "|";
echo ini_get("display_errors"), "|";
echo ini_set("missing.option", "x") === false ? "false" : "not-false";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "|0|0|1|1||false");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn get_cfg_var_reads_startup_configuration_not_runtime_mutation() {
    let execution = run_source(
        r#"<?php
echo get_cfg_var("memory_limit"), "|";
ini_set("memory_limit", "256M");
echo ini_get("memory_limit"), "|";
echo get_cfg_var("memory_limit"), "|";
echo get_cfg_var("missing.option") === false ? "false" : "not-false";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "128M|256M|128M|false");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn get_cfg_var_reads_phpt_startup_override_not_runtime_mutation() {
    let _guard = env_lock();
    let previous = env::var_os("PHPC_PHPT_INI_FLAGS");

    set_env_var("PHPC_PHPT_INI_FLAGS", "-d memory_limit=64M");
    let execution = run_source(
        r#"<?php
echo get_cfg_var("memory_limit"), "|";
echo ini_get("memory_limit"), "|";
ini_set("memory_limit", "256M");
echo get_cfg_var("memory_limit"), "|";
echo ini_get("memory_limit");
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "64M|64M|64M|256M");
    assert_eq!(execution.exit_code, 0);

    restore_env_var("PHPC_PHPT_INI_FLAGS", previous);
}

#[test]
fn ini_set_rejects_startup_only_max_memory_limit() {
    let execution = run_source(
        r#"<?php
var_dump(ini_set("max_memory_limit", "128M"));
var_dump(ini_set("MAX_MEMORY_LIMIT", "256M"));
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "bool(false)\nbool(false)\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn ini_set_rejects_empty_arg_separators_without_mutating() {
    let execution = run_source(
        r#"<?php
var_dump(ini_set("arg_separator.output", ""));
var_dump(ini_get("arg_separator.output"));
var_dump(ini_set("arg_separator.input", ""));
var_dump(ini_get("arg_separator.input"));
var_dump(ini_set("arg_separator.output", "|"));
var_dump(ini_get("arg_separator.output"));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "bool(false)\nstring(1) \"&\"\nbool(false)\nstring(1) \"&\"\nstring(1) \"&\"\nstring(1) \"|\"\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn ini_parse_quantity_handles_decimal_hex_and_compatibility_suffixes() {
    let _guard = env_lock();
    let previous = env::var_os("PHPC_PHPT_INI_FLAGS");

    env::remove_var("PHPC_PHPT_INI_FLAGS");
    let execution = run_source(
        r#"<?php
error_reporting(E_ALL ^ E_WARNING);
foreach (array('-1', '-0x412', '0', '1', '1b', '1k', '1m', '1g', '1gb', '14.2mb', '14.2bm', 'boat') as $input) {
    echo ini_parse_quantity($input), "\n";
}
"#,
    )
    .unwrap();
    let stdout = execution.stdout.clone();
    let exit_code = execution.exit_code;

    restore_env_var("PHPC_PHPT_INI_FLAGS", previous);

    assert_eq!(
        stdout,
        "-1\n-1042\n0\n1\n1\n1024\n1048576\n1073741824\n1\n14\n14680064\n0\n"
    );
    assert_eq!(exit_code, 0);
}

#[test]
fn ini_parse_quantity_emits_php_compatibility_warnings() {
    let _guard = env_lock();
    let previous = env::var_os("PHPC_PHPT_INI_FLAGS");

    env::remove_var("PHPC_PHPT_INI_FLAGS");
    let execution = run_source(
        r#"<?php
ini_parse_quantity('1mb');
ini_parse_quantity('14.2bm');
ini_parse_quantity('1.5');
ini_parse_quantity('0x');
ini_parse_quantity('256 then skip a few then g');
"#,
    )
    .unwrap();
    let stdout = execution.stdout.clone();
    let exit_code = execution.exit_code;

    restore_env_var("PHPC_PHPT_INI_FLAGS", previous);

    assert!(stdout.contains(
        "Warning: Invalid quantity \"1mb\": unknown multiplier \"b\", interpreting as \"1\" for backwards compatibility"
    ));
    assert!(stdout.contains(
        "Warning: Invalid quantity \"14.2bm\", interpreting as \"14m\" for backwards compatibility"
    ));
    assert!(stdout.contains(
        "Warning: Invalid quantity \"1.5\": unknown multiplier \"5\", interpreting as \"1\" for backwards compatibility"
    ));
    assert!(stdout.contains(
        "Warning: Invalid quantity \"0x\": no digits after base prefix, interpreting as \"0\" for backwards compatibility"
    ));
    assert!(stdout.contains(
        "Warning: Invalid quantity \"256 then skip a few then g\", interpreting as \"256 g\" for backwards compatibility"
    ));
    assert_eq!(exit_code, 0);
}

#[test]
fn ini_parse_quantity_uses_php_overflow_results() {
    let _guard = env_lock();
    let previous = env::var_os("PHPC_PHPT_INI_FLAGS");

    env::remove_var("PHPC_PHPT_INI_FLAGS");
    let execution = run_source(
        r#"<?php
$cases = array(
    '0x8000000000000000',
    '-0x8000000000000000',
    '9223372036854775808',
    '-9223372036854775808',
    '9223372036854775807K',
    '-9223372036854775808K',
);
foreach ($cases as $case) {
    echo "--", $case, "--\n";
    var_dump(ini_parse_quantity($case));
}
"#,
    )
    .unwrap();
    let stdout = execution.stdout.clone();
    let exit_code = execution.exit_code;

    restore_env_var("PHPC_PHPT_INI_FLAGS", previous);

    assert!(stdout.contains(
        "Warning: Invalid quantity \"0x8000000000000000\": value is out of range, using overflow result for backwards compatibility"
    ));
    assert!(stdout.contains(
        "Warning: Invalid quantity \"9223372036854775808\": value is out of range, using overflow result for backwards compatibility"
    ));
    assert!(stdout.contains(
        "Warning: Invalid quantity \"9223372036854775807K\": value is out of range, using overflow result for backwards compatibility"
    ));
    assert!(stdout.contains(
        "Warning: Invalid quantity \"-9223372036854775808K\": value is out of range, using overflow result for backwards compatibility"
    ));
    assert!(stdout.contains("--0x8000000000000000--\n\nWarning:"));
    assert!(stdout
        .contains("int(-9223372036854775808)\n---0x8000000000000000--\nint(-9223372036854775808)"));
    assert!(stdout.contains("--9223372036854775808--\n\nWarning:"));
    assert!(stdout.contains(
        "int(-9223372036854775808)\n---9223372036854775808--\nint(-9223372036854775808)"
    ));
    assert!(stdout.contains("int(-1024)"));
    assert!(stdout.contains("int(0)"));
    assert_eq!(exit_code, 0);
}

#[test]
fn ini_parse_quantity_honors_phpt_error_reporting_masks() {
    let _guard = env_lock();
    let previous = env::var_os("PHPC_PHPT_INI_FLAGS");

    set_env_var(
        "PHPC_PHPT_INI_FLAGS",
        "-d error_reporting=E_ALL ^ E_WARNING",
    );
    let xor_execution = run_source("<?php\necho ini_parse_quantity('1mb'), \"\\n\";\n").unwrap();
    assert_eq!(xor_execution.stdout, "1\n");
    assert_eq!(xor_execution.exit_code, 0);

    set_env_var(
        "PHPC_PHPT_INI_FLAGS",
        "-d error_reporting=~E_WARNING & E_ALL",
    );
    let leading_not_execution = run_source("<?php\necho error_reporting(), \"\\n\";\n").unwrap();
    let leading_not_stdout = leading_not_execution.stdout.clone();
    let leading_not_exit_code = leading_not_execution.exit_code;

    restore_env_var("PHPC_PHPT_INI_FLAGS", previous);

    assert_eq!(leading_not_stdout, "0\n");
    assert_eq!(leading_not_exit_code, 0);
}

#[test]
fn ini_builtins_are_available_through_string_valued_calls() {
    let execution = run_source(
        r#"<?php
$get = "ini_get";
$set = "ini_set";
echo function_exists($get) ? "yes" : "no";
echo "|";
echo is_callable($set) ? "callable" : "missing";
echo "|";
echo $set("memory_limit", "256M"), "|";
echo $get("memory_limit");
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|128M|256M");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn ini_builtins_reject_forms_outside_current_subset() {
    let non_string = run_source("<?php\nini_get(42);\n").unwrap_err();
    assert_eq!(non_string.phase, Phase::Runtime);
    assert_eq!(non_string.line, 2);
    assert_eq!(non_string.column, 1);
    assert_eq!(
        non_string.message,
        "unsupported call ini_get(): option argument must be string in the current subset, got int"
    );

    let non_string_set = run_source("<?php\nini_set(42, 'x');\n").unwrap_err();
    assert_eq!(non_string_set.phase, Phase::Runtime);
    assert_eq!(non_string_set.line, 2);
    assert_eq!(non_string_set.column, 1);
    assert_eq!(
        non_string_set.message,
        "unsupported call ini_set(): option argument must be string in the current subset, got int"
    );

    let array_value = run_source("<?php\nini_set('display_errors', []);\n").unwrap_err();
    assert_eq!(array_value.phase, Phase::Runtime);
    assert_eq!(array_value.line, 2);
    assert_eq!(array_value.column, 1);
    assert_eq!(
        array_value.message,
        "unsupported call ini_set(): value argument must be null or scalar in the current subset, got array"
    );

    let too_many = run_source("<?php\nini_get('memory_limit', true);\n").unwrap_err();
    assert_eq!(too_many.phase, Phase::Runtime);
    assert_eq!(too_many.line, 2);
    assert_eq!(too_many.column, 1);
    assert_eq!(
        too_many.message,
        "arity mismatch for ini_get(): expected 1 argument(s), got 2"
    );

    let too_few = run_source("<?php\nini_set('memory_limit');\n").unwrap();
    assert_eq!(too_few.exit_code, 255);
    assert!(
        too_few.stdout.contains(
            "Fatal error: Uncaught TypeError: Too few arguments to function ini_set(), 1 passed"
        ),
        "{}",
        too_few.stdout
    );
}

#[test]
fn emit_ir_folds_ini_get_metadata_but_rejects_direct_ini_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("ini_get") ? "1" : "0";
echo is_callable("ini_set") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source("<?php\nini_get('memory_limit');\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let set_error = emit_ir_source("<?php\nini_set('memory_limit', '256M');\n").unwrap_err();
    assert_eq!(set_error.phase, Phase::Codegen);
    assert_eq!(set_error.line, 2);
    assert_eq!(set_error.column, 1);
    assert_eq!(set_error.message, LLVM_FUNCTION_CALL_REJECTION);
}

#[test]
fn parse_ini_string_typed_scanner_converts_supported_scalars() {
    let execution = run_source(
        r#"<?php
$values = parse_ini_string(
    "a=true\nb=false\nc=null\nd=123\ne=3.5\nf=\"123\"\ng=on\nh=off\ni=yes\nj=no\nk=none\nl=\"true\"",
    false,
    INI_SCANNER_TYPED
);
var_dump($values);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "array(12) {\n  [\"a\"]=>\n  bool(true)\n  [\"b\"]=>\n  bool(false)\n  [\"c\"]=>\n  NULL\n  [\"d\"]=>\n  int(123)\n  [\"e\"]=>\n  float(3.5)\n  [\"f\"]=>\n  string(3) \"123\"\n  [\"g\"]=>\n  bool(true)\n  [\"h\"]=>\n  bool(false)\n  [\"i\"]=>\n  bool(true)\n  [\"j\"]=>\n  bool(false)\n  [\"k\"]=>\n  bool(false)\n  [\"l\"]=>\n  string(4) \"true\"\n}\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn parse_ini_string_raw_scanner_strips_unquoted_inline_comments() {
    let execution = run_source(
        r#"<?php
$ini = <<<END
1="foo"
2="bar" ; comment
3= baz
4= "foo;bar"
5= "foo" ; bar ; baz
6= "foo;bar" ; baz
7= foo"bar ; "ok
END;
var_dump(parse_ini_string($ini, false, INI_SCANNER_RAW));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "array(7) {\n  [1]=>\n  string(3) \"foo\"\n  [2]=>\n  string(3) \"bar\"\n  [3]=>\n  string(3) \"baz\"\n  [4]=>\n  string(7) \"foo;bar\"\n  [5]=>\n  string(3) \"foo\"\n  [6]=>\n  string(7) \"foo;bar\"\n  [7]=>\n  string(7) \"foo\"bar\"\n}\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn parse_ini_string_reports_unclosed_interpolation_in_section_header() {
    let execution = run_source("<?php\nvar_dump(parse_ini_string('[${ \t'));\n").unwrap();

    assert!(
        execution.stdout.contains(
            "Warning: syntax error, unexpected end of file, expecting TC_FALLBACK or '}' in Unknown on line 1"
        ),
        "{}",
        execution.stdout
    );
    assert!(
        execution.stdout.ends_with("bool(false)\n"),
        "{}",
        execution.stdout
    );
    assert_eq!(execution.exit_code, 0);
}
