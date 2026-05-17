<?php
function capture_include_warning($errno, $errstr, $errfile, $errline) {
    echo "warning:" . $errno;
    echo ":" . (str_contains($errstr, "missing-wordpress-") ? "path" : "missing");
    echo ":" . (str_contains($errstr, "Failed to open stream") ? "open" : (str_contains($errstr, "Failed opening") ? "opening" : "other"));
    echo ":" . basename($errfile);
    echo ":" . $errline;
    echo "\n";
    return true;
}

set_error_handler("capture_include_warning", E_WARNING);
$result = include __DIR__ . "/missing-wordpress-optional.php";
echo "include=" . ($result === false ? "false" : "value") . "\n";

$once = include_once __DIR__ . "/missing-wordpress-once.php";
echo "include_once=" . ($once === false ? "false" : "value") . "\n";

include __DIR__ . "/loaded_after_missing.inc";
echo "loaded=" . $loaded_after_missing;
