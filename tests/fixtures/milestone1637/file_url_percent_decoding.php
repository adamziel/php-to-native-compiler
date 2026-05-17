<?php
$base = realpath(__DIR__);
$payload = file_get_contents("file://" . $base . "/file%20url%20payload.txt", false, null, 8, 7);
echo $payload;
echo "|";

$stream = fopen("file://localhost" . $base . "/file%20url%20stream%20%23name.txt", "r");
echo fread($stream, 6);
fclose($stream);
echo "|";

$include_result = include "file://" . $base . "/file%20url%20include%20%23encoded.inc";
echo "include=" . $include_result . ":" . $included_from_percent_url;
echo "|";

$require_result = require_once "file://" . $base . "/file%20url%20required%20%2Bencoded.inc";
echo "require=" . $require_result . ":" . $required_from_percent_url;
