<?php
function capture_fopen_warning($errno, $errstr, $errfile, $errline) {
    echo "warning:" . $errno;
    echo ":" . (str_contains($errstr, "missing-wordpress-stream") ? "path" : "missing");
    echo ":" . (str_contains($errstr, "Failed to open stream") ? "open" : "other");
    echo ":" . basename($errfile);
    echo ":" . $errline;
    echo "\n";
    return true;
}

set_error_handler("capture_fopen_warning", E_WARNING);
$stream = fopen(__DIR__ . "/missing-wordpress-stream.txt", "r");
echo "fopen=" . ($stream === false ? "false" : "resource") . "\n";
echo "continued=after-fopen";
