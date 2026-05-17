<?php
session_id("phpcmilestone1548");
$out = array();
$started = session_start(array("use_cookies" => false));
$headers = headers_list();
$out[] = $started ? "started" : "failed";
$out[] = session_status() === PHP_SESSION_ACTIVE ? "active" : "inactive";
$out[] = count($headers);
$out[] = $headers[0];
$out[] = $headers[1];
$out[] = $headers[2];
echo implode("|", $out);
