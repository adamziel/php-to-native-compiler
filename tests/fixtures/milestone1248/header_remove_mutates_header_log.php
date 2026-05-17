<?php
header("X-WordPress-Test: one");
header("Content-Type: text/html; charset=UTF-8", true, 200);
header("X-WordPress-Test: two", false);
header_remove("X-WordPress-Test");
$headers = headers_list();
echo count($headers);
echo "|";
echo $headers[0];
header_remove();
echo "|";
echo count(headers_list());
