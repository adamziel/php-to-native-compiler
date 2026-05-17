<?php
$out = array();
$slots = array();
$file =& $slots["file"];
$line =& $slots["line"];
$call = "headers_sent";
$before = $call($file, $line);
$out[] = ($before ? "sent" : "open") . ":" . $slots["file"] . ":" . $slots["line"];
echo "body";
$after = headers_sent($file, $line);
$out[] = ($after ? "sent" : "open") . ":" . basename($slots["file"]) . ":" . $slots["line"];
$direct_file = "old";
$direct_line = -1;
$again = $call($direct_file, $direct_line);
$out[] = ($again ? "sent" : "open") . ":" . basename($direct_file) . ":" . $direct_line;
echo "|" . implode("|", $out);
