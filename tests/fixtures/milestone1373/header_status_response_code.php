<?php
$out = array();
$initial = http_response_code();
$out[] = $initial === false ? "false" : (string) $initial;
$previous = http_response_code(201);
$out[] = $previous === true ? "true" : (string) $previous;
$out[] = (string) http_response_code();
header("Location: /wp-admin/");
$out[] = (string) http_response_code();
http_response_code(404);
header("Location: /wp-login.php");
$out[] = (string) http_response_code();
header("HTTP/1.1 503 Service Unavailable");
$out[] = (string) http_response_code();
header("X-Test: one", true, 204);
$out[] = (string) http_response_code();
header("HTTP/1.1 500 Internal Server Error", true, 0);
$out[] = (string) http_response_code();
echo implode("|", $out);
