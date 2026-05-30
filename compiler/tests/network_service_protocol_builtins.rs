use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

#[test]
fn bounded_protocol_and_service_lookup_tables_cover_common_cli_entries() {
    let execution = run_source(
        r#"<?php
echo getprotobyname("tcp"), "|";
echo getprotobynumber(6), "|";
var_dump(getprotobyname("abc"));
var_dump(getprotobynumber(999));
foreach (["http", "ftp", "ssh", "telnet", "imap", "smtp", "nicname", "gopher", "finger", "pop3", "www"] as $service) {
    echo getservbyname($service, "tcp"), ",";
}
echo "|";
echo getservbyport(80, "tcp"), "|";
var_dump(getservbyport(-1, "tcp"));
var_dump(getservbyport(80, "ppp"));
var_dump(getservbyport(2, 2));
echo getservbyport("80", "tcp");
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "6|tcp|bool(false)\n\
bool(false)\n\
80,21,22,23,143,25,43,70,79,110,80,|http|bool(false)\n\
bool(false)\n\
bool(false)\n\
http"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn service_database_keeps_service_name_and_protocol_case_sensitive() {
    let execution = run_source(
        r#"<?php
var_dump(getservbyname("HTTP", "tcp"));
var_dump(getservbyname("http", "TCP"));
var_dump(getservbyport(80, "TCP"));
var_dump(getprotobyname("TCP"));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "bool(false)\n\
bool(false)\n\
bool(false)\n\
int(6)\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn service_protocol_functions_are_callable_and_have_reflection_metadata() {
    let execution = run_source(
        r#"<?php
foreach (["getprotobyname", "getprotobynumber", "getservbyname", "getservbyport"] as $call) {
    echo function_exists($call) ? "1" : "0";
    echo is_callable($call) ? "1" : "0";
    $ref = new ReflectionFunction($call);
    echo $ref->isInternal() ? "1" : "0";
}
echo "|";
$call = "getservbyname";
echo $call("http", "tcp");
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "111111111111|80");
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn service_protocol_rows_can_validate_services_with_preg_match_whitespace_run() {
    let execution = run_source(
        r#"<?php
$service = getservbyport(80, "tcp");
$services = "ssh 22/tcp\nhttp\t80/tcp\n";
echo preg_match("/$service\s+80\/tcp/", $services, $matches);
echo "|";
echo $matches[0];
echo "|";
echo preg_match("/$service\s+443\/tcp/", $services);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "1|http\t80/tcp|0");
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn network_database_null_bytes_are_catchable_value_errors() {
    let execution = run_source(
        "<?php\n\
foreach ([\n\
    fn() => getprotobyname(\"\\0\"),\n\
    fn() => getservbyname(\"\\0\", \"tcp\"),\n\
    fn() => getservbyname(\"x\", \"tcp\\0\"),\n\
    fn() => getservbyport(0, \"tcp\\0\"),\n\
] as $call) {\n\
    try {\n\
        $call();\n\
    } catch (Throwable $e) {\n\
        echo $e::class, \": \", $e->getMessage(), \"\\n\";\n\
    }\n\
}\n",
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "ValueError: getprotobyname(): Argument #1 ($protocol) must not contain any null bytes\n\
ValueError: getservbyname(): Argument #1 ($service) must not contain any null bytes\n\
ValueError: getservbyname(): Argument #2 ($protocol) must not contain any null bytes\n\
ValueError: getservbyport(): Argument #2 ($protocol) must not contain any null bytes\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_folds_service_protocol_metadata_but_rejects_direct_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("getprotobyname") ? "1" : "0";
echo is_callable("getprotobynumber") ? "1" : "0";
echo function_exists("getservbyname") ? "1" : "0";
echo is_callable("getservbyport") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 4, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source("<?php\necho getprotobyname('tcp');\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source("<?php\necho getservbyport(80, 'tcp');\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}
