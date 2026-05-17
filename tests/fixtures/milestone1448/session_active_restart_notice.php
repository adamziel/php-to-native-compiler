<?php
function active_session_notice($errno, $errstr, $errfile, $errline) {
    echo "notice:" . $errno;
    echo ":" . (str_contains($errstr, "already active") ? "active" : "other");
    echo ":" . basename($errfile) . ":" . $errline;
    return true;
}
session_id("phpcmilestone1448");
$first = session_start();
$_SESSION["phase"] = "open";
set_error_handler("active_session_notice", E_NOTICE);
$second = session_start(["read_and_close" => true]);
echo "|" . ($first ? "first" : "first-failed");
echo "|" . ($second ? "second" : "second-failed");
echo "|" . (session_status() === PHP_SESSION_ACTIVE ? "active" : "closed");
echo "|" . $_SESSION["phase"];
