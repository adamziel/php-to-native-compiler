<?php
session_id("phpcmilestone1458");
$out = array();
$out[] = session_start() ? "started" : "failed";
$out[] = session_status() === PHP_SESSION_ACTIVE ? "active" : "closed";
$headers = headers_list();
$out[] = count($headers);
$out[] = $headers[0];
session_write_close();
header_remove();
session_id("phpcmilestone1458nocookie");
$out[] = session_start(["use_cookies" => false]) ? "no-cookie-started" : "no-cookie-failed";
$out[] = session_status() === PHP_SESSION_ACTIVE ? "active" : "closed";
$out[] = count(headers_list());
echo implode("|", $out);
