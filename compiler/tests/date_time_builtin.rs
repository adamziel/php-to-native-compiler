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
fn datetime_format_constants_match_global_constants() {
    let execution = run_source(
        r#"<?php
var_dump(
    DATE_ATOM === DateTimeInterface::ATOM,
    DATE_COOKIE === DateTime::COOKIE,
    DATE_ISO8601 === DateTimeImmutable::ISO8601,
    DATE_ISO8601_EXPANDED === DateTimeInterface::ISO8601_EXPANDED,
    DATE_RFC822 === DateTime::RFC822,
    DATE_RFC850 === DateTimeImmutable::RFC850,
    DATE_RFC1036 === DateTimeInterface::RFC1036,
    DATE_RFC1123 === DateTime::RFC1123,
    DATE_RFC7231 === DateTimeImmutable::RFC7231,
    DATE_RFC2822 === DateTimeInterface::RFC2822,
    DATE_RFC3339 === DateTime::RFC3339,
    DATE_RFC3339_EXTENDED === DateTimeImmutable::RFC3339_EXTENDED,
    DATE_RSS === DateTimeInterface::RSS,
    DATE_W3C === DateTime::W3C
);
"#,
    )
    .unwrap();

    assert!(execution.stdout.contains(
        "Deprecated: Constant DATE_RFC7231 is deprecated since 8.5, as this format ignores the associated timezone and always uses GMT in Command line code on line 11"
    ));
    assert!(execution.stdout.contains(
        "Deprecated: Constant DateTimeInterface::RFC7231 is deprecated since 8.5, as this format ignores the associated timezone and always uses GMT in Command line code on line 11"
    ));
    assert_eq!(execution.stdout.matches("bool(true)\n").count(), 14);
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
fn timezone_location_metadata_and_open_warnings_cover_current_rows() {
    let execution = run_source(
        r#"<?php
$location = timezone_location_get(new DateTimeZone("Europe/Oslo"));
echo $location["country_code"], "|", $location["latitude"], "|", $location["longitude"], "|", $location["comments"], "\n";
var_dump((new DateTimeZone("UTC"))->getLocation());
$seen = [];
foreach (DateTimeZone::listAbbreviations() as $value) {
    if (NULL != $value[0]["timezone_id"]) {
        $tz = new DateTimeZone($value[0]["timezone_id"]);
        $tz_location = $tz->getLocation();
        if ($tz_location !== false && in_array($tz_location["country_code"], ["AU", "CA", "ET", "US"]) && !in_array($tz_location["country_code"], $seen)) {
            $seen[] = $tz_location["country_code"];
            echo $tz_location["country_code"], "\n";
        }
    }
}
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

    assert!(execution.stdout.starts_with(
        "NO|59.91666|10.75|\nbool(false)\nAU\nCA\nET\nUS\nIn: +02:30; Out: +02:30\nIn: Europe/Kyiv; Out: Europe/Kyiv\n"
    ));
    assert!(execution
        .stdout
        .contains("Warning: timezone_open(): Unknown or bad timezone (2.5)"));
    assert!(execution
        .stdout
        .contains("Warning: timezone_open(): Unknown or bad timezone (99:60)"));
    assert!(execution
        .stdout
        .contains("Warning: timezone_open(): Unknown or bad timezone (Europe/Lviv)"));
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
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "offsets=0|1\nEurope/London\nAmerica/New_York\nAmerica/Los_Angeles|same\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}
