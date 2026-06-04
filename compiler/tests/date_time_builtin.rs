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
fn strtotime_uses_base_timestamp_and_bounded_textual_forms() {
    let execution = run_source(
        r#"<?php
date_default_timezone_set("UTC");
echo strtotime("19:30 Dec 17 2005"), "|", strtotime("Dec 17 19:30 2005"), "\n";
echo gmdate("Y-m-d H:i:s", strtotime("Oct 2001")), "|";
echo gmdate("Y-m-d H:i:s", strtotime("2001 Oct")), "\n";
echo date(DATE_ISO8601, strtotime("17:00 2004-01-01")), "\n";
echo gmdate("Y-m-d H:i:s", strtotime("20 VI. 2005")), "\n";
echo strtotime("20050518t091234Z"), "\n";
echo date(DATE_ISO8601, strtotime("2005-12-22 noon")), "|";
echo date(DATE_ISO8601, strtotime("2005-12-22 midnight")), "\n";
echo date(DATE_ISO8601, strtotime("Sat 26th Nov 2005 18:18")), "|";
echo date(DATE_ISO8601, strtotime("26th Nov", 1134340285)), "|";
echo date(DATE_ISO8601, strtotime("Dec. 4th, 2005")), "\n";
$base = 1204200000;
echo date(DATE_ISO8601, strtotime("+80412 seconds", $base)), "|";
echo date(DATE_ISO8601, strtotime("-115 months", $base)), "\n";
echo date("D Y-m-d", strtotime("saturday this week", strtotime("Sun 2017-01-01"))), "|";
echo date("D Y-m-d", strtotime("saturday this week", strtotime("Mon 2017-01-02"))), "\n";
date_default_timezone_set("Europe/Amsterdam");
echo gmdate("d-m-Y H:i:s", strtotime("+30 minutes", 1100535573)), "\n";
var_dump(strtotime("-2\thours", 1140973388));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "1134847800|1134847800\n",
            "2001-10-01 00:00:00|2001-10-01 00:00:00\n",
            "2004-01-01T17:00:00+0000\n",
            "2005-06-20 00:00:00\n",
            "1116407554\n",
            "2005-12-22T12:00:00+0000|2005-12-22T00:00:00+0000\n",
            "2005-11-26T18:18:00+0000|2005-11-26T00:00:00+0000|2005-12-04T00:00:00+0000\n",
            "2008-02-29T10:20:12+0000|1998-07-28T12:00:00+0000\n",
            "Sat 2016-12-31|Sat 2017-01-07\n",
            "15-11-2004 16:49:33\n",
            "int(1140966188)\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn strtotime_accepts_bounded_absolute_edge_formats() {
    let execution = run_source(
        r#"<?php
date_default_timezone_set("US/Eastern");
echo date("r", strtotime("Sep 04 2001 16:39:45")), "\n";
date_default_timezone_set("America/New_York");
echo date("Y-m-d H:i:s T", strtotime("2003-10-28 10:20:30-0800")), "\n";
date_default_timezone_set("UTC");
echo gmdate("Y-m-d H:i:s", strtotime("20050620091407 GMT")), "\n";
echo gmdate("Y-m-d H:i:s", strtotime("2002-06-25 14:18:48.543728-04")), "|";
echo gmdate("Y-m-d H:i:s", strtotime("2002-06-25 14:18:48.543728+04")), "\n";
echo date("Y-m-d H:i:s", strtotime("2003-11-19T12:30:42Z")), "\n";
echo gmdate("m/d/y Hi", strtotime("04/04/04 0045")), "\n";
echo gmdate("Y-m-d H:i:s", strtotime("2004W30")), "\n";
echo date("m/d/Y", strtotime("11Oct2005")), "|", date("m/d/Y", strtotime("Oct11")), "\n";
$base = strtotime("2005-01-01 01:01:01");
echo date(DATE_ISO8601, strtotime("+5days", $base)), "|";
echo date(DATE_ISO8601, strtotime("+1month", $base)), "\n";
echo strtotime("2005/8/12"), "|", date(DATE_ISO8601, strtotime("2005/1/2")), "\n";
echo date(DATE_ISO8601, strtotime("2005-12-22 1p.m.")), "\n";
var_dump(strtotime("NOW", 1234567890) === 1234567890);
echo date(DATE_ISO8601, strtotime("2006-1-6T0:0:0-8:0")), "\n";
date_default_timezone_set("Europe/Oslo");
echo date(DATE_ATOM, strtotime("2006-01-31T19:23:56Z")), "\n";
date_default_timezone_set("UTC");
echo date("r", strtotime("May 18th 5:05pm", 1168156376)), "\n";
echo date(DATE_ISO8601, strtotime("July 1, 2000 00:00:00 UTC")), "|";
echo date(DATE_ISO8601, strtotime("July 1, 2000 00:00:00 GMT")), "\n";
$parsed = date_parse(" a ");
echo count($parsed), "|", $parsed["zone_type"], "|", $parsed["zone"], "|", $parsed["tz_abbr"], "\n";
var_dump(date_parse(" \n ")["error_count"]);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "Tue, 04 Sep 2001 16:39:45 -0400\n",
            "2003-10-28 13:20:30 EST\n",
            "2005-06-20 09:14:07\n",
            "2002-06-25 18:18:48|2002-06-25 10:18:48\n",
            "2003-11-19 12:30:42\n",
            "04/04/04 0045\n",
            "2004-07-19 00:00:00\n",
            "10/11/2005|10/11/1970\n",
            "2005-01-06T01:01:01+0000|2005-02-01T01:01:01+0000\n",
            "1123804800|2005-01-02T00:00:00+0000\n",
            "2005-12-22T13:00:00+0000\n",
            "bool(true)\n",
            "2006-01-06T08:00:00+0000\n",
            "2006-01-31T20:23:56+01:00\n",
            "Fri, 18 May 2007 17:05:00 +0000\n",
            "2000-07-01T00:00:00+0000|2000-07-01T00:00:00+0000\n",
            "16|2|3600|A\n",
            "int(0)\n",
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
fn datetime_immutable_copy_apis_and_setters_use_bounded_state() {
    let execution = run_source(
        r#"<?php
date_default_timezone_set("Europe/London");
class MyDateTime extends DateTime {}
class MyDateTimeImmutable extends DateTimeImmutable {}

$mutable = date_create("2014-03-02 16:24:08");
$immutable = date_create_immutable("2014-03-02 16:24:08");
echo get_class($immutable), "|", $immutable->format("Y-m-d H:i:s T e"), "\n";

$fromMutable = DateTimeImmutable::createFromMutable($mutable);
echo get_class($fromMutable), "|", $fromMutable->format("Y-m-d H:i:s T e"), "\n";
$subImmutable = MyDateTimeImmutable::createFromMutable($mutable);
echo get_class($subImmutable), "|", $subImmutable->format("Y-m-d H:i:s T e"), "\n";

$mutableCopy = DateTime::createFromImmutable($immutable);
$subMutable = MyDateTime::createFromImmutable($immutable);
$mutableCopy->modify("+1 hour");
echo get_class($mutableCopy), "|", $mutableCopy->format("Y-m-d H:i:s"), "|", $immutable->format("Y-m-d H:i:s"), "\n";
echo get_class($subMutable), "|", $subMutable->format("Y-m-d H:i:s T e"), "\n";
$interfaceMutable = DateTime::createFromInterface($immutable);
$interfaceImmutable = DateTimeImmutable::createFromInterface($mutable);
echo get_class($interfaceMutable), "|", $interfaceMutable->format("Y-m-d H:i:s T e"), "|", get_class($interfaceImmutable), "|", $interfaceImmutable->format("Y-m-d H:i:s T e"), "\n";

$changed = $immutable->setTime(25, 2)->setDate(2015, 13, 32)->setTimezone(new DateTimeZone("UTC"));
echo $immutable->format("Y-m-d H:i:s e"), "|", $changed->format("Y-m-d H:i:s e"), "|", ($changed === $immutable ? "same" : "new"), "\n";

$state = var_export($immutable, true);
eval("\$fromState = {$state};");
echo get_class($fromState), "|", $fromState->format("Y-m-d H:i:s T e"), "\n";
echo function_exists("date_create_immutable") ? "fn" : "missing";
echo "|", is_callable(array("DateTimeImmutable", "createFromMutable")) ? "static" : "missing";
echo "|", (DATE_ATOM === DateTimeImmutable::ATOM ? "const" : "bad"), "\n";

try {
    DateTimeImmutable::createFromMutable($immutable);
} catch (Throwable $e) {
    echo get_class($e), ": ", $e->getMessage(), "\n";
}
try {
    DateTime::createFromImmutable($mutable);
} catch (Throwable $e) {
    echo get_class($e), ": ", $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "DateTimeImmutable|2014-03-02 16:24:08 GMT Europe/London\n",
            "DateTimeImmutable|2014-03-02 16:24:08 GMT Europe/London\n",
            "MyDateTimeImmutable|2014-03-02 16:24:08 GMT Europe/London\n",
            "DateTime|2014-03-02 17:24:08|2014-03-02 16:24:08\n",
            "MyDateTime|2014-03-02 16:24:08 GMT Europe/London\n",
            "DateTime|2014-03-02 16:24:08 GMT Europe/London|DateTimeImmutable|2014-03-02 16:24:08 GMT Europe/London\n",
            "2014-03-02 16:24:08 Europe/London|2016-02-01 01:02:00 UTC|new\n",
            "DateTimeImmutable|2014-03-02 16:24:08 GMT Europe/London\n",
            "fn|static|const\n",
            "TypeError: DateTimeImmutable::createFromMutable(): Argument #1 ($object) must be of type DateTime, DateTimeImmutable given\n",
            "TypeError: DateTime::createFromImmutable(): Argument #1 ($object) must be of type DateTimeImmutable, DateTime given\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn date_objects_clone_dynamic_properties_and_compare_bounded_metadata() {
    let execution = run_source(
        r#"<?php
date_default_timezone_set("Europe/London");
class DateTimeExt extends DateTime { public $label = "ext"; }
class MyDateTime extends DateTime { function __construct() {} }
class MyDateTimeZone extends DateTimeZone { function __construct() {} }

$date = new DateTime("2009-02-03 12:34:41 GMT");
$date->property1 = 99;
$dateClone = clone $date;
echo "date-clone=", $dateClone->property1, "|", $dateClone->format("Y-m-d H:i:s T"), "\n";
$dateClone->property1 = 100;
echo "date-separate=", $date->property1, "|", $dateClone->property1, "\n";
$subclass = new DateTimeExt("2009-02-03 12:34:41 GMT");
echo "date-subclass=", (clone $subclass)->label, "|", (clone $subclass)->format("Y-m-d H:i:s T"), "\n";

$zone = new DateTimeZone("Europe/London");
$zone->property1 = "tz";
$zoneClone = clone $zone;
echo "zone-clone=", $zoneClone->property1, "|", $zoneClone->getName(), "\n";

$sameMutable = new DateTime("2023-01-16 17:09:08 UTC");
$sameImmutable = new DateTimeImmutable("2023-01-16 17:09:08 +0000");
$later = new DateTime("2023-01-16 17:09:09 UTC");
var_dump($sameMutable == $sameImmutable, $sameMutable < $sameImmutable, $sameMutable < $later, $later > $sameImmutable);

$fixed = new DateTimeZone("+0200");
var_dump($fixed == new DateTimeZone("+02:00"), $fixed < new DateTimeZone("-0200"));
try { var_dump(new DateTimeZone("Europe/Berlin") == new DateTimeZone("CET")); }
catch (DateException $e) { echo $e::class, ": ", $e->getMessage(), "\n"; }
try { var_dump(new MyDateTimeZone() == new MyDateTimeZone()); }
catch (DateObjectError $e) { echo $e::class, ": ", $e->getMessage(), "\n"; }
try { var_dump($sameMutable < new MyDateTime()); }
catch (DateObjectError $e) { echo $e::class, ": ", $e->getMessage(), "\n"; }
"#,
    )
    .unwrap();

    assert!(execution
        .stdout
        .contains("Deprecated: Creation of dynamic property DateTime::$property1 is deprecated"));
    assert!(execution.stdout.contains(
        "Deprecated: Creation of dynamic property DateTimeZone::$property1 is deprecated"
    ));
    assert!(execution
        .stdout
        .contains("date-clone=99|2009-02-03 12:34:41 GMT\n"));
    assert!(execution.stdout.contains("date-separate=99|100\n"));
    assert!(execution
        .stdout
        .contains("date-subclass=ext|2009-02-03 12:34:41 GMT\n"));
    assert!(execution.stdout.contains("zone-clone=tz|Europe/London\n"));
    assert!(execution
        .stdout
        .contains("bool(true)\nbool(false)\nbool(true)\nbool(true)\n"));
    assert!(execution.stdout.contains("bool(true)\nbool(false)\n"));
    assert!(execution
        .stdout
        .contains("DateException: Cannot compare two different kinds of DateTimeZone objects\n"));
    assert!(execution
        .stdout
        .contains("DateObjectError: Trying to compare uninitialized DateTimeZone objects\n"));
    assert!(execution.stdout.contains(
        "DateObjectError: Trying to compare an incomplete DateTime or DateTimeImmutable object\n"
    ));
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
fn datetime_mutation_cluster_handles_calendar_relative_and_subseconds() {
    let execution = run_source(
        r#"<?php
date_default_timezone_set("Asia/Calcutta");
$date = new DateTime("18-01-2009 00:00:00");
$date->setISODate(2009, 6, 1);
echo $date->format("Y-m-d H:i:s"), "|";
$date->setTime(8, 0);
echo $date->format("Y-m-d H:i:s"), "\n";

date_default_timezone_set("UTC");
$rel = date_parse("last day of next month")["relative"];
echo $rel["month"], "|", $rel["day"], "|", isset($rel["last_day_of_month"]) ? "last" : "missing", "\n";
$relative = new DateTime("2010-03-06 15:21 UTC");
$relative->modify("first day next month");
echo $relative->format(DateTime::ISO8601), "\n";
$relative = new DateTime("2010-03-06 15:21 UTC");
$relative->modify("last day of next month");
echo $relative->format(DateTime::ISO8601), "\n";

$week = new DateTime("2010-07-27 09:46:49", new DateTimeZone("Europe/London"));
$week->modify("this week +6 days");
echo $week->format("Y-m-d H:i:s"), "|", $week->getTimestamp(), "\n";
$friday = new DateTime("2017-01-08");
$friday->modify("Friday this week");
echo $friday->format("Y-m-d"), "\n";

$subsecond = new DateTimeImmutable("2016-10-07 13:25:50");
echo $subsecond->modify("+8 msec -2 µsec")->format("Y-m-d H:i:s.u"), "\n";
$noop = new DateTime("2015-01-01 00:00:00+00:00");
echo $noop->modify("+1 s")->format("c"), "\n";

date_default_timezone_set("Pacific/Kwajalein");
$kwajalein = date_create("Fri Aug 20 1993 23:59:59");
echo date_format($kwajalein, "D, d M Y H:i:s T"), "\n";
$kwajalein->modify("+1 second");
echo date_format($kwajalein, "D, d M Y H:i:s T"), "\n";

date_default_timezone_set("Europe/Amsterdam");
$spring = date_create("Sun Mar 27 01:59:59 2005");
echo date_format($spring, "D, d M Y H:i:s T"), "\n";
$spring->modify("+1 second");
echo date_format($spring, "D, d M Y H:i:s T"), "\n";
$fall = date_create("Sun Oct 30 01:59:59 2005");
echo date_format($fall, "D, d M Y H:i:s T"), "\n";
$fall->modify("+ 1 hour 1 second");
echo date_format($fall, "D, d M Y H:i:s T"), "\n";

date_default_timezone_set("UTC");
$strings = new DateTime("2011-12-25 00:00:00");
$strings->modify("first day of next month");
$strings->setDate("2012", "1", "29");
echo $strings->format("Y-m-d H:i:s.u"), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "2009-02-02 00:00:00|2009-02-02 08:00:00\n",
            "1|0|last\n",
            "2010-04-07T15:21:00+0000\n",
            "2010-04-30T15:21:00+0000\n",
            "2010-08-01 09:46:49|1280652409\n",
            "2017-01-06\n",
            "2016-10-07 13:25:50.007998\n",
            "2015-01-01T00:00:00+00:00\n",
            "Fri, 20 Aug 1993 23:59:59 -12\n",
            "Sun, 22 Aug 1993 00:00:00 +12\n",
            "Sun, 27 Mar 2005 01:59:59 CET\n",
            "Sun, 27 Mar 2005 03:00:00 CEST\n",
            "Sun, 30 Oct 2005 01:59:59 CEST\n",
            "Sun, 30 Oct 2005 03:00:00 CET\n",
            "2012-01-29 00:00:00.000000\n",
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
fn datetime_subclass_modify_override_precedes_core_dispatch() {
    let execution = run_source(
        r#"<?php
class MyDateTime extends DateTime
{
    #[ReturnTypeWillChange]
    public function modify(string $modifier) {
        return false;
    }
}

$date = new MyDateTime("2021-01-01 00:00:00");
var_dump($date->modify("+1 sec"));
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "bool(false)\n");
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn malformed_datetime_and_dateinterval_strings_throw_or_warn() {
    let execution = run_source(
        r#"<?php
date_default_timezone_set("UTC");

$mutable = new DateTime();
var_dump(date_modify($mutable, ""));
try {
    $mutable->modify("");
} catch (DateMalformedStringException $e) {
    echo $e::class, ": ", $e->getMessage(), "\n";
}

$immutable = new DateTimeImmutable();
try {
    $immutable->modify("");
} catch (DateMalformedStringException $e) {
    echo $e::class, ": ", $e->getMessage(), "\n";
}

var_dump(date_interval_create_from_date_string("foobar"));
var_dump(date_interval_create_from_date_string(null));

foreach (["next weekday 15:30", "+5 hours noon", "-8 days March 23", "+72 seconds UTC"] as $format) {
    try {
        DateInterval::createFromDateString($format);
    } catch (DateMalformedIntervalStringException $e) {
        echo $e::class, ": ", $e->getMessage(), "\n";
    }
}

foreach (["next weekday 15:30", "+5 hours noon", "-8 days March 23", "+72 seconds UTC"] as $format) {
    var_dump(date_interval_create_from_date_string($format));
}
"#,
    )
    .unwrap();

    assert!(execution.stdout.contains(
        "Warning: date_modify(): Failed to parse time string () at position 0 ( ): Empty string"
    ));
    assert!(execution.stdout.contains(
        "DateMalformedStringException: DateTime::modify(): Failed to parse time string () at position 0 ( ): Empty string"
    ));
    assert!(execution.stdout.contains(
        "DateMalformedStringException: DateTimeImmutable::modify(): Failed to parse time string () at position 0 ( ): Empty string"
    ));
    assert!(execution.stdout.contains(
        "Warning: date_interval_create_from_date_string(): Unknown or bad format (foobar) at position 0 (f): The timezone could not be found in the database"
    ));
    assert!(execution.stdout.contains(
        "Warning: date_interval_create_from_date_string(): Unknown or bad format () at position 0 ( ): Empty string"
    ));
    assert!(execution.stdout.contains(
        "DateMalformedIntervalStringException: String 'next weekday 15:30' contains non-relative elements"
    ));
    assert!(execution.stdout.contains(
        "Warning: date_interval_create_from_date_string(): String '+72 seconds UTC' contains non-relative elements"
    ));
    assert_eq!(execution.stdout.matches("bool(false)").count(), 7);
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
    DATE_W3C === DateTime::W3C,
    DATE_ATOM === DateTimeImmutable::ATOM,
    DATE_COOKIE === DateTimeImmutable::COOKIE,
    DATE_ISO8601 === DateTimeImmutable::ISO8601,
    DATE_ISO8601_EXPANDED === DateTimeImmutable::ISO8601_EXPANDED,
    DATE_RFC822 === DateTimeImmutable::RFC822,
    DATE_RFC850 === DateTimeImmutable::RFC850,
    DATE_RFC1036 === DateTimeImmutable::RFC1036,
    DATE_RFC1123 === DateTimeImmutable::RFC1123,
    DATE_RFC7231 === DateTimeImmutable::RFC7231,
    DATE_RFC2822 === DateTimeImmutable::RFC2822,
    DATE_RFC3339 === DateTimeImmutable::RFC3339,
    DATE_RFC3339_EXTENDED === DateTimeImmutable::RFC3339_EXTENDED,
    DATE_RSS === DateTimeImmutable::RSS,
    DATE_W3C === DateTimeImmutable::W3C
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
    assert_eq!(execution.stdout.matches("bool(true)").count(), 28);
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn timezone_metadata_inventory_interface_constants_and_offset_diagnostics() {
    let execution = run_source(
        r#"<?php
var_dump(interface_exists("DateTimeInterface"));
var_dump(defined("DateTimeInterface::ATOM"));
var_dump(
    DATE_ATOM === DateTimeInterface::ATOM,
    DATE_COOKIE === DateTimeInterface::COOKIE,
    DATE_ISO8601 === DateTimeInterface::ISO8601,
    DATE_ISO8601_EXPANDED === DateTimeInterface::ISO8601_EXPANDED,
    DATE_RFC822 === DateTimeInterface::RFC822,
    DATE_RFC850 === DateTimeInterface::RFC850,
    DATE_RFC1036 === DateTimeInterface::RFC1036,
    DATE_RFC1123 === DateTimeInterface::RFC1123,
    DATE_RFC7231 === DateTimeInterface::RFC7231,
    DATE_RFC2822 === DateTimeInterface::RFC2822,
    DATE_RFC3339 === DateTimeInterface::RFC3339,
    DATE_RFC3339_EXTENDED === DateTimeInterface::RFC3339_EXTENDED,
    DATE_RSS === DateTimeInterface::RSS,
    DATE_W3C === DateTimeInterface::W3C
);

$abbreviations = timezone_abbreviations_list();
echo count($abbreviations), "|", count(DateTimeZone::listAbbreviations()), "\n";
foreach ($abbreviations["acst"] as $row) {
    echo ($row["dst"] ? "1" : "0"), "|", $row["offset"], "|", $row["timezone_id"], "\n";
}

$oslo = timezone_location_get(new DateTimeZone("Europe/Oslo"));
echo $oslo["country_code"], "|", $oslo["latitude"], "|", $oslo["longitude"], "|", $oslo["comments"], "\n";
$printed = array();
foreach (DateTimeZone::listAbbreviations() as $value) {
    if (NULL != $value[0]["timezone_id"]) {
        $location = (new DateTimeZone($value[0]["timezone_id"]))->getLocation();
        if (false === $location) {
            continue;
        }
        if (!isset($printed[$location["country_code"]]) && in_array($location["country_code"], array("AU", "CA", "ET", "AF", "US", "KZ", "AM"))) {
            $printed[$location["country_code"]] = true;
            echo $location["country_code"], "|";
        }
    }
}
echo "\n";

$tz = new DateTimeZone("Europe/London");
$dt = new DateTimeImmutable("2014-09-20", $tz);
echo $tz->getOffset($dt), "|", timezone_offset_get($tz, $dt), "\n";
try { $tz->getOffset(1); } catch (TypeError $e) { echo $e::class, ": ", $e->getMessage(), "\n"; }
try { timezone_offset_get(new stdClass(), $dt); } catch (Error $e) { echo $e::class, ": ", $e->getMessage(), "\n"; }
try { timezone_offset_get($tz, null); } catch (Error $e) { echo $e::class, ": ", $e->getMessage(), "\n"; }
"#,
    )
    .unwrap();

    assert!(execution.stdout.contains(
        "Deprecated: Constant DATE_RFC7231 is deprecated since 8.5, as this format ignores the associated timezone and always uses GMT"
    ));
    assert!(execution.stdout.contains(
        "Deprecated: Constant DateTimeInterface::RFC7231 is deprecated since 8.5, as this format ignores the associated timezone and always uses GMT"
    ));
    assert_eq!(execution.stdout.matches("bool(true)").count(), 16);
    assert!(execution.stdout.contains("144|144\n"));
    assert!(execution
        .stdout
        .contains("0|34200|Australia/Adelaide\n0|34200|Australia/Broken_Hill\n"));
    assert!(execution
        .stdout
        .contains("0|34200|Australia/Yancowinna\nNO|59.91666|10.75|\nAU|CA|US|ET|\n"));
    assert!(execution.stdout.contains("3600|3600\n"));
    assert!(execution.stdout.contains(
        "TypeError: DateTimeZone::getOffset(): Argument #1 ($datetime) must be of type DateTimeInterface, int given\n"
    ));
    assert!(execution.stdout.contains(
        "TypeError: timezone_offset_get(): Argument #1 ($object) must be of type DateTimeZone, stdClass given\n"
    ));
    assert!(execution.stdout.contains(
        "TypeError: timezone_offset_get(): Argument #2 ($datetime) must be of type DateTimeInterface, null given\n"
    ));
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
fn date_object_serialization_state_helpers_validate_bounded_metadata() {
    let execution = run_source(
        r#"<?php
date_default_timezone_set("Europe/London");
$date = new DateTime("2005-07-14 22:30:41");
$serialized = serialize($date);
echo $serialized, "\n";
$copy = unserialize($serialized);
echo $copy->format("F j, Y, g:i a"), "\n";
try {
    DateTime::__set_state(array(
        "date" => 2023.113,
        "timezone_type" => 3,
        "timezone" => "Europe/Kyiv",
    ));
} catch (Throwable $e) {
    echo get_class($e), ": ", $e->getMessage(), "\n";
}

$tz = new DateTimeZone("CEST");
$state = $tz->__serialize();
echo $state["timezone_type"], "|", $state["timezone"], "\n";
$tz->__unserialize(array("timezone_type" => 1, "timezone" => "+0130"));
echo $tz->getName(), "\n";
$copy = DateTimeZone::__set_state(array("timezone_type" => 3, "timezone" => "Europe/Kyiv"));
echo $copy->getName(), "\n";
try {
    DateTimeZone::__set_state(array("timezone_type" => 4, "timezone" => "Europe/Kyiv"));
} catch (Throwable $e) {
    echo get_class($e), ": ", $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "O:8:\"DateTime\":3:{s:4:\"date\";s:26:\"2005-07-14 22:30:41.000000\";s:13:\"timezone_type\";i:3;s:8:\"timezone\";s:13:\"Europe/London\";}\n",
            "July 14, 2005, 10:30 pm\n",
            "Error: Invalid serialization data for DateTime object\n",
            "2|CEST\n",
            "+01:30\n",
            "Europe/Kyiv\n",
            "Error: Invalid serialization data for DateTimeZone object\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn datetime_immutable_serialization_state_and_uninitialized_copy_errors() {
    let execution = run_source(
        r#"<?php
date_default_timezone_set("Europe/London");
class SerialDateTimeImmutable extends DateTimeImmutable {
    public function __construct(public ?bool $myProperty = null) {
        parent::__construct("2022-04-14 11:27:42");
    }
}
class MyDateTime extends DateTime {
    public function __construct() {}
}
class MyDateTimeImmutable extends DateTimeImmutable {
    public function __construct() {}
}

$date = new DateTimeImmutable("2022-04-14 11:27:42");
$serialized = serialize($date);
echo $serialized, "\n";
$copy = unserialize($serialized);
echo get_class($copy), "|", $copy->format("F j, Y, g:i a"), "\n";
$state = $date->__serialize();
echo $state["date"], "|", $state["timezone_type"], "|", $state["timezone"], "\n";
$date->__unserialize(array(
    "date" => "2006-01-02 03:04:05.541106",
    "timezone_type" => 1,
    "timezone" => "+0130",
));
echo $date->date, "|", $date->timezone_type, "|", $date->timezone, "\n";

$child = new SerialDateTimeImmutable(true);
$roundTrip = unserialize(serialize($child));
var_dump($roundTrip->myProperty);

try {
    DateTimeImmutable::__set_state(array(
        "date" => 2023.113,
        "timezone_type" => 3,
        "timezone" => "Europe/Kyiv",
    ));
} catch (Throwable $e) {
    echo get_class($e), ": ", $e->getMessage(), "\n";
}
try {
    DateTimeImmutable::createFromMutable(new MyDateTime());
} catch (DateObjectError $e) {
    echo get_class($e), ": ", $e->getMessage(), "\n";
}
try {
    DateTimeImmutable::createFromInterface(new MyDateTimeImmutable());
} catch (DateObjectError $e) {
    echo get_class($e), ": ", $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "O:17:\"DateTimeImmutable\":3:{s:4:\"date\";s:26:\"2022-04-14 11:27:42.000000\";s:13:\"timezone_type\";i:3;s:8:\"timezone\";s:13:\"Europe/London\";}\n",
            "DateTimeImmutable|April 14, 2022, 11:27 am\n",
            "2022-04-14 11:27:42.000000|3|Europe/London\n",
            "2006-01-02 03:04:05.541106|1|+01:30\n",
            "bool(true)\n",
            "Error: Invalid serialization data for DateTimeImmutable object\n",
            "DateObjectError: Object of type MyDateTime (inheriting DateTime) has not been correctly initialized by calling parent::__construct() in its constructor\n",
            "DateObjectError: Object of type MyDateTimeImmutable (inheriting DateTimeImmutable) has not been correctly initialized by calling parent::__construct() in its constructor\n",
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
fn dateinterval_metadata_format_and_diff_cover_bounded_rows() {
    let execution = run_source(
        r#"<?php
date_default_timezone_set("UTC");
$date1 = new DateTime("2000-01-01 00:00:00");
$date2 = new DateTime("2001-03-04 04:05:06");
$interval = $date1->diff($date2);
echo $interval->format("Y=%Y M=%M D=%D H=%H I=%I S=%S R=%R a=%a"), "\n";
echo $interval->format("y=%y m=%m d=%d h=%h i=%i s=%s r=%r"), "\n";
$reverse = date_diff($date2, $date1);
echo $reverse->format("inverted R=%R r=%r %=%% x=%x"), "\n";
$absolute = $date2->diff($date1, true);
echo $absolute->format("absolute R=%R r=%r"), "\n";
$direct = new DateInterval("P2Y4DT6H8M");
echo date_interval_format($direct, "%d days"), "|";
echo $direct->days === false ? "false" : "bad", "|";
echo $direct->f = 0.5, "\n";
$weeks = date_interval_create_from_date_string("2 weeks");
$combo = DateInterval::createFromDateString("1 year + 1 day");
echo $weeks->d, "|", $combo->y, "|", $combo->d, "\n";
class Z extends DateInterval {}
$sub = new Z("P32D");
echo get_class($sub), "|", $sub->format("%d days"), "\n";
try {
    new DateInterval("");
} catch (DateMalformedIntervalStringException $e) {
    echo $e::class, ": ", $e->getMessage(), "\n";
}
try {
    new DateInterval("2007-05-11T15:30:00Z/");
} catch (DateMalformedIntervalStringException $e) {
    echo $e::class, ": ", $e->getMessage(), "\n";
}
try {
    DateInterval::createFromDateString("foobar");
} catch (DateMalformedIntervalStringException $e) {
    echo $e::class, ": ", $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "Y=01 M=02 D=03 H=04 I=05 S=06 R=+ a=428\n",
            "y=1 m=2 d=3 h=4 i=5 s=6 r=\n",
            "inverted R=- r=- %=% x=%x\n",
            "absolute R=+ r=\n",
            "4 days|false|0.5\n",
            "14|1|1\n",
            "Z|32 days\n",
            "DateMalformedIntervalStringException: Unknown or bad format ()\n",
            "DateMalformedIntervalStringException: Failed to parse interval (2007-05-11T15:30:00Z/)\n",
            "DateMalformedIntervalStringException: Unknown or bad format (foobar) at position 0 (f): The timezone could not be found in the database\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn datetime_diff_uses_directional_calendar_borrow_and_total_days() {
    let execution = run_source(
        r#"<?php
date_default_timezone_set("America/New_York");

$start = new DateTime("2010-01-31");
$end = new DateTime("2010-03-01");
echo $start->diff($end)->format("P%R%yY%mM%dDT%hH%iM%sS days=%a"), "\n";
echo $end->diff($start)->format("P%R%yY%mM%dDT%hH%iM%sS days=%a"), "\n";

$start = new DateTime("2010-01-31");
$end = new DateTime("2010-03-31");
echo $start->diff($end)->format("P%R%yY%mM%dDT%hH%iM%sS days=%a"), "\n";

$start = new DateTime("2010-02-28");
$end = new DateTime("2010-03-28");
echo $start->diff($end)->format("P%R%yY%mM%dDT%hH%iM%sS days=%a"), "\n";

$start = new DateTime("2000-02-07");
$end = new DateTime("2007-02-06");
echo $start->diff($end)->format("P%R%yY%mM%dDT%hH%iM%sS days=%a"), "\n";
echo $end->diff($start)->format("P%R%yY%mM%dDT%hH%iM%sS days=%a"), "\n";

$start = new DateTime("2010-03-13 18:38:28 EST");
$end = new DateTime("2010-03-14 03:16:55 EDT");
echo $start->diff($end)->format("days=%a"), "\n";

$start = new DateTime("2010-11-06 18:38:28 EDT");
$end = new DateTime("2010-11-07 03:16:55 EST");
echo $start->diff($end)->format("days=%a"), "\n";

$start = new DateTime("2010-11-07 00:10:20 EDT");
$end = new DateTime("2010-11-08 19:59:59 EST");
echo $start->diff($end)->format("days=%a"), "\n";

$start = new DateTime("2010-11-07 01:59:59 EDT");
$end = new DateTime("2010-11-07 01:00:00 EST");
echo $start->diff($end)->format("P%R%yY%mM%dDT%hH%iM%sS days=%a"), "\n";
echo $end->diff($start)->format("P%R%yY%mM%dDT%hH%iM%sS days=%a"), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "P+0Y0M29DT0H0M0S days=29\n",
            "P-0Y1M1DT0H0M0S days=29\n",
            "P+0Y2M0DT0H0M0S days=59\n",
            "P+0Y1M0DT0H0M0S days=28\n",
            "P+6Y11M30DT0H0M0S days=2556\n",
            "P-6Y11M28DT0H0M0S days=2556\n",
            "days=0\n",
            "days=0\n",
            "days=1\n",
            "P+0Y0M0DT0H0M1S days=0\n",
            "P-0Y0M0DT0H0M1S days=0\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn dateinterval_diff_dump_hides_absent_date_string_metadata() {
    let execution = run_source(
        r#"<?php
date_default_timezone_set("UTC");

$interval = date_diff(
    new DateTime("2010-10-04 02:18:48 EDT"),
    new DateTime("2010-11-06 18:38:28 EDT")
);
echo "--diff--\n";
var_dump($interval);

$relative = DateInterval::createFromDateString("1 day");
echo "--relative--\n";
var_dump($relative);
"#,
    )
    .unwrap();

    let (diff_output, relative_output) = execution
        .stdout
        .split_once("--relative--\n")
        .expect("relative DateInterval dump marker should be present");
    assert!(diff_output.contains("[\"from_string\"]=>\n  bool(false)"));
    assert!(!diff_output.contains("[\"date_string\"]=>"));
    assert!(relative_output.contains("[\"from_string\"]=>\n  bool(true)"));
    assert!(relative_output.contains("[\"date_string\"]=>\n  string(5) \"1 day\""));
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn datetime_interval_add_sub_mutate_or_copy_bounded_state() {
    let execution = run_source(
        r#"<?php
date_default_timezone_set("UTC");
$datetime = new DateTime("2008-01-01 12:25");
$returned = $datetime->add(new DateInterval("P3Y6M4DT12H30M5S"));
echo $datetime->format("Y-m-d H:i:s"), "|", ($returned === $datetime ? "same" : "different"), "\n";
$inverted = new DateInterval("P2DT1M");
$inverted->invert = true;
$datetime->add($inverted);
echo $datetime->format("Y-m-d H:i:s"), "\n";
date_sub($datetime, $inverted);
echo $datetime->format("Y-m-d H:i:s"), "\n";
date_add($datetime, new DateInterval("P1Y2MT23H43M150S"));
echo $datetime->format("Y-m-d H:i:s"), "\n";
date_default_timezone_set("Europe/London");
$immutable = new DateTimeImmutable("2012-12-27 16:24:08");
$added = $immutable->add(new DateInterval("P2DT2S"));
$subtracted = $immutable->sub(new DateInterval("P2DT2S"));
echo $immutable->format("Y-m-d H:i:s e"), "|", $added->format("Y-m-d H:i:s e"), "|", $subtracted->format("Y-m-d H:i:s e"), "\n";
$tokyo = $immutable->setTimezone(new DateTimeZone("Asia/Tokyo"));
echo $tokyo->format("Y-m-d H:i:s e T P"), "\n";
echo is_callable([$datetime, "add"]) ? "add-method" : "missing";
echo "|", is_callable([$immutable, "sub"]) ? "sub-method" : "missing";
echo "|", function_exists("date_add") ? "date-add-fn" : "missing";
echo "|", function_exists("date_sub") ? "date-sub-fn" : "missing";
echo "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "2011-07-06 00:55:05|same\n",
            "2011-07-04 00:54:05\n",
            "2011-07-06 00:55:05\n",
            "2012-09-07 00:40:35\n",
            "2012-12-27 16:24:08 Europe/London|2012-12-29 16:24:10 Europe/London|2012-12-25 16:24:06 Europe/London\n",
            "2012-12-28 01:24:08 Asia/Tokyo JST +09:00\n",
            "add-method|sub-method|date-add-fn|date-sub-fn\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn datetime_diff_loop_uses_cached_interval_state_and_direct_offset_reads() {
    let execution = run_source(
        r#"<?php
date_default_timezone_set("UTC");
$ok = 0;
$d0 = new DateTime("2009-11-20");
for ($i = 0; $i < 18; $i++) {
    $d = clone $d0;
    $dates[$i] = $d->add(new DateInterval("P{$i}D"));
}

for ($i = 0; $i < 6; $i++) {
    for ($j = 0; $j < 18; $j++) {
        $diff = date_diff($dates[$i], $dates[$j]);
        $current = clone $dates[$i];
        $int = new DateInterval($diff->format("P%yY%mM%dD"));
        if ($current > $dates[$j]) {
            $current->sub($int);
        } else {
            $current->add($int);
        }
        if ($current == $dates[$j]) {
            $ok++;
        }
    }
}

$inverted = new DateInterval("P2D");
$inverted->invert = true;
$probe = new DateTime("2009-11-20");
$probe->add($inverted);
echo $ok, "|", $probe->format("Y-m-d"), "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "108|2009-11-18\n");
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn datetime_interval_add_sub_use_named_us_dst_transition_boundaries() {
    let execution = run_source(
        r#"<?php
date_default_timezone_set("America/New_York");

$spring = new DateTime("2010-03-13 18:38:28");
$spring->add(new DateInterval("PT7H38M27S"));
echo $spring->format("Y-m-d H:i:s T e"), "\n";

$springBack = new DateTime("2010-03-15 19:59:59");
$springBack->sub(new DateInterval("P1DT18H49M39S"));
echo $springBack->format("Y-m-d H:i:s T e"), "\n";

$fall = new DateTime("2010-11-06 18:38:28");
$fall->add(new DateInterval("PT7H36M16S"));
echo $fall->format("Y-m-d H:i:s T e"), "\n";

$fallBack = new DateTime("2010-11-07 03:16:55");
$fallBack->sub(new DateInterval("PT4H6M35S"));
echo $fallBack->format("Y-m-d H:i:s T e"), "\n";

$calendar = new DateTime("2010-11-06 18:38:28");
$calendar->add(new DateInterval("P2DT1H21M31S"));
echo $calendar->format("Y-m-d H:i:s T e"), "\n";

$fixed = new DateTime("2010-03-14 01:59:59 EST");
$fixed->add(new DateInterval("PT1S"));
echo $fixed->format("Y-m-d H:i:s T e"), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "2010-03-14 03:16:55 EDT America/New_York\n",
            "2010-03-14 00:10:20 EST America/New_York\n",
            "2010-11-07 01:14:44 EST America/New_York\n",
            "2010-11-07 00:10:20 EDT America/New_York\n",
            "2010-11-08 19:59:59 EST America/New_York\n",
            "2010-03-14 02:00:00 EST EST\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn datetime_microsecond_format_create_from_format_and_diff_are_bounded() {
    let execution = run_source(
        r#"<?php
date_default_timezone_set("UTC");
$date = date_create_from_format("Y-m-d H:i:s.u", "2016-10-03 12:47:18.819313");
echo $date->format("Y-m-d H:i:s.u"), "|", $date->getMicrosecond(), "\n";
$date->setMicrosecond(12);
echo $date->format("Y-m-d H:i:s.u"), "|", $date->getMicrosecond(), "\n";

$start = new DateTimeImmutable("2016-10-03 13:20:07.103123");
$target = DateTimeImmutable::createFromFormat("Y-m-d H:i:s.u", "2016-10-03 13:20:07.481312");
$diff = $start->diff($target);
echo $diff->format("%s"), "|", $diff->f, "\n";
echo $target->add($diff)->format("Y-m-d H:i:s.u"), "\n";

$rfc = DateTime::createFromFormat(DateTime::RFC3339_EXTENDED, "2018-10-09T09:56:45.412+01:00");
echo $rfc->format("Y-m-d H:i:s.u P"), "\n";
try {
    $date->setMicrosecond(-1);
} catch (Throwable $e) {
    echo get_class($e), ": ", $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "2016-10-03 12:47:18.819313|819313\n",
            "2016-10-03 12:47:18.000012|12\n",
            "0|0.378189\n",
            "2016-10-03 13:20:07.859501\n",
            "2018-10-09 09:56:45.412000 +01:00\n",
            "DateRangeError: DateTime::setMicrosecond(): Argument #1 ($microsecond) must be between 0 and 999999, -1 given\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn date_parse_and_parse_from_format_return_bounded_metadata() {
    let execution = run_source(
        r#"<?php
date_default_timezone_set("UTC");
$full = date_parse("2009-02-27 10:00:00.5");
echo $full["year"], "-", $full["month"], "-", $full["day"], "|", $full["hour"], ":", $full["minute"], ":", $full["second"], "|", $full["fraction"], "\n";
$bad = date_parse("2006-02-30");
echo $bad["warning_count"], "|", $bad["warnings"][11], "\n";
$zone = date_parse("2006-12--12");
echo $zone["day"], "|", $zone["error_count"], "|", $zone["zone_type"], "|", $zone["zone"], "\n";
$time = date_parse("10:00:00.5");
echo ($time["year"] === false ? "no-year" : "year"), "|", $time["hour"], "|", $time["fraction"], "\n";
$empty = date_parse("");
echo $empty["error_count"], "|", $empty["errors"][0], "\n";
$format = date_parse_from_format("!m*d*y", "06/08/04");
echo $format["year"], "-", $format["month"], "-", $format["day"], "|", $format["hour"], ":", $format["minute"], ":", $format["second"], "|", ($format["is_localtime"] ? "local" : "not-local"), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "2009-2-27|10:0:0|0.5\n",
            "1|The parsed date was invalid\n",
            "1|1|1|-43200\n",
            "no-year|10|0.5\n",
            "1|Empty string\n",
            "2004-6-8|0:0:0|not-local\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn create_from_format_parses_bounded_rfc_and_textual_formats() {
    let execution = run_source(
        r#"<?php
date_default_timezone_set("Europe/Oslo");
$date = new DateTime("2008-07-08T22:14:12+02:00");
foreach ([DATE_ATOM, DATE_COOKIE, DATE_ISO8601, DATE_RFC822, DATE_RFC3339_EXTENDED] as $format) {
    $parsed = date_create_from_format($format, $date->format($format));
    echo $format, "|", $parsed->format("Y-m-d H:i:s.u e"), "\n";
}
date_default_timezone_set("Europe/London");
$textual = DateTime::createFromFormat("D., M# j, Y g:iA", "Thu., Nov. 29, 2012 5:00PM");
echo $textual->format("D., M. j, Y g:iA"), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "Y-m-d\\TH:i:sP|2008-07-08 22:14:12.000000 +02:00\n",
            "l, d-M-Y H:i:s T|2008-07-08 22:14:12.000000 +02:00\n",
            "Y-m-d\\TH:i:sO|2008-07-08 22:14:12.000000 +02:00\n",
            "D, d M y H:i:s O|2008-07-08 22:14:12.000000 +02:00\n",
            "Y-m-d\\TH:i:s.vP|2008-07-08 22:14:12.000000 +02:00\n",
            "Thu., Nov. 29, 2012 5:00PM\n",
        )
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
