<?php
function late_header_warning($errno, $errstr, $errfile, $errline) {
    echo "|warn:" . $errno;
    echo ":" . (str_contains($errstr, "Cannot modify header information") ? "cannot" : "other");
    echo ":" . (str_contains($errstr, "output started at") ? "started" : "missing");
    echo ":" . basename($errfile) . ":" . $errline;
    return true;
}
header("X-Before: one");
set_error_handler("late_header_warning", E_WARNING);
echo "body";
$header_result = header("X-Late: two");
$cookie_result = setcookie("late_cookie", "1");
$remove_result = header_remove("X-Before");
echo "|returns:" . ($header_result === null ? "null" : "other");
echo ":" . ($cookie_result ? "cookie-true" : "cookie-false");
echo ":" . ($remove_result === null ? "remove-null" : "remove-other");
restore_error_handler();
