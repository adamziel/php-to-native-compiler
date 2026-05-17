<?php
setcookie("wordpress_logged_in", "user token", ["Path" => "/first", "path" => "/second", "Domain" => "Example.TEST", "domain" => "lower.test", "Secure" => false, "secure" => true, "HttpOnly" => false, "httponly" => true, "SameSite" => "Lax", "samesite" => "Strict"]);
setrawcookie("wordpress_sec", "raw token", ["expires" => 2000000000, "Expires" => 1, "PATH" => "/old", "Path" => "/new"]);
$headers = headers_list();
echo count($headers) . "\n";
echo $headers[0] . "\n";
echo $headers[1];
