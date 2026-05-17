<?php
setcookie("wordpress_test_cookie", "WP Cookie check");
setcookie("empty_cookie");
$headers = headers_list();
echo count($headers);
echo "|";
echo $headers[0];
echo "|";
echo $headers[1];
