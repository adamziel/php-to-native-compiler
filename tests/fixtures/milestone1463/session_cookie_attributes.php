<?php
session_id("phpcmilestone1463");
$out = array();
$out[] = session_start([
    "cookie_lifetime" => 7200,
    "cookie_path" => "/wp-admin",
    "cookie_domain" => "example.test",
    "cookie_secure" => true,
    "cookie_httponly" => true,
    "cookie_samesite" => "Strict",
]) ? "started" : "failed";
$out[] = session_status() === PHP_SESSION_ACTIVE ? "active" : "closed";
$headers = headers_list();
$out[] = count($headers);
$out[] = $headers[0];
echo implode("|", $out);
