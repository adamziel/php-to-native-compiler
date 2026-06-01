use php_compiler::{emit_ir_source, run_source};

#[test]
fn parse_url_extracts_full_url_parts_and_relative_authority() {
    let execution = run_source(
        r#"<?php
$url = parse_url("http://secret:hideout@www.php.net:80/index.php?test=1#frag");
echo $url["scheme"], "|", $url["host"], "|", $url["port"], "|", $url["user"], "|", $url["pass"], "|", $url["path"], "|", $url["query"], "|", $url["fragment"], "\n";
$relative = parse_url("//example.org:81/hi?a=b#c=d");
echo $relative["host"], "|", $relative["port"], "|", $relative["path"], "|", $relative["query"], "|", $relative["fragment"], "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "http|www.php.net|80|secret|hideout|/index.php|test=1|frag\nexample.org|81|/hi|a=b|c=d\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn parse_url_supports_component_constants_and_null_missing_parts() {
    let execution = run_source(
        r#"<?php
echo PHP_URL_SCHEME, PHP_URL_HOST, PHP_URL_PORT, PHP_URL_USER, PHP_URL_PASS, PHP_URL_PATH, PHP_URL_QUERY, PHP_URL_FRAGMENT, "\n";
var_dump(parse_url("www.php.net:80/index.php?x=1#frag", PHP_URL_SCHEME));
echo parse_url("www.php.net:80/index.php?x=1#frag", PHP_URL_HOST), "|";
echo parse_url("www.php.net:80/index.php?x=1#frag", PHP_URL_PORT), "|";
echo parse_url("www.php.net:80/index.php?x=1#frag", PHP_URL_PATH), "|";
echo parse_url("www.php.net:80/index.php?x=1#frag", PHP_URL_QUERY), "|";
echo parse_url("www.php.net:80/index.php?x=1#frag", PHP_URL_FRAGMENT), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "01234567\nNULL\nwww.php.net|80|/index.php|x=1|frag\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn raw_url_encoding_matches_rfc3986_percent_boundaries() {
    let execution = run_source(
        r#"<?php
echo urlencode("a b+c/%\0"), "\n";
echo urldecode("a+b%2Bc%2F%25%00"), "\n";
echo bin2hex(urldecode("search%e4")), "\n";
echo rawurlencode("A1_-.~ +/%"), "\n";
echo rawurldecode("%41%31%5F%2D%2E%7E%20%2B%2F%25"), "\n";
echo bin2hex(rawurldecode("%00%FF%7E")), "\n";
echo function_exists("urldecode") ? "fn" : "missing";
echo "|", is_callable("rawurldecode") ? "callable" : "missing";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "a+b%2Bc%2F%25%00\na b+c/%\0\n736561726368e4\nA1_-.~%20%2B%2F%25\nA1_-.~ +/%\n00ff7e\nfn|callable"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn parse_url_matches_bounded_edge_cases_from_url_phpts() {
    let execution = run_source(
        r#"<?php
echo parse_url("127.0.0.1:9999?", PHP_URL_SCHEME), "|", parse_url("127.0.0.1:9999?", PHP_URL_PATH), "|", parse_url("127.0.0.1:9999?", PHP_URL_QUERY), "\n";
echo parse_url("file:///a:/", PHP_URL_PATH), "|", parse_url("file:///:80/", PHP_URL_PATH), "|", parse_url("http://::?", PHP_URL_HOST), "|", parse_url("x://::6.5", PHP_URL_PORT), "\n";
var_dump(parse_url(":"));
var_dump(parse_url("http://blah.com:123456"));
var_dump(parse_url("http://blah.com:abcdef", PHP_URL_HOST));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "127.0.0.1|9999|\na:/|/:80/|:|6\nbool(false)\nbool(false)\nbool(false)\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn parse_url_invalid_component_raises_value_error() {
    let execution = run_source(
        r#"<?php
try {
    parse_url("http://www.php.net", 99);
} catch (ValueError $exception) {
    echo $exception->getMessage();
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "parse_url(): Argument #2 ($component) must be a valid URL component identifier, 99 given"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_folds_parse_url_metadata_and_component_constants() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("parse_url") ? "1" : "0";
echo defined("PHP_URL_SCHEME") ? "1" : "0";
echo defined("PHP_URL_FRAGMENT") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 3, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("PHP_URL_SCHEME"), "{ir}");
    assert!(!ir.contains("PHP_URL_FRAGMENT"), "{ir}");
}
