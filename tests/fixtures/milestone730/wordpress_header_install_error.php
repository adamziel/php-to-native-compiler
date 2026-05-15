<?php
$protocol = "HTTP/1.1";
header(sprintf('%s 500 Internal Server Error', $protocol), true, 500);
header('Content-Type: text/html; charset=utf-8');
echo "install-error";

