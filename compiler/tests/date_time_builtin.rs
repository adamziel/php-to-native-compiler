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
fn idate_supports_iso_weekday_and_week_year_tokens() {
    let execution = run_source(
        r#"<?php
date_default_timezone_set("UTC");
foreach (["2018-12-31", "2021-01-03", "2021-01-04"] as $date) {
    $timestamp = strtotime($date);
    echo $date, "|N=", idate("N", $timestamp);
    echo "|W=", idate("W", $timestamp);
    echo "|o=", idate("o", $timestamp);
    echo "|Y=", idate("Y", $timestamp), "\n";
}
var_dump(idate("O", strtotime("2021-01-01")));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "2018-12-31|N=1|W=1|o=2019|Y=2018\n",
            "2021-01-03|N=7|W=53|o=2020|Y=2021\n",
            "2021-01-04|N=1|W=1|o=2021|Y=2021\n",
            "\nWarning: idate(): Unrecognized date format token in Command line code on line 10\n",
            "bool(false)\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn datetime_timestamp_helpers_get_and_set_bounded_state() {
    let execution = run_source(
        r#"<?php
date_default_timezone_set("UTC");
$date = date_create("1970-01-01T00:00:00UTC");
var_dump(date_timestamp_get($date));
var_dump($date->getTimeStamp() === 0);
date_timestamp_set($date, 1234567890);
echo date_format($date, "B => (U) => T Y-M-d H:i:s"), "\n";
date_default_timezone_set("Europe/Oslo");
$at = new DateTime("@1217184864");
echo $at->format("Y-m-d H:i e"), "\n";
$local = new DateTime();
$local->setTimestamp(1217184864);
echo $local->format("Y-m-d H:i e"), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "int(0)\n",
            "bool(true)\n",
            "021 => (1234567890) => UTC 2009-Feb-13 23:31:30\n",
            "2008-07-27 18:54 +00:00\n",
            "2008-07-27 20:54 Europe/Oslo\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn datetime_constructor_timezone_argument_matches_bounded_rows() {
    let execution = run_source(
        r#"<?php
date_default_timezone_set("Europe/Oslo");
$local = new DateTime("2009-01-01 00:00:00", new DateTimeZone("Europe/Oslo"));
echo $local->format("Y-m-d H:i:s T e"), "\n";
$procedural = date_create("2009-01-01 00:00:00", new DateTimeZone("America/New_York"));
echo $procedural->format("Y-m-d H:i:s T e"), "\n";
$null = date_create("2009-01-01", null);
echo $null->format(DateTime::COOKIE), "\n";
$explicit = new DateTime("2009-01-01 00:00:00 GMT", new DateTimeZone("Europe/Oslo"));
echo $explicit->getTimezone()->getName(), "|", $explicit->format("Y-m-d H:i:s T"), "\n";
$timestamp = new DateTime("@0", new DateTimeZone(date_default_timezone_get()));
echo $timestamp->getTimezone()->getName(), "|", $timestamp->format("Y-m-d H:i:s"), "\n";
$timestamp->setTimezone(new DateTimeZone(date_default_timezone_get()));
echo $timestamp->getTimezone()->getName(), "|", $timestamp->format("Y-m-d H:i:s"), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "2009-01-01 00:00:00 CET Europe/Oslo\n",
            "2009-01-01 00:00:00 EST America/New_York\n",
            "Thursday, 01-Jan-2009 00:00:00 CET\n",
            "GMT|2009-01-01 00:00:00 GMT\n",
            "+00:00|1970-01-01 00:00:00\n",
            "Europe/Oslo|1970-01-01 01:00:00\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn datetime_core_constructors_report_catchable_argument_count_errors() {
    let execution = run_source(
        r#"<?php
date_default_timezone_set("UTC");
try {
    new DateTime("GMT", timezone_open("GMT"), 99);
} catch (TypeError $e) {
    echo $e::class, ": ", $e->getMessage(), "\n";
}
try {
    new DateTimeZone();
} catch (TypeError $e) {
    echo $e::class, ": ", $e->getMessage(), "\n";
}
try {
    new DateTimeZone("GMT", 99);
} catch (TypeError $e) {
    echo $e::class, ": ", $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "ArgumentCountError: DateTime::__construct() expects at most 2 arguments, 3 given\n",
            "ArgumentCountError: DateTimeZone::__construct() expects exactly 1 argument, 0 given\n",
            "ArgumentCountError: DateTimeZone::__construct() expects exactly 1 argument, 2 given\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn datetime_set_state_recreates_exported_bounded_state() {
    let execution = run_source(
        r#"<?php
date_default_timezone_set("Europe/London");
$manual = DateTime::__set_state(array(
    "date" => "2017-10-06 23:30:00.000000",
    "timezone_type" => 3,
    "timezone" => "UTC",
));
echo $manual->format("Y-m-d H:i:s T e"), "\n";
$original = new DateTime("2017-10-06 23:30:00", new DateTimeZone("UTC"));
$state = var_export($original, true);
eval("\$copy = {$state};");
echo $copy->format("Y-m-d H:i:s T e"), "\n";
echo ($copy === $original ? "same" : "different"), "\n";
echo is_callable(array("DateTime", "__set_state")) ? "callable" : "missing";
echo "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "2017-10-06 23:30:00 UTC UTC\n",
            "2017-10-06 23:30:00 UTC UTC\n",
            "different\n",
            "callable\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn date_modify_mutates_bounded_datetime_relative_forms() {
    let execution = run_source(
        r#"<?php
date_default_timezone_set("Europe/London");
$datetime = date_create("2009-01-31 14:28:41");
date_modify($datetime, "+1 day");
echo date_format($datetime, "D, d M Y"), "\n";
$datetime->modify("+1 week 2 days 4 hours 2 seconds");
echo date_format($datetime, "D, d M Y H:i:s"), "\n";
date_modify($datetime, "next Thursday");
echo date_format($datetime, "D, d M Y"), "\n";
$datetime->modify("last Sunday");
echo date_format($datetime, "D, d M Y"), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "Sun, 01 Feb 2009\n",
            "Tue, 10 Feb 2009 18:28:43\n",
            "Thu, 12 Feb 2009\n",
            "Sun, 08 Feb 2009\n"
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn date_modify_helpers_use_php_string_modifier_boundary() {
    let execution = run_source(
        r#"<?php
date_default_timezone_set("UTC");
class NextDay { public function __toString() { return "+1 day"; } }
$datetime = new DateTime("2001-02-03 04:05:06");
$returned = $datetime->modify(new NextDay());
echo $datetime->format("Y-m-d"), "|", ($returned === $datetime ? "same" : "different"), "\n";
$call = "date_modify";
$returned = $call($datetime, new NextDay());
echo $datetime->format("Y-m-d"), "|", ($returned === $datetime ? "same" : "different"), "\n";
try { $datetime->modify(array()); } catch (Throwable $e) { echo get_class($e), ": ", $e->getMessage(), "\n"; }
try { date_modify($datetime, array()); } catch (Throwable $e) { echo get_class($e), ": ", $e->getMessage(), "\n"; }
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "2001-02-04|same\n",
            "2001-02-05|same\n",
            "TypeError: DateTime::modify(): Argument #1 ($modifier) must be of type string, array given\n",
            "TypeError: date_modify(): Argument #2 ($modifier) must be of type string, array given\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn datetime_modify_at_timestamp_uses_unix_timestamp_timezone_identity() {
    let execution = run_source(
        r#"<?php
$m = new DateTime("2022-12-20 14:30:25", new DateTimeZone("Europe/Paris"));
$returned = $m->modify("@1234567890");
var_dump($m->getTimeStamp());
echo $m->getTimezone()->getName(), "|", $m->format(DateTime::ATOM), "|";
echo ($returned === $m ? "same" : "different"), "\n";

$a = new DateTime("2022-11-01 13:30:00", new DateTimezone("America/Lima"));
$b = clone $a;
echo $a->format(DateTime::ATOM), "|", $a->getTimestamp(), "|", $a->format("T e Z"), "\n";
$a->modify("@" . $a->getTimestamp());
$b->setTimestamp($b->getTimestamp());
echo $a->format(DateTime::ATOM), "|", $a->getTimezone()->getName(), "\n";
echo $b->format(DateTime::ATOM), "|", $b->getTimezone()->getName(), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "int(1234567890)\n",
            "+00:00|2009-02-13T23:31:30+00:00|same\n",
            "2022-11-01T13:30:00-05:00|1667327400|-05 America/Lima -18000\n",
            "2022-11-01T18:30:00+00:00|+00:00\n",
            "2022-11-01T13:30:00-05:00|America/Lima\n",
        )
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
        "bool(true)\nUTC|2/177/am/0/0/1151366400\nbool(true)\nAsia/Jerusalem|2/177/am/3/10800/1151366400\nbool(true)\nAmerica/Chicago|1/176/pm/19/-18000/1151366400\nbool(true)\nEurope/London|2/177/am/1/3600/1151366400\n\nNotice: date_default_timezone_set(): Timezone ID 'Not/AZone' is invalid in Command line code on line 8\nbool(false)\nEurope/London\n"
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
fn date_formatters_use_php_string_argument_boundary() {
    let execution = run_source(
        r#"<?php
date_default_timezone_set("UTC");
class YearFormat { public function __toString() { return "Y"; } }
$date = new DateTime("2001-02-03 04:05:06", new DateTimeZone("UTC"));
var_dump($date->format(123));
var_dump(date_format($date, new YearFormat()));
var_dump(date(new YearFormat(), 0));
var_dump(gmdate(new YearFormat(), 0));
var_dump(idate("Y", 0));
var_dump(idate(123, 0));
var_dump($date->format(null));
try { date_format($date, array()); } catch (Throwable $e) { echo get_class($e), ": ", $e->getMessage(), "\n"; }
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "string(3) \"123\"\n",
            "string(4) \"2001\"\n",
            "string(4) \"1970\"\n",
            "string(4) \"1970\"\n",
            "int(1970)\n",
            "\nWarning: idate(): idate format is one char in Command line code on line 10\n",
            "bool(false)\n",
            "\nDeprecated: DateTime::format(): Passing null to parameter #1 ($format) of type string is deprecated in Command line code on line 11\n",
            "string(0) \"\"\n",
            "TypeError: date_format(): Argument #2 ($format) must be of type string, array given\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn datetime_class_format_constants_alias_global_date_constants() {
    let execution = run_source(
        r#"<?php
var_dump(
    DATE_ATOM === DateTime::ATOM,
    DATE_COOKIE === DateTime::COOKIE,
    DATE_ISO8601 === DateTime::ISO8601,
    DATE_ISO8601_EXPANDED === DateTime::ISO8601_EXPANDED,
    DATE_RFC822 === DateTime::RFC822,
    DATE_RFC850 === DateTime::RFC850,
    DATE_RFC1036 === DateTime::RFC1036,
    DATE_RFC1123 === DateTime::RFC1123,
    DATE_RFC7231 === DateTime::RFC7231,
    DATE_RFC2822 === DateTime::RFC2822,
    DATE_RFC3339 === DateTime::RFC3339,
    DATE_RFC3339_EXTENDED === DateTime::RFC3339_EXTENDED,
    DATE_RSS === DateTime::RSS,
    DATE_W3C === DateTime::W3C
);
"#,
    )
    .unwrap();

    assert!(execution.stdout.contains(
        "Deprecated: Constant DATE_RFC7231 is deprecated since 8.5, as this format ignores the associated timezone and always uses GMT"
    ));
    assert!(execution.stdout.contains(
        "Deprecated: Constant DateTimeInterface::RFC7231 is deprecated since 8.5, as this format ignores the associated timezone and always uses GMT"
    ));
    assert_eq!(execution.stdout.matches("bool(true)").count(), 14);
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn datetime_fixed_offset_abbreviations_match_bounded_timelib_rows() {
    let execution = run_source(
        r#"<?php
date_default_timezone_set("GMT");
$date = date_create("2005-07-18 22:10:00 +0400");
echo $date->format("D, d M Y H:i:s T"), "\n";
$date = date_create("@1121710200 +0912");
echo $date->format("D, d M Y H:i:s T"), "\n";
echo date_format(date_create("2005-07-18 22:10:00 GMT"), "T"), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Mon, 18 Jul 2005 22:10:00 GMT+0400\nMon, 18 Jul 2005 18:10:00 GMT+0000\nGMT\n"
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
fn timezone_open_warns_and_returns_false_for_invalid_scalar_ids() {
    let execution = run_source(
        r#"<?php
$timezones = [ "+02:30", "Europe/Kyiv", 2.5, "99:60", "Europe/Lviv" ];

foreach ($timezones as $timezone) {
    $d = timezone_open($timezone);
    if ($d) {
        echo "In: {$timezone}; Out: ", $d->getName(), "\n";
    }
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "In: +02:30; Out: +02:30\n",
            "In: Europe/Kyiv; Out: Europe/Kyiv\n",
            "\nWarning: timezone_open(): Unknown or bad timezone (2.5) in Command line code on line 5\n",
            "\nWarning: timezone_open(): Unknown or bad timezone (99:60) in Command line code on line 5\n",
            "\nWarning: timezone_open(): Unknown or bad timezone (Europe/Lviv) in Command line code on line 5\n"
        )
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

#[test]
fn solar_date_builtins_match_bounded_timelib_rows() {
    let execution = run_source(
        r#"<?php
error_reporting(E_ALL & ~E_DEPRECATED);
date_default_timezone_set("UTC");
$sun = date_sun_info(strtotime("2006-12-12"), 31.7667, 35.2333);
echo $sun["sunrise"], "|", $sun["sunset"], "|", $sun["transit"], "\n";
echo date("H:i:s", date_sunrise(mktime(8, 8, 8, 8, 11, 2008), SUNFUNCS_RET_TIMESTAMP, -14.24, -170.72, 90, -11)), "\n";
echo date_sunrise(mktime(8, 8, 8, 8, 12, 2008), SUNFUNCS_RET_STRING, 41.85, -87.65, 90, -5), "\n";
echo date_sunset(mktime(8, 8, 8, 8, 12, 2008), SUNFUNCS_RET_STRING, 55.75, 37.58, 90, 4), "\n";
date_default_timezone_set("America/Sao_Paulo");
$polar = date_sun_info(strtotime("2015-01-12 00:00:00 UTC"), 89.00, 1.00);
var_dump($polar["sunrise"]);
echo date("H:i:s", $polar["transit"]), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "1165897761|1165934160|1165915961\n17:42:19\n05:59\n21:08\nbool(false)\n10:03:48\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn datetimezone_and_datetime_residuals_match_bounded_rows() {
    let execution = run_source(
        r#"<?php
date_default_timezone_set("Europe/London");
$tz = timezone_open("Europe/Oslo");
echo timezone_name_get($tz), "\n";
var_dump(new DateTimeZone("GMT"));
$date = date_create("2005-07-14 22:30:41 GMT");
echo date_format($date, "D M j G:i:s T Y"), "\n";
$epoch = date_create("2009-02-13 23:31:30 GMT");
echo date_format($epoch, "B => (U) => T Y-M-d H:i:s"), "\n";
$ny = new DateTimeZone("America/New_York");
$fixed = date_create("2008-12-25 14:25:41 GMT");
echo $ny->getName(), "|", $ny->getOffset($fixed), "\n";
$tran = timezone_transitions_get(timezone_open("Europe/London"), -306972000, -37241999);
echo count($tran), "|", $tran[6]["ts"], "|", $tran[6]["abbr"], "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "Europe/Oslo\n",
            "object(DateTimeZone)#2 (2) {\n",
            "  [\"timezone_type\"]=>\n",
            "  int(2)\n",
            "  [\"timezone\"]=>\n",
            "  string(3) \"GMT\"\n",
            "}\n",
            "Thu Jul 14 22:30:41 GMT 2005\n",
            "021 => (1234567890) => GMT 2009-Feb-13 23:31:30\n",
            "America/New_York|-18000\n",
            "18|-213228000|BST\n"
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn datetimezone_serialize_roundtrips_bounded_metadata() {
    let execution = run_source(
        r#"<?php
date_default_timezone_set("Europe/London");
$fixed = date_create("2012-01-01 10:00 +1:00")->getTimezone();
foreach (array($fixed, new DateTimeZone("EST"), new DateTimeZone("America/New_York")) as $tz) {
    $serialized = serialize($tz);
    $copy = unserialize($serialized);
    echo $tz->getName(), "|", $serialized, "|", $copy->getName(), "\n";
}
$bad = 'O:12:"DateTimeZone":2:{s:13:"timezone_type";i:3;s:8:"timezone";s:17:"Ame' . "\0" . 'rica/New_York";}';
try {
    unserialize($bad);
} catch (Throwable $e) {
    echo get_class($e), ": ", $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "+01:00|O:12:\"DateTimeZone\":2:{s:13:\"timezone_type\";i:1;s:8:\"timezone\";s:6:\"+01:00\";}|+01:00\n",
            "EST|O:12:\"DateTimeZone\":2:{s:13:\"timezone_type\";i:2;s:8:\"timezone\";s:3:\"EST\";}|EST\n",
            "America/New_York|O:12:\"DateTimeZone\":2:{s:13:\"timezone_type\";i:3;s:8:\"timezone\";s:16:\"America/New_York\";}|America/New_York\n",
            "Error: Invalid serialization data for DateTimeZone object\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn datetime_mutable_timezone_metadata_matches_basic_rows() {
    let execution = run_source(
        r#"<?php
date_default_timezone_set("Europe/London");
$winter = new DateTime("2008-12-25 14:25:41");
$summer = new DateTime("2008-07-02 14:25:41");
echo "offsets=", $winter->getOffset() / 3600, "|", $summer->getOffset() / 3600, "\n";
$object = new DateTime("2009-01-30 17:57:32");
echo $object->getTimeZone()->getName(), "\n";
date_default_timezone_set("America/New_York");
$object = new DateTime("2009-01-30 17:57:32");
echo $object->getTimeZone()->getName(), "\n";
$returned = $object->setTimeZone(new DateTimeZone("America/Los_Angeles"));
echo date_timezone_get($object)->getName(), "|";
echo ($returned === $object ? "same" : "different"), "\n";
$returned = date_timezone_set($object, new DateTimeZone("Europe/London"));
echo date_timezone_get($object)->getName(), "|";
echo $object->format("Y-m-d H:i:s T"), "|";
echo ($returned === $object ? "same" : "different"), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "offsets=0|1\nEurope/London\nAmerica/New_York\nAmerica/Los_Angeles|same\nEurope/London|2009-01-30 22:57:32 GMT|same\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn datetime_mutable_date_time_setters_normalize_bounded_parts() {
    let execution = run_source(
        r#"<?php
date_default_timezone_set("Europe/London");
$datetime = new DateTime("2009-01-30 19:34:10");
$returned = $datetime->setDate(2008, 2, 1);
echo $datetime->format(DATE_RFC2822), "|", ($returned === $datetime ? "same" : "different"), "\n";
$returned = $datetime->setTime(24, 10);
echo $datetime->format(DATE_RFC2822), "|", ($returned === $datetime ? "same" : "different"), "\n";
$returned = $datetime->setISODate(2009, 30, 3);
echo $datetime->format("Y-m-d D H:i:s T"), "|", ($returned === $datetime ? "same" : "different"), "\n";
$call = "date_time_set";
$returned = $call($datetime, 47, 35, 47);
echo $datetime->format(DATE_RFC2822), "|", ($returned === $datetime ? "same" : "different"), "\n";
$returned = date_date_set($datetime, 2010, 13, 32);
echo $datetime->format("Y-m-d H:i:s T"), "|", ($returned === $datetime ? "same" : "different"), "\n";
$returned = date_isodate_set($datetime, 2008, 40);
echo $datetime->format("Y-m-d D H:i:s T"), "|", ($returned === $datetime ? "same" : "different"), "\n";
echo function_exists("date_date_set") ? "datefn" : "missing";
echo "|", function_exists("date_time_set") ? "timefn" : "missing";
echo "|", function_exists("date_isodate_set") ? "isofn" : "missing";
echo "|", is_callable([$datetime, "setDate"]) ? "setdate" : "missing";
echo "|", is_callable([$datetime, "setTime"]) ? "settime" : "missing";
echo "|", is_callable([$datetime, "setISODate"]) ? "setiso" : "missing";
echo "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "Fri, 01 Feb 2008 19:34:10 +0000|same\n",
            "Sat, 02 Feb 2008 00:10:00 +0000|same\n",
            "2009-07-22 Wed 00:10:00 BST|same\n",
            "Thu, 23 Jul 2009 23:35:47 +0100|same\n",
            "2011-02-01 23:35:47 GMT|same\n",
            "2008-09-29 Mon 23:35:47 BST|same\n",
            "datefn|timefn|isofn|setdate|settime|setiso\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn datetime_mutable_setters_reject_parts_outside_bounded_timestamp_range() {
    let error = run_source(
        r#"<?php
$datetime = new DateTime("2009-01-30 19:34:10");
$datetime->setDate(PHP_INT_MAX, PHP_INT_MAX, 1);
"#,
    )
    .unwrap_err();

    assert!(error.to_string().contains(
        "unsupported call DateTime::setDate(): date/time parts overflow the current bounded timestamp subset"
    ));
}
