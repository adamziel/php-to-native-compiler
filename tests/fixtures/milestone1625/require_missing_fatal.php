<?php
function capture_require_warning($errno, $errstr, $errfile, $errline) {
    echo "warning:" . $errno;
    echo ":" . (str_contains($errstr, "missing-wordpress-required") ? "path" : "missing");
    echo ":" . (str_contains($errstr, "Failed to open stream") ? "open" : (str_contains($errstr, "Failed opening") ? "opening" : "other"));
    echo ":" . basename($errfile) . ":" . $errline . "|";
    return true;
}

set_error_handler("capture_require_warning", E_WARNING);
require "missing-wordpress-required.php";
echo "not reached";
