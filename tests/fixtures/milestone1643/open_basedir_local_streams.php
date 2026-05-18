<?php
$base = realpath(__DIR__);
$allowed = $base . "/allowed";
$denied = $base . "/denied";

function capture_open_basedir_warning($errno, $errstr) {
    if (str_contains($errstr, "open_basedir")) {
        echo "|warning:" . $errno . ":basedir";
    }
    return true;
}

ini_set("open_basedir", $allowed);
set_error_handler("capture_open_basedir_warning", E_WARNING);

echo rtrim(file_get_contents($allowed . "/payload.txt"));
$blocked_read = file_get_contents($denied . "/secret.txt");
echo $blocked_read === false ? "|read-blocked" : "|read-open";

$stream = fopen("file://" . $allowed . "/stream.txt", "r");
echo "|" . fread($stream, 6);
fclose($stream);

$blocked_stream = fopen($denied . "/secret.txt", "r");
echo $blocked_stream === false ? "|fopen-blocked" : "|fopen-open";
