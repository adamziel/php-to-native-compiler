<?php
echo is_array($_GET) ? "get-array" : "get-missing";
echo "|";
echo is_array($_POST) ? "post-array" : "post-missing";
echo "|";
echo is_array($_REQUEST) ? "request-array" : "request-missing";
echo "|";
echo isset($_GET["preview"]) ? "preview" : "get-empty";

function wp_request_probe() {
    $_GET["preview"] = "true";
    $_POST["action"] = "save";
    $_REQUEST["preview"] = $_GET["preview"];
    $_REQUEST["action"] = $_POST["action"];
}

wp_request_probe();
echo "|";
echo $_GET["preview"];
echo "|";
echo $_POST["action"];
echo "|";
echo $_REQUEST["preview"];
echo ":";
echo $_REQUEST["action"];
