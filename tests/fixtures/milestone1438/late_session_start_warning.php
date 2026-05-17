<?php
function late_session_warning($errno, $errstr, $errfile, $errline) {
    echo "|warn:" . $errno;
    echo ":" . (str_contains($errstr, "Session cannot be started") ? "session" : "other");
    echo ":" . (str_contains($errstr, "headers have already been sent") ? "headers" : "missing");
    echo ":" . basename($errfile) . ":" . $errline;
    return true;
}
set_error_handler("late_session_warning", E_WARNING);
echo "body";
$started = session_start();
echo "|return:" . ($started ? "true" : "false");
echo "|status:" . (session_status() === PHP_SESSION_NONE ? "none" : "active");
