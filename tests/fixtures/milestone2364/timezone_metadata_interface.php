<?php
echo interface_exists("DateTimeInterface") ? "iface\n" : "missing\n";
echo DATE_RFC3339 === DateTimeInterface::RFC3339 ? "const\n" : "bad\n";

$abbreviations = timezone_abbreviations_list();
echo count($abbreviations), "|", count($abbreviations["acst"]), "|";
echo $abbreviations["acst"][0]["timezone_id"], "|", $abbreviations["acst"][5]["timezone_id"], "\n";

$oslo = timezone_location_get(new DateTimeZone("Europe/Oslo"));
echo $oslo["country_code"], "|", $oslo["latitude"], "|", $oslo["longitude"], "|", $oslo["comments"], "\n";

$tz = new DateTimeZone("Europe/London");
$dt = new DateTimeImmutable("2014-09-20", $tz);
echo $tz->getOffset($dt), "|", timezone_offset_get($tz, $dt), "\n";

try {
    timezone_offset_get($tz, 1);
} catch (Throwable $e) {
    echo $e::class, ": ", $e->getMessage();
}
