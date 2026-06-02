use php_compiler::run_source;

#[test]
fn filter_metadata_lists_ids_and_absent_inputs() {
    let execution = run_source(
        r#"<?php
$filters = filter_list();
echo count($filters), "|", $filters[0], "|", $filters[20], "\n";
var_dump(filter_id("stripped"));
var_dump(filter_id("string"));
var_dump(filter_id("url"));
var_dump(filter_id("int"));
var_dump(filter_id("none"));
var_dump(filter_id(-1));
var_dump(filter_input(INPUT_GET, "missing"));
var_dump(filter_input(INPUT_GET, "missing", FILTER_DEFAULT, FILTER_NULL_ON_FAILURE));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "21|int|callback\nint(513)\nint(513)\nint(518)\nint(257)\nbool(false)\nbool(false)\nNULL\nbool(false)\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn filter_var_validates_scalar_arrays_and_null_on_failure() {
    let execution = run_source(
        r#"<?php
$ints = filter_var(array(1, "1", "", "-23234", "text", "asdf234asdfgs", array()), FILTER_VALIDATE_INT, FILTER_REQUIRE_ARRAY);
echo $ints[0], "|", $ints[1], "|", $ints[3], "\n";
var_dump($ints[2], $ints[4], $ints[5], $ints[6]);
$floats = filter_var(array(1.2, "1.7", "", "-23234.123", "text", "asdf234.2asdfgs", array()), FILTER_VALIDATE_FLOAT, FILTER_REQUIRE_ARRAY);
echo $floats[0], "|", $floats[1], "|", $floats[3], "\n";
var_dump($floats[2], $floats[4], $floats[5], $floats[6]);
var_dump(filter_var("invalid", FILTER_VALIDATE_BOOL, FILTER_NULL_ON_FAILURE));
var_dump(filter_var("invalid", FILTER_VALIDATE_INT, FILTER_NULL_ON_FAILURE));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "1|1|-23234\nbool(false)\nbool(false)\nbool(false)\narray(0) {\n}\n1.2|1.7|-23234.123\nbool(false)\nbool(false)\nbool(false)\narray(0) {\n}\nNULL\nNULL\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn filter_var_rejects_invalid_url_and_domain_authority_forms() {
    let execution = run_source(
        r#"<?php
foreach ([
    "http://php.net\\@aliyun.com/aaa.do",
    "https://example.com\\uFF03@bing.com",
    "https://example.com:\\@test.com/",
    "https://user:\\epass@test.com",
    "https://user:\\@test.com",
] as $url) {
    var_dump(filter_var($url, FILTER_VALIDATE_URL));
}
var_dump(filter_var(".invalid", FILTER_VALIDATE_DOMAIN, FILTER_NULL_ON_FAILURE));
var_dump(filter_var("example.com", FILTER_VALIDATE_DOMAIN, FILTER_NULL_ON_FAILURE));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "bool(false)\n",
            "bool(false)\n",
            "bool(false)\n",
            "bool(false)\n",
            "bool(false)\n",
            "NULL\n",
            "string(11) \"example.com\"\n",
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn filter_var_validate_ip_rejects_bounded_ipv6_private_and_reserved_ranges() {
    let execution = run_source(
        r#"<?php
foreach ([
    ["FC00::1", FILTER_FLAG_IPV6 | FILTER_FLAG_NO_PRIV_RANGE],
    ["fdff:ffff:ffff:ffff:ffff:ffff:ffff:ffff", FILTER_FLAG_IPV6 | FILTER_FLAG_NO_PRIV_RANGE],
    ["::", FILTER_FLAG_IPV6 | FILTER_FLAG_NO_RES_RANGE],
    ["::1", FILTER_FLAG_NO_RES_RANGE],
    ["0:0:0:0:0:0:0:1", FILTER_FLAG_NO_RES_RANGE],
    ["fe80:5:6::1", FILTER_FLAG_IPV6 | FILTER_FLAG_NO_RES_RANGE],
    ["::ffff:0:1", FILTER_FLAG_NO_RES_RANGE],
    ["2001:0db8::1", FILTER_FLAG_IPV6 | FILTER_FLAG_NO_RES_RANGE],
    ["2001:0010::1", FILTER_FLAG_IPV6 | FILTER_FLAG_NO_RES_RANGE],
    ["240b:0010::1", FILTER_FLAG_IPV6 | FILTER_FLAG_NO_RES_RANGE],
    ["::ffff:192.168.1.1", FILTER_FLAG_NO_PRIV_RANGE],
] as $case) {
    var_dump(filter_var($case[0], FILTER_VALIDATE_IP, $case[1]));
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "bool(false)\n",
            "bool(false)\n",
            "bool(false)\n",
            "bool(false)\n",
            "bool(false)\n",
            "bool(false)\n",
            "bool(false)\n",
            "string(12) \"2001:0db8::1\"\n",
            "string(12) \"2001:0010::1\"\n",
            "string(12) \"240b:0010::1\"\n",
            "string(18) \"::ffff:192.168.1.1\"\n",
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn filter_validator_edge_flags_and_ranges_match_php_boundaries() {
    let execution = run_source(
        r#"<?php
foreach ([
    "0x7fffffffffffffff",
    "0x8000000000000000",
    "0xffffffffffffffff",
    "0x10000000000000000",
    "0777777777777777777777",
    "01000000000000000000000",
    "01777777777777777777777",
    "02000000000000000000000",
] as $value) {
    var_dump(filter_var($value, FILTER_VALIDATE_INT, FILTER_FLAG_ALLOW_HEX | FILTER_FLAG_ALLOW_OCTAL));
}
var_dump(defined("FILTER_FLAG_GLOBAL_RANGE"));
var_dump(FILTER_FLAG_GLOBAL_RANGE);
var_dump(FILTER_FLAG_HOSTNAME);
foreach ([
    "0.0.0.0",
    "100.127.255.255",
    "192.88.99.1",
    "185.85.0.29",
    "::",
    "::ffff:ffff:ffff",
    "64:ff9b::",
    "100::ffff:ffff:ffff:ffff",
    "2001:1f:ffff:ffff:ffff:ffff:ffff:ffff",
    "240b:10::1",
] as $ip) {
    var_dump(filter_var($ip, FILTER_VALIDATE_IP, FILTER_FLAG_GLOBAL_RANGE));
}
foreach ([
    "0.255.255.255",
    "127.255.255.255",
    "169.254.0.0",
    "224.0.0.0",
    "240.0.0.0",
    "255.255.255.255",
] as $ip) {
    var_dump(filter_var($ip, FILTER_VALIDATE_IP, FILTER_FLAG_IPV4 | FILTER_FLAG_NO_RES_RANGE));
}
foreach ([
    "http://t[est@127.0.0.1",
    "http://t[est@[::1]",
    "http://test@127.0.0.1",
    "http://test@[2001:db8:3333:4444:5555:6666:1.2.3.4]",
    "http://test@[::1]",
] as $url) {
    var_dump(filter_var($url, FILTER_VALIDATE_URL));
}
foreach (["a-.bc.com", "a.bc-.com", "a.bc.com-"] as $domain) {
    var_dump(filter_var($domain, FILTER_VALIDATE_DOMAIN, FILTER_FLAG_HOSTNAME));
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "int(9223372036854775807)\n",
            "int(-9223372036854775808)\n",
            "int(-1)\n",
            "bool(false)\n",
            "int(9223372036854775807)\n",
            "int(-9223372036854775808)\n",
            "int(-1)\n",
            "bool(false)\n",
            "bool(true)\n",
            "int(268435456)\n",
            "int(1048576)\n",
            "bool(false)\n",
            "bool(false)\n",
            "string(11) \"192.88.99.1\"\n",
            "string(11) \"185.85.0.29\"\n",
            "bool(false)\n",
            "bool(false)\n",
            "string(9) \"64:ff9b::\"\n",
            "bool(false)\n",
            "bool(false)\n",
            "string(10) \"240b:10::1\"\n",
            "bool(false)\n",
            "bool(false)\n",
            "bool(false)\n",
            "string(9) \"224.0.0.0\"\n",
            "bool(false)\n",
            "bool(false)\n",
            "bool(false)\n",
            "bool(false)\n",
            "string(21) \"http://test@127.0.0.1\"\n",
            "string(50) \"http://test@[2001:db8:3333:4444:5555:6666:1.2.3.4]\"\n",
            "string(17) \"http://test@[::1]\"\n",
            "bool(false)\n",
            "bool(false)\n",
            "bool(false)\n",
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn filter_var_sanitizes_scalars_and_warns_for_unknown_filters() {
    let execution = run_source(
        r#"<?php
var_dump(filter_var(1, FILTER_SANITIZE_SPECIAL_CHARS, 1));
var_dump(filter_var(1, FILTER_SANITIZE_SPECIAL_CHARS, 0));
var_dump(filter_var(1, FILTER_SANITIZE_SPECIAL_CHARS, array()));
var_dump(filter_var("<>&\"'plain", FILTER_SANITIZE_SPECIAL_CHARS));
var_dump(filter_var(array("<tag>", "safe"), FILTER_SANITIZE_SPECIAL_CHARS, FILTER_REQUIRE_ARRAY));
var_dump(filter_var(1, -1, array(123)));
"#,
    )
    .unwrap();

    assert!(execution
        .stdout
        .contains("string(1) \"1\"\nstring(1) \"1\"\nstring(1) \"1\""));
    assert!(
        execution
            .stdout
            .contains("string(30) \"&#60;&#62;&#38;&#34;&#39;plain\""),
        "{}",
        execution.stdout
    );
    assert!(
        execution.stdout.contains("string(13) \"&#60;tag&#62;\""),
        "{}",
        execution.stdout
    );
    assert!(
        execution
            .stdout
            .contains("Warning: filter_var(): Unknown filter with ID -1"),
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

#[test]
fn filter_options_float_ranges_and_callback_arrays_match_php_boundaries() {
    let execution = run_source(
        r#"<?php
function filter_cb($var) {
    return 1;
}

$options = array("flags" => (string) FILTER_FLAG_ALLOW_HEX, "options" => array("min_range" => "0", "max_range" => "1024"));
var_dump(filter_var("0xff", FILTER_VALIDATE_INT, $options));
var_dump(filter_var("0xff", (string) FILTER_VALIDATE_INT, $options));
echo gettype($options["flags"]), "|", $options["options"]["min_range"], "\n";

$grouped = filter_var("1,234,567,890.1234567165", FILTER_VALIDATE_FLOAT, array("flags" => FILTER_FLAG_ALLOW_THOUSAND));
var_dump($grouped > 1234567890 && $grouped < 1234567891);
var_dump(filter_var("1234,567,890.1", FILTER_VALIDATE_FLOAT, array("flags" => FILTER_FLAG_ALLOW_THOUSAND)));
var_dump(filter_var("1e-324", FILTER_VALIDATE_FLOAT));
var_dump(filter_var("1000", FILTER_VALIDATE_FLOAT, array("options" => array("max_range" => 999.999, "default" => 0))));

$data = array("bar" => array("fu<script>bar", "bar<script>fu"));
var_dump(filter_var($data, FILTER_CALLBACK, array("options" => "filter_cb")));
var_dump($data);
var_dump(filter_var_array(array("test" => "0xff"), array("test" => array("filter" => (string) FILTER_VALIDATE_INT, "flags" => (string) FILTER_FLAG_ALLOW_HEX))));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "int(255)\n",
            "int(255)\n",
            "string|0\n",
            "bool(true)\n",
            "bool(false)\n",
            "bool(false)\n",
            "int(0)\n",
            "array(1) {\n",
            "  [\"bar\"]=>\n",
            "  array(2) {\n",
            "    [0]=>\n",
            "    int(1)\n",
            "    [1]=>\n",
            "    int(1)\n",
            "  }\n",
            "}\n",
            "array(1) {\n",
            "  [\"bar\"]=>\n",
            "  array(2) {\n",
            "    [0]=>\n",
            "    string(13) \"fu<script>bar\"\n",
            "    [1]=>\n",
            "    string(13) \"bar<script>fu\"\n",
            "  }\n",
            "}\n",
            "array(1) {\n",
            "  [\"test\"]=>\n",
            "  int(255)\n",
            "}\n",
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn filter_byte_flags_options_and_input_defaults_cover_bounded_rows() {
    let execution = run_source(
        r#"<?php
$flags = FILTER_FLAG_ENCODE_LOW | FILTER_FLAG_ENCODE_HIGH | FILTER_FLAG_ENCODE_AMP;
var_dump(filter_var(chr(0) . "&" . chr(127) . chr(255), FILTER_UNSAFE_RAW, array("flags" => $flags)));
var_dump(filter_var("``a`" . chr(127), FILTER_UNSAFE_RAW, FILTER_FLAG_STRIP_BACKTICK | FILTER_FLAG_STRIP_HIGH));
var_dump(filter_var(chr(127), FILTER_SANITIZE_ENCODED, FILTER_FLAG_STRIP_HIGH));
var_dump(filter_var("", FILTER_DEFAULT, FILTER_FLAG_EMPTY_STRING_NULL));
var_dump(filter_var("1 234.5", FILTER_VALIDATE_FLOAT, array("flags" => FILTER_FLAG_ALLOW_THOUSAND, "options" => array("thousand" => " "))));
try {
    filter_var("123", FILTER_VALIDATE_FLOAT, array("flags" => FILTER_FLAG_ALLOW_THOUSAND, "options" => array("thousand" => "")));
} catch (ValueError $e) {
    echo $e->getMessage(), "\n";
}
var_dump(filter_var("01-23-45-67-89-ab", FILTER_VALIDATE_MAC));
var_dump(filter_var("0123.4567.89ab", FILTER_VALIDATE_MAC));
var_dump(filter_var("01-23-45-67-89-ab", FILTER_VALIDATE_MAC, array("options" => array("separator" => ":"))));
try {
    filter_var("01-23-45-67-89-ab", FILTER_VALIDATE_MAC, array("options" => array("separator" => "--")));
} catch (ValueError $e) {
    echo $e->getMessage(), "\n";
}
var_dump(filter_input(INPUT_SERVER, "PHP_SELF"));
var_dump(filter_input(INPUT_GET, "missing", FILTER_VALIDATE_INT, array("flags" => FILTER_REQUIRE_SCALAR, "options" => array("default" => 23))));
var_dump(filter_var_array(array("test" => "42"), array("test" => FILTER_VALIDATE_INT | FILTER_NULL_ON_FAILURE)));
"#,
    )
    .unwrap();

    assert!(execution
        .stdout
        .contains("string(21) \"&#0;&#38;&#127;&#255;\""));
    assert!(execution.stdout.contains("string(1) \"a\""));
    assert!(execution.stdout.contains("string(0) \"\"\nNULL\n"));
    assert!(execution.stdout.contains("float(1234.5)"));
    assert!(execution
        .stdout
        .contains("filter_var(): \"thousand\" option must not be empty"));
    assert!(execution
        .stdout
        .contains("string(17) \"01-23-45-67-89-ab\""));
    assert!(execution.stdout.contains("string(14) \"0123.4567.89ab\""));
    assert!(execution
        .stdout
        .contains("filter_var(): \"separator\" option must be one character long"));
    assert!(execution.stdout.contains("string(10) \"/index.php\""));
    assert!(execution.stdout.contains("int(23)"));
    assert!(execution
        .stdout
        .contains("Warning: filter_var_array(): Unknown filter with ID 134217985"));
    assert!(execution.stdout.contains("string(2) \"42\""));
    assert_eq!(execution.exit_code, 0);
}
