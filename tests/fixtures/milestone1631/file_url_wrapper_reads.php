<?php
$path = realpath(__DIR__ . "/file_url_payload.txt");
$url = "file://" . $path;
echo file_get_contents($url, false, null, 7, 6);
echo "|";

$stream = fopen($url, "r");
$meta = stream_get_meta_data($stream);
echo $meta["wrapper_type"];
echo ":";
echo $meta["stream_type"];
echo ":";
echo str_contains($meta["uri"], "file://") ? "file-url" : "other";
echo ":";
echo fread($stream, 8);
fclose($stream);
echo "|";

$include_url = "file://" . realpath(__DIR__ . "/file_url_include.inc");
$include_result = include $include_url;
echo "include=" . $include_result . ":" . $included_from_url;
echo "|";

$require_url = "file://" . realpath(__DIR__ . "/file_url_required.inc");
$require_result = require_once $require_url;
echo "require=" . $require_result . ":" . $required_from_url;
$again = require_once $require_url;
echo "|again=" . ($again === true ? "true" : "other");
