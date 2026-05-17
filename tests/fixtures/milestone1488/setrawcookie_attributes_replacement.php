<?php
$out = array();
setrawcookie("wordpress_test_cookie", "old raw");
$out[] = count(headers_list());
setrawcookie("wordpress_test_cookie", "WP Cookie check", 1700000000, "/wp-admin", "example.test", true, true);
setrawcookie("logged_in", "delete me", ["expires" => 1, "path" => "/", "secure" => false, "httponly" => true, "samesite" => "Strict"]);
$headers = headers_list();
$out[] = count($headers);
$out[] = $headers[0];
$out[] = $headers[1];
echo implode("\n", $out);
