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
fn parse_url_query_and_fragment_only_inputs_do_not_materialize_empty_path() {
    let execution = run_source(
        r##"<?php
foreach (["?", "#", "?q", "#f", "?#", "?#f", "?q#f"] as $url) {
    $parts = parse_url($url);
    echo $url, "|";
    echo array_key_exists("path", $parts) ? "path" : "no-path";
    echo "|", parse_url($url, PHP_URL_PATH) === null ? "null" : "path";
    echo "|", parse_url($url, PHP_URL_QUERY) === null ? "null" : parse_url($url, PHP_URL_QUERY);
    echo "|", parse_url($url, PHP_URL_FRAGMENT) === null ? "null" : parse_url($url, PHP_URL_FRAGMENT);
    echo "\n";
}
"##,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "?|no-path|null||null\n",
            "#|no-path|null|null|\n",
            "?q|no-path|null|q|null\n",
            "#f|no-path|null|null|f\n",
            "?#|no-path|null||\n",
            "?#f|no-path|null||f\n",
            "?q#f|no-path|null|q|f\n",
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn raw_url_encoding_matches_rfc3986_percent_boundaries() {
    let execution = run_source(
        r#"<?php
echo urlencode("A1_-.~ +/%"), "\n";
echo rawurlencode("A1_-.~ +/%"), "\n";
echo rawurldecode("%41%31%5F%2D%2E%7E%20%2B%2F%25"), "\n";
echo urlencode("\xA3"), "\n";
echo bin2hex(rawurldecode("%00%FF%7E")), "\n";
echo function_exists("rawurlencode") ? "fn" : "missing";
echo "|", is_callable("rawurldecode") ? "callable" : "missing";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "A1_-.%7E+%2B%2F%25\nA1_-.~%20%2B%2F%25\nA1_-.~ +/%\n%A3\n00ff7e\nfn|callable"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn url_helpers_accept_stringable_objects_and_report_type_errors() {
    let execution = run_source(
        r#"<?php
class UrlText {
    public function __toString() { return "http://example.test/a b?x=1+2"; }
}
class EncodedText {
    public function __toString() { return "%41+%2B"; }
}
class PrefixText {
    public function __toString() { return "pre_"; }
}
class SeparatorText {
    public function __toString() { return "|"; }
}

var_dump(parse_url(new UrlText(), PHP_URL_HOST));
echo urlencode(new UrlText()), "\n";
echo rawurlencode(new UrlText()), "\n";
echo rawurldecode(new EncodedText()), "\n";
echo http_build_query([1], new PrefixText()), "\n";
echo http_build_query(["a" => 1, "b" => 2], "", new SeparatorText()), "\n";
var_dump(parse_url(null, PHP_URL_PATH));
try { parse_url(new stdClass()); } catch (Throwable $e) { echo "parse-object-caught\n"; }
try { urlencode([]); } catch (Throwable $e) { echo "urlencode-array-caught\n"; }
try { http_build_query([1], []); } catch (Throwable $e) { echo "prefix-array-caught\n"; }
try { http_build_query(["a" => 1], "", []); } catch (Throwable $e) { echo "separator-array-caught\n"; }
"#,
    )
    .unwrap();

    assert!(
        execution
            .stdout
            .contains("Deprecated: parse_url(): Passing null to parameter #1 ($url) of type string is deprecated"),
        "{}",
        execution.stdout
    );
    assert!(execution.stdout.contains("string(12) \"example.test\""));
    assert!(execution
        .stdout
        .contains("http%3A%2F%2Fexample.test%2Fa+b%3Fx%3D1%2B2\n"));
    assert!(execution
        .stdout
        .contains("http%3A%2F%2Fexample.test%2Fa%20b%3Fx%3D1%2B2\n"));
    assert!(execution.stdout.contains("A++\npre_0=1\na=1|b=2\n"));
    assert!(execution.stdout.contains("string(0) \"\""));
    assert!(execution.stdout.contains("parse-object-caught\n"));
    assert!(execution.stdout.contains("urlencode-array-caught\n"));
    assert!(execution.stdout.contains("prefix-array-caught\n"));
    assert!(execution.stdout.contains("separator-array-caught\n"));
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
