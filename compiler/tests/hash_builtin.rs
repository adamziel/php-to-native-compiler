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
fn hash_sha_family_and_algorithm_listing_are_available() {
    let execution = run_source(
        r#"<?php
echo hash("sha1", "abc"), "\n";
echo hash("sha224", "abc"), "\n";
echo hash("sha256", "abc"), "\n";
echo hash("sha384", "abc"), "\n";
echo hash("sha512/224", "abc"), "\n";
echo hash("sha512/256", "abc"), "\n";
echo substr(hash("sha512", "abc"), 0, 16), "\n";
echo bin2hex(hash("sha256", "abc", true)), "\n";
echo in_array("sha512/256", hash_algos(), true) ? "listed" : "missing";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "a9993e364706816aba3e25717850c26c9cd0d89d\n\
23097d223405d8228642a477bda255b32aadbce4bda0b3f7e36c9da7\n\
ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad\n\
cb00753f45a35e8bb5a03d699ac65007272c32ab0eded1631a8b605a43ff5bed8086072ba1e7cc2358baeca134c825a7\n\
4634270f707b6a54daae7530460842e20e37ed265ceee9a43e8924aa\n\
53048e2681941ef99b2e29b76b4c7dabe4c2d0c634fc6d46e0e2f13107e7af23\n\
ddaf35a193617aba\n\
ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad\n\
listed"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn hash_rejects_unknown_algorithms() {
    let execution = run_source("<?php\nhash('foo', '');\n").unwrap();
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 255);
    assert!(
        execution.stdout.contains(
            "Fatal error: Uncaught ValueError: hash(): Argument #1 ($algo) must be a valid hashing algorithm"
        ),
        "{}",
        execution.stdout
    );
}

#[test]
fn hash_unknown_algorithm_is_catchable_value_error() {
    let execution = run_source(
        r#"<?php
try {
    hash("foo", "");
} catch (ValueError $e) {
    echo get_class($e), ":", $e->getMessage();
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "ValueError:hash(): Argument #1 ($algo) must be a valid hashing algorithm"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_folds_hash_metadata_but_rejects_direct_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("hash") ? "1" : "0";
echo is_callable("hash_algos") ? "1" : "0";
echo function_exists("hash_hmac") ? "1" : "0";
echo is_callable("hash_hmac") ? "1" : "0";
echo function_exists("uniqid") ? "1" : "0";
echo is_callable("uniqid") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 6, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source("<?php\nhash('sha256', 'data');\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source("<?php\nhash_hmac('sha256', 'data', 'key');\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}
