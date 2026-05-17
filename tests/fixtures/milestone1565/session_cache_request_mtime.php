<?php
session_cache_limiter("public");
session_cache_expire(2);
session_id("phpcmilestone1565mtime");
session_start(array("use_cookies" => false));
$headers = headers_list();
$expected = $_SERVER["REQUEST_TIME"] === 978307200
    ? "Expires: Mon, 01 Jan 2001 00:02:00 GMT"
    : "Expires: Thu, 01 Jan 1970 00:02:00 GMT";

echo $_SERVER["REQUEST_TIME"];
echo "|";
echo $headers[0] === $expected ? "request-expires" : "bad-expires";
echo "|";
echo $headers[1] === "Cache-Control: public, max-age=120" ? "public-cache" : "bad-cache";
echo "|";
echo str_contains($headers[2], "1970") ? "stale-mtime" : "script-mtime";
echo "|";
echo str_contains($headers[2], "GMT") ? "gmt" : "bad-date";
