<?php
session_cache_limiter("");
$out = array();
setcookie("PHPSESSID", "manual", 0, "/wp-admin", "EXAMPLE.test");
setcookie("PHPSESSID", "root", 0, "/");
session_id("firstid");
session_start(array(
    "cookie_path" => "/wp-admin",
    "cookie_domain" => "example.test",
    "cookie_secure" => true,
    "cookie_httponly" => true,
));
$headers = headers_list();
$out[] = count($headers);
$out[] = $headers[0];
$out[] = $headers[1];
session_write_close();
session_id("secondid");
session_start(array(
    "cookie_path" => "/wp-admin",
    "cookie_domain" => "EXAMPLE.TEST",
));
$headers = headers_list();
$out[] = count($headers);
$out[] = $headers[0];
$out[] = $headers[1];
echo implode("|", $out);
