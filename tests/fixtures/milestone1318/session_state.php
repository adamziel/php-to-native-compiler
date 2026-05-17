<?php
$before = session_status();
$old = session_id("phpcmilestone1318");
$started = session_start();
$_SESSION["auth"]["user"] = "admin";
$_SESSION["auth"]["roles"][] = "editor";
$_SESSION["count"] = 2;
$closed = session_write_close();

echo ($before === PHP_SESSION_NONE ? "none" : "other");
echo "|";
echo ($old === "" ? "empty-id" : "had-id");
echo "|";
echo ($started ? "started" : "failed");
echo "|";
echo session_id();
echo "|";
echo $_SESSION["auth"]["user"];
echo ":";
echo $_SESSION["auth"]["roles"][0];
echo ":";
echo $_SESSION["count"];
echo "|";
echo ($closed ? "closed" : "close-failed");
echo "|";
echo (session_status() === PHP_SESSION_NONE ? "none" : "active");
