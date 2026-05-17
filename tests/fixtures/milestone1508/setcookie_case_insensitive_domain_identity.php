<?php
setcookie("wordpress_test_cookie", "upper", 0, "/wp-admin", "EXAMPLE.test");
setrawcookie("wordpress_test_cookie", "mixed", 0, "/wp-admin", "Example.TEST");
setcookie("wordpress_test_cookie", "lower", 0, "/wp-admin", "example.test");
setcookie("wordpress_test_cookie", "network", 0, "/wp-admin", "network.example.test");
$headers = headers_list();
echo count($headers);
echo "\n";
echo $headers[0];
echo "\n";
echo $headers[1];
