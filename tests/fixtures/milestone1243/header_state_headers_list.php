<?php
$initial = count(headers_list());
header("X-WordPress-Test: one");
header("Content-Type: text/html; charset=UTF-8", true, 200);
header("X-WordPress-Test: two", false);
$headers = headers_list();
echo $initial;
echo "|";
echo count($headers);
echo "|";
echo $headers[0];
echo "|";
echo $headers[1];
echo "|";
echo $headers[2];
