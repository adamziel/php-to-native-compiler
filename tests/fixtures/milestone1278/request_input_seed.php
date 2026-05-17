<?php
echo $_SERVER["REQUEST_METHOD"];
echo "|";
echo $_SERVER["QUERY_STRING"];
echo "|";
echo isset($_GET["preview"]) ? $_GET["preview"] : "empty";
echo "|";
echo isset($_GET["name"]) ? $_GET["name"] : "empty";
echo "|";
echo isset($_POST["action"]) ? $_POST["action"] : "empty";
echo "|";
echo isset($_POST["space"]) ? $_POST["space"] : "empty";
echo "|";
echo isset($_REQUEST["preview"]) ? $_REQUEST["preview"] : "empty";
echo "|";
echo isset($_REQUEST["action"]) ? $_REQUEST["action"] : "empty";
echo "|";
echo file_get_contents("php://input");
