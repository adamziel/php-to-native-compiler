<?php
$memory = fopen("php://memory", "w+");
fwrite($memory, "wp-cache");
echo ftell($memory);
echo "|" . (feof($memory) ? "eof" : "more");
fseek($memory, -5, SEEK_CUR);
echo "|" . ftell($memory);
echo "|" . fread($memory, 2);
echo "|" . (feof($memory) ? "eof" : "more");
fseek($memory, 0, SEEK_END);
echo "|" . (feof($memory) ? "eof" : "more");
fclose($memory);

$path = "/tmp/phpc_milestone1338_stream_status_seek.txt";
$file = fopen($path, "w+");
fwrite($file, "plugin-cache");
echo "|" . ftell($file);
fseek($file, -5, SEEK_END);
echo "|" . ftell($file);
echo "|" . fread($file, 5);
echo "|" . (feof($file) ? "eof" : "more");
echo "|" . (fclose($file) ? "closed" : "open");
