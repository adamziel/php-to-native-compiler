<?php
function milestone1473_session_warning($errno, $errstr, $errfile, $errline) {
    echo "warning:" . $errno;
    echo ":" . (str_contains($errstr, "bounded file-safe subset") ? "id" : "other");
    echo ":" . basename($errfile) . ":" . $errline;
    return true;
}

set_error_handler("milestone1473_session_warning", E_WARNING);
session_id("bad/slash");
$started = session_start(["use_cookies" => false]);
echo "|return:" . ($started ? "true" : "false");
echo "|status:" . (session_status() === PHP_SESSION_NONE ? "none" : "active");
echo "|headers:" . count(headers_list());
echo "|session:" . (isset($_SESSION) ? "set" : "unset");
