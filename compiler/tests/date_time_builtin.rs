use php_compiler::run_source;

#[test]
fn scalar_date_builtins_format_mktime_and_checkdate_in_utc() {
    let execution = run_source(
        r#"<?php
date_default_timezone_set("UTC");
$t = mktime(0, 0, 0, 1, 0, 2006);
echo date("Y-m-d H:i:s jS w z n t L a B g G Z U", $t), "\n";
var_dump(checkdate(2, 29, 2006));
var_dump(checkdate(2, 29, 2008));
echo idate("B", mktime(0, 0, 0, 6, 27, 2006)), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "2005-12-31 00:00:00 31st 6 364 12 31 0 am 041 12 0 0 1135987200\nbool(false)\nbool(true)\n41\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn date_default_timezone_state_applies_bounded_offsets() {
    let execution = run_source(
        r#"<?php
date_default_timezone_set("UTC");
$t = mktime(0, 0, 0, 6, 27, 2006);
foreach (["UTC", "Asia/Jerusalem", "America/Chicago", "Europe/London"] as $zone) {
    var_dump(date_default_timezone_set($zone));
    echo date_default_timezone_get(), "|", date("w/z/a/G/Z/U", $t), "\n";
}
var_dump(date_default_timezone_set("Not/AZone"));
echo date_default_timezone_get(), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "bool(true)\nUTC|2/177/am/0/0/1151366400\nbool(true)\nAsia/Jerusalem|2/177/am/3/10800/1151366400\nbool(true)\nAmerica/Chicago|1/176/pm/19/-18000/1151366400\nbool(true)\nEurope/London|2/177/am/1/3600/1151366400\nbool(false)\nEurope/London\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn getdate_and_localtime_expose_ordered_php_arrays() {
    let execution = run_source(
        r#"<?php
date_default_timezone_set("UTC");
$t = mktime(0, 0, 0, 6, 27, 2006);
$date = getdate($t);
echo $date["weekday"], "|", $date["month"], "|", $date["year"], "|", $date[0], "\n";
$local = localtime($t, true);
echo $local["tm_sec"], "|", $local["tm_min"], "|", $local["tm_hour"], "|", $local["tm_mday"], "|", $local["tm_mon"], "|", $local["tm_year"], "|", $local["tm_wday"], "|", $local["tm_yday"], "|", $local["tm_isdst"], "\n";
$numeric = localtime($t);
echo $numeric[0], "|", $numeric[1], "|", $numeric[2], "|", $numeric[3], "|", $numeric[4], "|", $numeric[5], "|", $numeric[6], "|", $numeric[7], "|", $numeric[8], "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Tuesday|June|2006|1151366400\n0|0|0|27|5|106|2|177|0\n0|0|0|27|5|106|2|177|0\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn current_public_date_formats_and_strtotime_subset() {
    let execution = run_source(
        r#"<?php
date_default_timezone_set("Europe/Oslo");
echo date(DATE_ISO8601, strtotime("2005-07-14 22:30:41")), "\n";
echo date(DATE_ISO8601, strtotime("2005-07-14 22:30:41 GMT")), "\n";
echo date(DATE_ISO8601, strtotime("@1121373041 CEST")), "\n";
date_default_timezone_set("UTC");
echo gmdate(DATE_COOKIE, mktime(8, 8, 8, 8, 8, 2008)), "\n";
echo date("r", strtotime("19970523091528")), "\n";
var_dump(strtotime("mayy 2 2009"));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "2005-07-14T22:30:41+0200\n2005-07-15T00:30:41+0200\n2005-07-14T22:30:41+0200\nFriday, 08-Aug-2008 08:08:08 GMT\nFri, 23 May 1997 09:15:28 +0000\nbool(false)\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn timezone_metadata_helpers_cover_current_public_rows() {
    let execution = run_source(
        r#"<?php
$version = timezone_version_get();
echo strpos($version, ".") !== false ? "version" : "bad", "\n";
$zones = timezone_identifiers_list();
echo is_array($zones) ? "array" : "other", "\n";
echo in_array("Europe/London", $zones) ? "london" : "missing", "\n";
echo in_array("America/New_York", $zones) ? "ny" : "missing", "\n";
echo in_array("UTC", $zones) ? "utc" : "missing", "\n";
$filtered = timezone_identifiers_list(DateTimezone::EUROPE | DateTimezone::UTC);
echo in_array("Europe/Oslo", $filtered) ? "oslo" : "missing", "\n";
echo in_array("America/New_York", $filtered) ? "bad" : "no-ny", "\n";
var_dump(timezone_name_from_abbr("CET"));
var_dump(timezone_name_from_abbr("", -14400, 0));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "version\narray\nlondon\nny\nutc\noslo\nno-ny\nstring(13) \"Europe/Berlin\"\nstring(15) \"America/Halifax\"\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn gettimeofday_uses_default_timezone_offset() {
    let execution = run_source(
        r#"<?php
date_default_timezone_set("Asia/Calcutta");
$time = gettimeofday();
echo is_array($time) ? "array" : "other";
echo "|", $time["minuteswest"];
echo "|", $time["dsttime"], "\n";
echo is_float(gettimeofday(true)) ? "float" : "other", "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "array|-330|0\nfloat\n");
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}
