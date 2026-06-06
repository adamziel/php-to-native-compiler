use php_compiler::run_source;

#[test]
fn strftime_and_gmstrftime_cover_c_locale_date_tokens() {
    let execution = run_source(
        r#"<?php
var_dump(defined("LC_ALL"));
var_dump(function_exists("strftime"));
var_dump(function_exists("gmstrftime"));
var_dump(setlocale(LC_ALL, "C"));
error_reporting(24575);
date_default_timezone_set("UTC");
$t = mktime(8, 8, 8, 8, 8, 2008);
echo strftime("%b %d %Y %H:%M:%S", $t), "\n";
echo gmstrftime("%A|%a|%B|%h|%e|%j|%u|%w|%U|%W", $t), "\n";
echo strftime("%F|%D|%R|%T|%r|%p|%P|%%", $t), "\n";
echo strftime("%x|%X|%z|%Z", $t), "\n";
var_dump(strftime("", $t));
var_dump(gmstrftime("", $t));
date_default_timezone_set("Asia/Calcutta");
echo strftime("%z|%Z|%H:%M", 0), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "bool(true)\nbool(true)\nbool(true)\nstring(1) \"C\"\nAug 08 2008 08:08:08\nFriday|Fri|August|Aug| 8|221|5|5|31|31\n2008-08-08|08/08/08|08:08|08:08:08|08:08:08 AM|AM|am|%\n08/08/08|08:08:08|+0000|GMT\nbool(false)\nbool(false)\n+0530|IST|05:30\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn strftime_deprecation_uses_php_display_diagnostic_and_block_semicolon_is_empty_statement() {
    let execution = run_source(
        r#"<?php
$inputs = array("local" => "strftime", "utc" => "gmstrftime");
foreach ($inputs as $label => $function) {
    echo $label, "\n";
    echo $function("%b", 0), "\n";
};
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "local\n\nDeprecated: Function strftime() is deprecated since 8.1, use IntlDateFormatter::format() instead in Command line code on line 5\nJan\nutc\n\nDeprecated: Function gmstrftime() is deprecated since 8.1, use IntlDateFormatter::format() instead in Command line code on line 5\nJan\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}
