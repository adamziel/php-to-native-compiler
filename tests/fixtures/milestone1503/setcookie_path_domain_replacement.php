<?php
setcookie("wordpress_test_cookie", "root", 0, "/");
setcookie("wordpress_test_cookie", "admin", 0, "/wp-admin", "example.test");
setcookie("wordpress_test_cookie", "network", 0, "/wp-admin", "network.example.test");
setcookie("wordpress_test_cookie", "admin-new", 0, "/wp-admin", "example.test");
setrawcookie("wordpress_test_cookie", "root raw", 0, "/");
$headers = headers_list();
echo count($headers);
echo "\n";
echo $headers[0];
echo "\n";
echo $headers[1];
echo "\n";
echo $headers[2];
