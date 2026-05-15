<?php
$result = header("HTTP/1.1 500 Internal Server Error", true, 500);
header("Content-Type: text/html; charset=utf-8");
header("X-No-Replace: one", false);
echo $result === null ? "null" : "not-null";

