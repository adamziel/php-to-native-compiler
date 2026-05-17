<?php
session_id("phpcmilestone1443");
$out = array();
$out[] = session_start(["read_and_close" => true]) ? "started" : "failed";
$out[] = session_status() === PHP_SESSION_NONE ? "closed" : "active";
$out[] = session_id();
$_SESSION["after"] = "visible";
$out[] = $_SESSION["after"];
$out[] = session_start(["read_and_close" => false]) ? "again-started" : "again-failed";
$out[] = session_status() === PHP_SESSION_ACTIVE ? "active" : "closed";
$_SESSION["during"] = "open";
session_write_close();
$out[] = session_status() === PHP_SESSION_NONE ? "closed" : "active";
$out[] = $_SESSION["during"];
echo implode("|", $out);
