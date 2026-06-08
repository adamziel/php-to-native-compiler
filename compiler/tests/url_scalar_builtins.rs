use php_compiler::run_source;

#[test]
fn raw_url_encode_decode_cover_rfc3986_unreserved_bytes() {
    let execution = run_source(
        r#"<?php
var_dump(rawurlencode('A1_-.~'));
var_dump(rawurldecode('%41%31%5F%2D%2E%7E'));
echo rawurlencode("a b+%"), "\n";
echo rawurldecode("a%20b%2B%25"), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "string(6) \"A1_-.~\"\nstring(6) \"A1_-.~\"\na%20b%2B%25\na b+%\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn parse_url_returns_bounded_authority_path_query_fragment_arrays() {
    let execution = run_source(
        r#"<?php
var_dump(parse_url("http://example.com/path/script.html?t=1#fragment?data"));
var_dump(parse_url("http://example.com/path/script.html#fragment?data"));
var_dump(parse_url("//example.org"));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "array(5) {\n",
            "  [\"scheme\"]=>\n",
            "  string(4) \"http\"\n",
            "  [\"host\"]=>\n",
            "  string(11) \"example.com\"\n",
            "  [\"path\"]=>\n",
            "  string(17) \"/path/script.html\"\n",
            "  [\"query\"]=>\n",
            "  string(3) \"t=1\"\n",
            "  [\"fragment\"]=>\n",
            "  string(13) \"fragment?data\"\n",
            "}\n",
            "array(4) {\n",
            "  [\"scheme\"]=>\n",
            "  string(4) \"http\"\n",
            "  [\"host\"]=>\n",
            "  string(11) \"example.com\"\n",
            "  [\"path\"]=>\n",
            "  string(17) \"/path/script.html\"\n",
            "  [\"fragment\"]=>\n",
            "  string(13) \"fragment?data\"\n",
            "}\n",
            "array(1) {\n",
            "  [\"host\"]=>\n",
            "  string(11) \"example.org\"\n",
            "}\n",
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn parse_url_splits_bounded_userinfo() {
    let execution = run_source(
        r#"<?php
var_dump(parse_url('http://user:pass@host'));
var_dump(parse_url('//user:pass@host'));
var_dump(parse_url('//user@host'));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "array(4) {\n",
            "  [\"scheme\"]=>\n",
            "  string(4) \"http\"\n",
            "  [\"host\"]=>\n",
            "  string(4) \"host\"\n",
            "  [\"user\"]=>\n",
            "  string(4) \"user\"\n",
            "  [\"pass\"]=>\n",
            "  string(4) \"pass\"\n",
            "}\n",
            "array(3) {\n",
            "  [\"host\"]=>\n",
            "  string(4) \"host\"\n",
            "  [\"user\"]=>\n",
            "  string(4) \"user\"\n",
            "  [\"pass\"]=>\n",
            "  string(4) \"pass\"\n",
            "}\n",
            "array(2) {\n",
            "  [\"host\"]=>\n",
            "  string(4) \"host\"\n",
            "  [\"user\"]=>\n",
            "  string(4) \"user\"\n",
            "}\n",
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn url_scalar_builtin_metadata_is_available_to_capability_checks() {
    let execution = run_source(
        r#"<?php
echo function_exists("rawurlencode") ? "fn" : "missing";
echo "|", is_callable("rawurldecode") ? "callable" : "not";
echo "|", function_exists("parse_url") ? "parse" : "missing";
$encode = "rawurlencode";
$decode = "rawurldecode";
$parse = "parse_url";
$parsed = $parse("//example.org");
echo "|", $encode("~ ");
echo "|", $decode("%7E%20");
echo "|", $parsed["host"];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "fn|callable|parse|~%20|~ |example.org");
    assert_eq!(execution.exit_code, 0);
}
