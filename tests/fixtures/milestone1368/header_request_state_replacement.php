<?php
header("X-Replace: one");
header("X-Keep: one");
header("x-replace: two");
header("X-Keep: two", false);
$open = headers_sent($file, $line);
$headers = headers_list();
echo count($headers) . "|" . $headers[0] . "|" . $headers[1] . "|" . $headers[2] . "|" . ($open ? "sent" : "open") . ":" . $file . ":" . $line;
header("X-Late: ignored");
$sent = headers_sent($sent_file, $sent_line);
$after = headers_list();
echo "|" . ($sent ? "sent" : "open") . ":" . basename($sent_file) . ":" . $sent_line . "|" . count($after);
