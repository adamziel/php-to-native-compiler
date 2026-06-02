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
fn hash_md5_and_small_pure_digest_algorithms_are_available() {
    let execution = run_source(
        r#"<?php
echo hash("md5", ""), "\n";
echo hash("md5", "a"), "\n";
echo hash("md5", "012345678901234567890123456789012345678901234567890123456789"), "\n";
var_dump(hash("md5", "string") === md5("string"));
echo bin2hex(hash("md5", "string", true)), "\n";
echo hash("adler32", ""), "\n";
echo hash("adler32", "abc"), "\n";
echo hash("fnv132", ""), "\n";
echo hash("fnv132", "foobar"), "\n";
echo bin2hex(hash("fnv132", "", true)), "\n";
echo hash("fnv1a32", "a"), "\n";
echo hash("fnv164", ""), "\n";
echo hash("fnv164", "foobar"), "\n";
echo hash("fnv1a64", "9"), "\n";
echo hash("joaat", "hello world"), "\n";
echo bin2hex(hash("joaat", "", true)), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "d41d8cd98f00b204e9800998ecf8427e\n\
0cc175b9c0f1b6a831c399e269772661\n\
1ced811af47ead374872fcca9d73dd71\n\
bool(true)\n\
b45cffe084dd3d20d928bee85e7b0f21\n\
00000001\n\
024d0127\n\
811c9dc5\n\
31f0b262\n\
811c9dc5\n\
e40c292c\n\
cbf29ce484222325\n\
340d8765a4dda9c2\n\
af63b44c8601a894\n\
3e4a5a57\n\
00000000\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn hash_crc_algorithms_and_php82_algorithm_listing_are_available() {
    let execution = run_source(
        r#"<?php
echo hash("crc32", "123456789"), "\n";
echo hash("crc32b", "123456789"), "\n";
echo hash("crc32c", "123456789"), "\n";
echo bin2hex(hash("crc32", "123456789", true)), "\n";
$algos = hash_algos();
echo count($algos), "|", $algos[30], "|", $algos[31], "|", $algos[32];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "181989fc\ncbf43926\ne3069283\n181989fc\n60|crc32|crc32b|crc32c"
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
fn hash_hmac_invalid_or_non_crypto_algorithms_are_catchable_value_errors() {
    let execution = run_source(
        r#"<?php
foreach (["foo", "crc32"] as $algorithm) {
    try {
        hash_hmac($algorithm, "data", "key");
    } catch (ValueError $e) {
        echo $e->getMessage(), "\n";
    }
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "hash_hmac(): Argument #1 ($algo) must be a valid cryptographic hashing algorithm\n\
hash_hmac(): Argument #1 ($algo) must be a valid cryptographic hashing algorithm\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn hash_init_validates_algorithm_hmac_mode_and_key_before_streaming_boundary() {
    let execution = run_source(
        r#"<?php
echo defined("HASH_HMAC") ? HASH_HMAC : "missing";
echo "|", function_exists("hash_init") ? "fn" : "missing";
echo "|", is_callable("hash_init") ? "callable" : "missing", "\n";

foreach ([
    fn() => hash_init("dummy"),
    fn() => hash_init("crc32", HASH_HMAC),
    fn() => hash_init("md5", HASH_HMAC),
    fn() => hash_init("md5", HASH_HMAC, null),
] as $test) {
    try {
        var_dump($test());
    } catch (\Error $e) {
        echo get_class($e), ":", $e->getMessage(), "\n";
    }
}
"#,
    )
    .unwrap();

    assert!(execution.stdout.starts_with(
        "1|fn|callable\n\
ValueError:hash_init(): Argument #1 ($algo) must be a valid hashing algorithm\n\
ValueError:hash_init(): Argument #1 ($algo) must be a cryptographic hashing algorithm if HMAC is requested\n\
ValueError:hash_init(): Argument #3 ($key) must not be empty when HMAC is requested\n"
    ));
    assert!(
        execution.stdout.contains(
            "Deprecated: hash_init(): Passing null to parameter #3 ($key) of type string is deprecated"
        ),
        "{}",
        execution.stdout
    );
    assert!(
        execution.stdout.ends_with(
            "ValueError:hash_init(): Argument #3 ($key) must not be empty when HMAC is requested\n"
        ),
        "{}",
        execution.stdout
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);

    let boundary = run_source("<?php\nhash_init('md5');\n").unwrap_err();
    assert_eq!(boundary.phase, Phase::Runtime);
    assert_eq!(boundary.line, 2);
    assert_eq!(boundary.column, 1);
    assert_eq!(
        boundary.message,
        "unsupported call hash_init(): HashContext allocation and streaming updates are not implemented in the current subset"
    );
}

#[test]
fn hash_pbkdf2_validates_algorithm_iterations_and_length_before_derivation_boundary() {
    let execution = run_source(
        r#"<?php
echo function_exists("hash_pbkdf2") ? "fn" : "missing";
echo "|", is_callable("hash_pbkdf2") ? "callable" : "missing";
$fn = new ReflectionFunction("hash_pbkdf2");
echo "|", $fn->getNumberOfRequiredParameters(), "/", $fn->getNumberOfParameters(), "\n";

foreach ([
    fn() => hash_pbkdf2("foo", "password", "salt", 1),
    fn() => hash_pbkdf2("crc32", "password", "salt", 1),
    fn() => hash_pbkdf2("md5", "password", "salt", 0),
    fn() => hash_pbkdf2("md5", "password", "salt", -1),
    fn() => hash_pbkdf2("md5", "password", "salt", 1, -1),
] as $test) {
    try {
        var_dump($test());
    } catch (\Error $e) {
        echo get_class($e), ":", $e->getMessage(), "\n";
    }
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "fn|callable|4/6\n\
ValueError:hash_pbkdf2(): Argument #1 ($algo) must be a valid cryptographic hashing algorithm\n\
ValueError:hash_pbkdf2(): Argument #1 ($algo) must be a valid cryptographic hashing algorithm\n\
ValueError:hash_pbkdf2(): Argument #4 ($iterations) must be greater than 0\n\
ValueError:hash_pbkdf2(): Argument #4 ($iterations) must be greater than 0\n\
ValueError:hash_pbkdf2(): Argument #5 ($length) must be greater than or equal to 0\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);

    let boundary = run_source("<?php\nhash_pbkdf2('md5', 'password', 'salt', 1);\n").unwrap_err();
    assert_eq!(boundary.phase, Phase::Runtime);
    assert_eq!(boundary.line, 2);
    assert_eq!(boundary.column, 1);
    assert_eq!(
        boundary.message,
        "unsupported call hash_pbkdf2(): PBKDF2 derivation is not implemented in the current subset"
    );
}

#[test]
fn hash_equals_and_hmac_algorithm_metadata_cover_phpt_rows() {
    let execution = run_source(
        r#"<?php
echo hash_equals("same", "same") ? "same" : "bad";
echo "|", hash_equals("not1same", "not2same") ? "bad" : "diff";
echo "|", hash_equals("short", "longer") ? "bad" : "length";
echo "|", hash_equals("", "") ? "empty" : "bad", "\n";

$algos = hash_hmac_algos();
echo count($algos), "|", $algos[0], "|", $algos[5], "|", $algos[43], "|";
echo in_array("crc32", $algos, true) ? "bad" : "crypto-only";
echo "\n";

foreach ([
    fn() => hash_equals(123, "NaN"),
    fn() => hash_equals("NaN", 123),
    fn() => hash_equals(null, null),
] as $test) {
    try {
        var_dump($test());
    } catch (\Error $e) {
        echo get_class($e), ":", $e->getMessage(), "\n";
    }
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "same|diff|length|empty\n\
44|md2|sha256|haval256,5|crypto-only\n\
TypeError:hash_equals(): Argument #1 ($known_string) must be of type string, int given\n\
TypeError:hash_equals(): Argument #2 ($user_string) must be of type string, int given\n\
TypeError:hash_equals(): Argument #1 ($known_string) must be of type string, null given\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_folds_hash_metadata_but_rejects_direct_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("hash") ? "1" : "0";
echo is_callable("hash_algos") ? "1" : "0";
echo function_exists("hash_init") ? "1" : "0";
echo is_callable("hash_init") ? "1" : "0";
echo function_exists("hash_hmac") ? "1" : "0";
echo is_callable("hash_hmac") ? "1" : "0";
echo function_exists("hash_pbkdf2") ? "1" : "0";
echo is_callable("hash_pbkdf2") ? "1" : "0";
echo function_exists("hash_hmac_algos") ? "1" : "0";
echo is_callable("hash_hmac_algos") ? "1" : "0";
echo function_exists("hash_equals") ? "1" : "0";
echo is_callable("hash_equals") ? "1" : "0";
echo function_exists("uniqid") ? "1" : "0";
echo is_callable("uniqid") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 14, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source("<?php\nhash('sha256', 'data');\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source("<?php\nhash_init('sha256');\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source("<?php\nhash_hmac('sha256', 'data', 'key');\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source("<?php\nhash_pbkdf2('sha1', 'password', 'salt', 1);\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source("<?php\nhash_equals('same', 'same');\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}
