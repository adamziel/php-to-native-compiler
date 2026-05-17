<?php
function milestone1478_session_warning($errno, $errstr, $errfile, $errline) {
    echo "warning:" . $errno;
    echo ":" . (str_contains($errstr, "Malformed session file") ? "malformed" : "other");
    echo ":" . basename($errfile) . ":" . $errline;
    return true;
}

ini_set("session.save_path", str_contains(getcwd(), "/compiler") ? "../tests/fixtures/milestone1478" : "tests/fixtures/milestone1478");
set_error_handler("milestone1478_session_warning", E_WARNING);
session_id("phpcmilestone1478");
$started = session_start(["use_cookies" => false]);
echo "|return:" . ($started ? "true" : "false");
echo "|status:" . (session_status() === PHP_SESSION_ACTIVE ? "active" : "none");
echo "|session:" . (isset($_SESSION["token"]) ? $_SESSION["token"] : "empty");
