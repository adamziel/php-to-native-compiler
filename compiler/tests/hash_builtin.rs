use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

#[test]
fn hash_hmac_sha256_and_uniqid_cover_current_placeholder_slice() {
    let execution = run_source(
        r#"<?php
echo function_exists("hash_hmac") ? "hash" : "missing";
echo "|";
echo is_callable("uniqid") ? "uniqid" : "missing";
echo "|";
echo uniqid("salt", true);
echo "|";
echo hash_hmac("sha256", "data", "key");
echo "|";
$call = "hash_hmac";
echo $call("sha256", uniqid("salt", true), "salt", false);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "hash|uniqid|salt0000000000000.000000000|5031fe3d989c6d1537a013fa6e739da23463fdaec3b70137d828e36ace221bd0|a7afc0875fafe2923927b020bdb1a243deb47ffd9d2429731c87e88dc3cf2d4f"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn uniqid_more_entropy_matches_php_shaped_lengths() {
    let execution = run_source(
        r#"<?php
echo strlen(uniqid()), ":", uniqid(), "\n";
echo strlen(uniqid('', true)), ":", uniqid('', true), "\n";
echo strlen(uniqid(99999, true)), ":", uniqid(99999, true), "\n";
echo strlen(uniqid(10.5e2, true)), ":", uniqid(10.5e2, true), "\n";
echo strlen(uniqid(true, true)), ":", uniqid(true, true), "\n";
echo strlen(uniqid(false, true)), ":", uniqid(false, true), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "13:0000000000000\n23:0000000000000.000000000\n28:999990000000000000.000000000\n27:10500000000000000.000000000\n24:10000000000000.000000000\n23:0000000000000.000000000\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn hash_hmac_and_uniqid_reject_forms_outside_current_boundary() {
    let algorithm = run_source("<?php\nhash_hmac('md5', 'data', 'key');\n").unwrap_err();
    assert_eq!(algorithm.phase, Phase::Runtime);
    assert_eq!(algorithm.line, 2);
    assert_eq!(algorithm.column, 1);
    assert_eq!(
        algorithm.message,
        "unsupported call hash_hmac(): only sha256 is implemented in the current subset"
    );

    let raw = run_source("<?php\nhash_hmac('sha256', 'data', 'key', true);\n").unwrap_err();
    assert_eq!(raw.phase, Phase::Runtime);
    assert_eq!(raw.line, 2);
    assert_eq!(raw.column, 1);
    assert_eq!(
        raw.message,
        "unsupported call hash_hmac(): raw binary output is not implemented; omit raw_output or pass false in the current subset"
    );

    let entropy = run_source("<?php\nuniqid('salt', 1);\n").unwrap_err();
    assert_eq!(entropy.phase, Phase::Runtime);
    assert_eq!(entropy.line, 2);
    assert_eq!(entropy.column, 1);
    assert_eq!(
        entropy.message,
        "unsupported call uniqid(): more_entropy argument must be bool in the current subset, got int"
    );
}

#[test]
fn emit_ir_folds_hash_metadata_but_rejects_direct_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("hash_hmac") ? "1" : "0";
echo is_callable("hash_hmac") ? "1" : "0";
echo function_exists("uniqid") ? "1" : "0";
echo is_callable("uniqid") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 4, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source("<?php\nhash_hmac('sha256', 'data', 'key');\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}
