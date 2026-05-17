<?php
header("X-WordPress-Test: one");
header("Content-Type: text/html; charset=UTF-8", true, 200);
header("X-WordPress-Test: two", false);
header_remove("X-WordPress-Test");
$headers = headers_list();
$count = count($headers);
$first = $headers[0];
header_remove();
$final = count(headers_list());
echo $count;
echo "|";
echo $first;
echo "|";
echo $final;
