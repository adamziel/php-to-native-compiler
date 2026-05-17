<?php
session_start();
$_SESSION["payload"] = ["slot" => "session"];
$alias =& $_SESSION["payload"]["slot"];

function wp_refcow_session_touch($suffix) {
    $_SESSION["payload"]["slot"] = $_SESSION["payload"]["slot"] . ":" . $suffix;
}

wp_refcow_session_touch("function");
$alias = $alias . ":alias";
echo $_SESSION["payload"]["slot"], "|", $alias, "\n";

session_write_close();
wp_refcow_session_touch("closed");
echo $_SESSION["payload"]["slot"], "|", $alias;
