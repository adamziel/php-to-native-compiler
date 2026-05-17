<?php
$memory = fopen("php://memory", "w+");
fwrite($memory, "wp-body");
$memory_meta = stream_get_meta_data($memory);
$memory_stat = fstat($memory);
echo $memory_meta["wrapper_type"] . ":" . $memory_meta["stream_type"];
echo ":" . $memory_meta["mode"] . ":" . $memory_meta["uri"];
echo ":" . $memory_stat["size"] . ":" . $memory_stat[7];
fread($memory, 16);
$memory_eof = stream_get_meta_data($memory);
echo ":" . ($memory_eof["eof"] ? "eof" : "more");
fclose($memory);

$temp = fopen("php://temp", "w+b");
fwrite($temp, "temp-cache");
$temp_meta = stream_get_meta_data($temp);
$temp_stat = fstat($temp);
echo "|" . $temp_meta["stream_type"] . ":" . $temp_meta["mode"];
echo ":" . $temp_meta["uri"] . ":" . $temp_stat["size"];
fclose($temp);

$path = "/tmp/phpc_milestone1343_stream_metadata.txt";
$file = fopen($path, "w+");
fwrite($file, "plugin-cache");
$file_meta = stream_get_meta_data($file);
$file_stat = fstat($file);
echo "|" . $file_meta["wrapper_type"] . ":" . $file_meta["stream_type"];
echo ":" . $file_meta["mode"] . ":" . ($file_meta["seekable"] ? "seekable" : "fixed");
echo ":" . ($file_meta["uri"] === $path ? "same-uri" : "other-uri");
echo ":" . $file_stat["size"] . ":" . $file_stat[7];
echo ":" . ($file_stat["mode"] > 0 ? "mode" : "no-mode");
echo "|" . (fclose($file) ? "closed" : "open");
