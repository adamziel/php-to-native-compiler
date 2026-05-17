<?php
$out = array();
setcookie("wordpress_test_cookie", "old value");
$out[] = count(headers_list());
setcookie("wordpress_test_cookie", "WP Cookie check", 1700000000, "/wp-admin", "example.test", true, true);
setcookie("logged_in", "delete me", ["expires" => 1, "path" => "/", "secure" => false, "httponly" => true, "samesite" => "Lax"]);
$headers = headers_list();
$out[] = count($headers);
$out[] = $headers[0];
$out[] = $headers[1];
echo implode("\n", $out);
