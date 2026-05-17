<?php
setcookie("wordpress_logged_in", "user token", ["Expires" => 1, "Path" => "/wp-admin", "Domain" => "Example.TEST", "Secure" => true, "HttpOnly" => true, "SameSite" => "Lax"]);
setrawcookie("wordpress_sec", "raw token", ["PATH" => "/", "DOMAIN" => "EXAMPLE.test", "SECURE" => true, "HTTPONLY" => true, "SAMESITE" => "Strict"]);
$headers = headers_list();
echo count($headers) . "\n";
echo $headers[0] . "\n";
echo $headers[1];
