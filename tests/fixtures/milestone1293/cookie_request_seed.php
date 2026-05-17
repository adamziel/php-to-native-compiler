<?php
echo isset($_COOKIE["wordpress_test_cookie"]) ? $_COOKIE["wordpress_test_cookie"] : "empty";
echo "|";
echo isset($_COOKIE["logged_in"]) ? $_COOKIE["logged_in"] : "empty";
echo "|";
echo isset($_COOKIE["settings"]["theme"]) ? $_COOKIE["settings"]["theme"] : "empty";
echo "|";
echo isset($_COOKIE["dotted_name"]) ? $_COOKIE["dotted_name"] : "empty";
echo "|";
echo isset($_GET["preview"]) ? $_GET["preview"] : "empty";
echo "|";
echo isset($_REQUEST["wordpress_test_cookie"]) ? $_REQUEST["wordpress_test_cookie"] : "request-empty";
echo "|";
echo $_SERVER["HTTP_COOKIE"];
