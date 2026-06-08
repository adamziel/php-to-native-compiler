use php_compiler::run_source;

#[test]
fn password_bcrypt_constants_hash_verify_and_crypt_round_trip() {
    let execution = run_source(
        r#"<?php
echo PASSWORD_DEFAULT, "|", PASSWORD_BCRYPT, "|", PASSWORD_BCRYPT_DEFAULT_COST, "\n";
$hash = password_hash("foo", PASSWORD_BCRYPT, ["cost" => 4]);
echo strlen($hash), "|", substr($hash, 0, 7), "\n";
var_dump(password_verify("foo", $hash));
var_dump($hash === crypt("foo", $hash));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "2y|2y|12\n60|$2y$04$\nbool(true)\nbool(true)\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn password_get_info_and_needs_rehash_use_bcrypt_metadata() {
    let execution = run_source(
        r#"<?php
$hash = '$2y$10$MTIzNDU2Nzg5MDEyMzQ1Nej0NmcAWSLR.oP7XOR9HD/vjUuOj100y';
$info = password_get_info($hash);
echo $info["algo"], "|", $info["algoName"], "|", $info["options"]["cost"], "\n";
var_dump(password_needs_rehash($hash, PASSWORD_BCRYPT, ["cost" => 10]));
var_dump(password_needs_rehash($hash, PASSWORD_BCRYPT, ["cost" => 11]));
var_dump(password_needs_rehash("", PASSWORD_BCRYPT));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "2y|bcrypt|10\nbool(false)\nbool(true)\nbool(true)\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn password_errors_are_catchable_and_mask_sensitive_arguments() {
    let execution = run_source(
        r#"<?php
try {
    password_hash("secret");
} catch (Throwable $e) {
    echo get_class($e), ":", $e->getMessage(), "\n";
    echo strpos((string) $e, "password_hash(Object(SensitiveParameterValue))") !== false ? "masked" : "leaked";
    echo "\n";
}
try {
    password_hash("secret", []);
} catch (Throwable $e) {
    echo get_class($e), ":", $e->getMessage(), "\n";
}
try {
    password_hash("null\0password", PASSWORD_BCRYPT);
} catch (Throwable $e) {
    echo get_class($e), ":", $e->getMessage(), "\n";
}
try {
    password_verify([], "hash");
} catch (Throwable $e) {
    echo strpos((string) $e, "password_verify(Object(SensitiveParameterValue), 'hash')") !== false ? "masked" : "leaked";
    echo "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "ArgumentCountError:password_hash() expects at least 2 arguments, 1 given\nmasked\nTypeError:password_hash(): Argument #2 ($algo) must be of type string|int|null, array given\nValueError:Bcrypt password must not contain null character\nmasked\n"
    );
    assert_eq!(execution.exit_code, 0);
}
