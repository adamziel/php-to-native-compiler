use php_compiler::run_source;

#[test]
fn sha1_matches_public_phpt_vectors_and_raw_output() {
    let execution = run_source(
        r#"<?php
foreach ([
    "",
    "a",
    "abc",
    "message digest",
    "abcdefghijklmnopqrstuvwxyz",
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789",
    "12345678901234567890123456789012345678901234567890123456789012345678901234567890",
] as $value) {
    echo sha1($value), "|", bin2hex(sha1($value, true)), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "da39a3ee5e6b4b0d3255bfef95601890afd80709|da39a3ee5e6b4b0d3255bfef95601890afd80709\n",
            "86f7e437faa5a7fce15d1ddcb9eaeaea377667b8|86f7e437faa5a7fce15d1ddcb9eaeaea377667b8\n",
            "a9993e364706816aba3e25717850c26c9cd0d89d|a9993e364706816aba3e25717850c26c9cd0d89d\n",
            "c12252ceda8be8994d5fa0290a47231c1d16aae3|c12252ceda8be8994d5fa0290a47231c1d16aae3\n",
            "32d10c7b8cf96570ca04ce37f2a19d84240d3a89|32d10c7b8cf96570ca04ce37f2a19d84240d3a89\n",
            "761c457bf73b14d27e9e9265c46f4b4dda11f940|761c457bf73b14d27e9e9265c46f4b4dda11f940\n",
            "50abf5706a150990a08b2c5ea40fa0e585554732|50abf5706a150990a08b2c5ea40fa0e585554732\n"
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn sha1_metadata_and_string_dynamic_calls_are_available() {
    let execution = run_source(
        r#"<?php
echo function_exists("sha1") ? "fn" : "missing";
echo "|", is_callable("sha1") ? "callable" : "not";
$call = "sha1";
echo "|", $call("abc");
echo "|", bin2hex($call("abc", true));
$fn = new ReflectionFunction("sha1");
echo "|", $fn->getNumberOfRequiredParameters(), "/", $fn->getNumberOfParameters();
echo ":", $fn->getParameters()[1]->getName();
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "fn|callable|a9993e364706816aba3e25717850c26c9cd0d89d|a9993e364706816aba3e25717850c26c9cd0d89d|1/2:binary"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}
