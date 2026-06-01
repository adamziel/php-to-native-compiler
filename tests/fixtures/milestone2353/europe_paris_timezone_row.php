<?php
$name = timezone_name_from_abbr("", 3600, 0);
echo $name, "\n";
$tz = timezone_open($name);
echo $tz->getName(), "\n";
date_default_timezone_set($name);
$winter = strtotime("2020-01-01 12:00:00");
$summer = strtotime("2020-07-01 12:00:00");
echo date("T|Z|Y-m-d H:i", $winter), "\n";
echo date("T|Z|Y-m-d H:i", $summer), "\n";
$zones = timezone_identifiers_list(DateTimeZone::EUROPE);
echo in_array("Europe/Paris", $zones) ? "listed" : "missing", "\n";
